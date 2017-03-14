use stm::*;
use std::sync::Arc;
use std::any::Any;
use super::arclist::*;

#[derive(Clone)]
pub struct TQueue<T> {
    read: TVar<ArcList<T>>,
    write: TVar<ArcList<T>>,
}

/// A threadsafe Queue using transactional memory.
impl<T: Any+Sync+Clone+Send> TQueue<T> {
    pub fn new() -> TQueue<T> {
        TQueue {
            read: TVar::new(End),
            write: TVar::new(End),
        }
    }

    pub fn push(&self, trans: &mut Transaction, val: T) -> StmResult<()> {
        let end = self.write.read(trans)?;
        self.write.write(trans, Elem(val, Arc::new(end)))
    }

    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        match self.read.read(trans)? {
            Elem(x, xs)     => {
                self.read.write(trans, (*xs).clone())?;
                Ok(x)
            }
            End             => {
                let write_list = self.write.read(trans)?;
                self.write.write(trans, End);
                match write_list.reverse() {
                    End     => retry()?,
                    Elem(x,xs) => {
                        self.read.write(trans, (*xs).clone());
                        Ok(x)
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use stm::*;

    #[test]
    fn test_queue_push_pop() {
        let mut queue = TQueue::new();
        let x = atomically(|trans| {
            queue.push(trans, 42)?;
            queue.pop(trans)
        });
        assert_eq!(42, x);
    }
    #[test]
    fn test_queue_order() {
        let mut queue = TQueue::new();
        let x = atomically(|trans| {
            queue.push(trans, 1)?;
            queue.push(trans, 2)?;
            queue.push(trans, 3)?;
            let x1 = queue.pop(trans)?;
            let x2 = queue.pop(trans)?;
            let x3 = queue.pop(trans)?;
            Ok((x1,x2,x3))
        });
        assert_eq!((1,2,3), x);
    }

    #[test]
    fn test_queue_multi_transactions() {
        let mut queue = TQueue::new();
        let mut queue2 = queue.clone();

        atomically(|trans| {
            queue2.push(trans, 1)?;
            queue2.push(trans, 2)
        });
        atomically(|trans| {
            queue.push(trans, 3)
        });

        let x = atomically(|trans| {
            let x1 = queue.pop(trans)?;
            let x2 = queue.pop(trans)?;
            let x3 = queue.pop(trans)?;
            Ok((x1,x2,x3))
        });
        assert_eq!((1,2,3), x);
    }

    #[test]
    fn test_queue_threaded() {
        use std::thread;
        use std::time::Duration;
        let mut queue = TQueue::new();

        for i in 0..10 {
            let mut queue2 = queue.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(20-i as u64));
                atomically(|trans| 
                    queue2.push(trans, i)
                );
            });
        }

        let mut v = atomically(|trans| {
            let mut v = Vec::new();
            for i in 0..10 {
                v.push(queue.pop(trans)?);
            }
            Ok(v)
        });

        v.sort();
        for i in 0..10 {
            assert_eq!(v[i],i);
        }
    }
}

