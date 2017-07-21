use stm::*;
use std::sync::Arc;
use std::any::Any;
use super::arclist::*;

// Queue is implemented using two lists (`read` and `write`).
// `push` writes to the beginning of `write` and `pop` reads from the
// beginning of `read`. If `read` is empty, the reversed list `write` is
// used as a new list. This way all operations are amortized constant time.

/// `Queue` is a threadsafe FIFO queue, that uses software transactional memory.
///
/// It is similar to channels, but undoes operations in case of aborted transactions.
///
///
/// # Example
///
/// ```
/// extern crate stm;
/// extern crate stm_datastructures;
///
/// use stm::*;
/// use stm_datastructures::Queue;
///
/// fn main() {
///     let queue = Queue::new();
///     let x = atomically(|trans| {
///         queue.push(trans, 42)?;
///         queue.pop(trans)
///     });
///     assert_eq!(x, 42);
/// }
/// ```
#[derive(Clone)]
pub struct Queue<T> {
    read: TVar<ArcList<T>>,
    write: TVar<ArcList<T>>,
}

impl<T: Any+Sync+Clone+Send> Queue<T> {
    /// Create a new queue.
    pub fn new() -> Queue<T> {
        Queue {
            read: TVar::new(End),
            write: TVar::new(End),
        }
    }

    /// Add a new element to the queue.
    pub fn push(&self, trans: &mut Transaction, value: T) -> StmResult<()> {
        self.write.modify(trans, |end| 
            Elem(value, Arc::new(end))
        )
    }

    /// Push a value to the front of the queue. Next call to `pop` will return `value`.
    ///
    /// `push_front` allows to undo pop-operations and operates the queue in a LIFO way.
    pub fn push_front(&self, trans: &mut Transaction, value: T) -> StmResult<()> {
        self.read.modify(trans, |end| 
            Elem(value, Arc::new(end))
        )
    }

    /// Return the first element without removing it.
    pub fn try_peek(&self, trans: &mut Transaction) -> StmResult<Option<T>> {
        let v = self.try_pop(trans)?;
        if let Some(ref e) = v {
            self.push_front(trans, e.clone())?;
        }
        Ok(v)
    }

    /// Return the first element without removing it.
    pub fn peek(&self, trans: &mut Transaction) -> StmResult<T> {
        let v = self.pop(trans)?;
        self.push_front(trans, v.clone())?;
        Ok(v)
    }

    /// Remove an element from the queue.
    pub fn try_pop(&self, trans: &mut Transaction) -> StmResult<Option<T>> {
        Ok(match self.read.read(trans)? {
            Elem(x, xs)     => {
                self.read.write(trans, (*xs).clone())?;
                Some(x)
            }
            End             => {
                let write_list = self.write.replace(trans, End)?;
                match write_list.reverse() {
                    End     => None,
                    Elem(x,xs) => {
                        self.read.write(trans, (*xs).clone())?;
                        Some(x)
                    }
                }
            }
        })
    }

    /// Remove an element from the queue.
    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        unwrap_or_retry(self.try_pop(trans)?)
    }

    /// Check if a queue is empty.
    pub fn is_empty(&self, trans: &mut Transaction) -> StmResult<bool> {
        Ok(
            self.read.read(trans)?.is_empty() || 
            self.write.read(trans)?.is_empty()
        )
    }
}


#[cfg(test)]
mod tests {
    use stm::*;
    use super::*;

    #[test]
    fn channel_push_pop() {
        let queue = Queue::new();
        let x = atomically(|trans| {
            queue.push(trans, 42)?;
            queue.pop(trans)
        });
        assert_eq!(42, x);
    }
    #[test]
    fn channel_order() {
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
    fn channel_multi_transactions() {
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
    fn channel_threaded() {
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
            for _ in 0..10 {
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
