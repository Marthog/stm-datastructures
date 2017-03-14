use stm::*;
use std::sync::Arc;
use std::any::Any;
use super::arclist::*;

#[derive(Clone)]
pub struct Queue<T> {
    read: TVar<ArcList<T>>,
    write: TVar<ArcList<T>>,
}

/// A threadsafe Queue using transactional memory.
impl<T: Any+Sync+Clone+Send> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            read: TVar::new(End),
            write: TVar::new(End),
        }
    }

    pub fn push(&self, trans: &mut Transaction, val: T) -> StmResult<()> {
        self.write.modify(trans, |end| 
            Elem(val, Arc::new(end))
        )
    }

    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        match self.read.read(trans)? {
            Elem(x, xs)     => {
                self.read.write(trans, (*xs).clone())?;
                Ok(x)
            }
            End             => {
                let write_list = self.write.replace(trans, End)?;
                match write_list.reverse() {
                    End     => retry()?,
                    Elem(x,xs) => {
                        self.read.write(trans, (*xs).clone())?;
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
    fn test_channel_push_pop() {
        let queue = Queue::new();
        let x = atomically(|trans| {
            queue.push(trans, 42)?;
            queue.pop(trans)
        });
        assert_eq!(42, x);
    }
    #[test]
    fn test_channel_order() {
        let queue = Queue::new();
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
    fn test_channel_multi_transactions() {
        let queue = Queue::new();
        let queue2 = queue.clone();

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
    fn test_channel_threaded() {
        use std::thread;
        use std::time::Duration;
        let queue = Queue::new();

        for i in 0..10 {
            let queue2 = queue.clone();
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

