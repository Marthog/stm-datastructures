use stm::*;
use std::sync::Arc;
use std::any::Any;
use super::queue::Queue;

#[derive(Clone)]
pub struct BoundedQueue<T> {
    queue: Queue<T>,
    len: TVar<usize>,
    max_len: usize,
}


/// A threadsafe stack using transactional memory.
impl<T: Any+Sync+Clone+Send> BoundedQueue<T> {
    pub fn new(max_len: usize) -> BoundedQueue<T> {
        BoundedQueue {
            queue: Queue::new(),
            len: TVar::new(0),
            max_len: max_len,
        }
    }

    pub fn push(&self, trans: &mut Transaction, val: T) -> StmResult<()> {
        let len = self.len.read(trans)?;
        if len>=self.max_len {
            retry()?;
        }
        self.len.write(trans, len+1)?;
        self.queue.push(trans, val)
    }

    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        let x = self.queue.pop(trans)?;
        // Order matters: decreasing len first reduces the number of values.
        let len = self.len.read(trans)?;
        self.len.write(trans, len-1)?;
        Ok(x)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use stm::*;

    #[test]
    fn test_bqueue_push_pop() {
        let mut queue = BoundedQueue::new(1);
        let x = atomically(|trans| {
            queue.push(trans, 42)?;
            queue.pop(trans)
        });
        assert_eq!(42, x);
    }

    #[test]
    fn test_bqueue_order() {
        let mut queue = BoundedQueue::new(3);
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
    fn test_bqueue_multi_transactions() {
        let mut queue = BoundedQueue::new(3);
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
    fn test_bqueue_threaded() {
        use std::thread;
        use std::time::Duration;
        let mut queue = BoundedQueue::new(10);

        for i in 0..10 {
            let mut queue2 = queue.clone();
            thread::spawn(move || {
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

    /// Just like `test_bqueue_threaded`, but the
    /// queue is too short to hold all elements simultaneously.
    ///
    /// The threads must push the elments one after another.
    /// The main thread has to block multiple times while querying 
    /// all elements.
    #[test]
    fn test_bqueue_threaded_short_queue() {
        use std::thread;
        use std::time::Duration;
        let mut queue = BoundedQueue::new(2);

        for i in 0..10 {
            let mut queue2 = queue.clone();
            thread::spawn(move || {
                atomically(|trans| 
                    queue2.push(trans, i)
                );
            });
        }

        let mut v = Vec::new();
        for i in 0..10 {
            v.push(atomically(|trans| queue.pop(trans)));
        }

        v.sort();
        for i in 0..10 {
            assert_eq!(v[i],i);
        }
    }
}

