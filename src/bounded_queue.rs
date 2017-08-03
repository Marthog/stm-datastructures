use stm::*;
use std::any::Any;
use super::Queue;

/// `Queue` is a threadsafe FIFO queue, that uses software transactional memory.
///
/// It is similar to synchronous channels, but undoes operations in case of aborted transactions.
///
///
/// # Example
///
/// ```
/// extern crate stm;
/// extern crate stm_datastructures;
///
/// use stm::*;
/// use ::stm_datastructures::BoundedQueue;
///
/// fn main() {
/// let queue = BoundedQueue::new(10);
/// let x = atomically(|trans| {
///     queue.push(trans, 42)?;
///     queue.pop(trans)
/// });
/// assert_eq!(x, 42);
/// }
/// ```
#[derive(Clone)]
pub struct BoundedQueue<T> {
    /// Internally use a normal queue.
    queue: Queue<T>,

    /// `cap` stores the number of elements, that may still
    /// fit into this queue.
    cap: TVar<usize>,
}


impl<T: Any + Sync + Clone + Send> BoundedQueue<T> {
    /// Create new `BoundedQueue`, that can hold maximally
    /// `capacity` elements.
    pub fn new(capacity: usize) -> BoundedQueue<T> {
        BoundedQueue {
            queue: Queue::new(),
            cap: TVar::new(capacity),
        }
    }

    /// Add a new element to the queue.
    pub fn push(&self, trans: &mut Transaction, val: T) -> StmResult<()> {
        let cap = self.cap.read(trans)?;
        guard(cap > 0)?;
        self.cap.write(trans, cap - 1)?;
        self.queue.push(trans, val)
    }

    /// Push a value to the front of the queue. Next call to `pop` will return `value`.
    ///
    /// `push_front` allows to undo pop-operations and operates the queue in a LIFO way.
    pub fn push_front(&self, trans: &mut Transaction, value: T) -> StmResult<()> {
        let cap = self.cap.read(trans)?;
        guard(cap > 0)?;
        self.cap.write(trans, cap - 1)?;
        self.queue.push_front(trans, value)
    }

    /// Return the first element without removing it.
    pub fn try_peek(&self, trans: &mut Transaction) -> StmResult<Option<T>> {
        self.queue.try_peek(trans)
    }

    /// Return the first element without removing it.
    pub fn peek(&self, trans: &mut Transaction) -> StmResult<T> {
        self.queue.peek(trans)
    }

    /// Remove an element from the queue.
    pub fn try_pop(&self, trans: &mut Transaction) -> StmResult<Option<T>> {
        let v = self.queue.try_pop(trans)?;
        if v.is_some() {
            self.cap.modify(trans, |x| x + 1)?;
        }
        Ok(v)
    }

    /// Remove an element from the queue.
    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        self.cap.modify(trans, |x| x + 1)?;
        self.queue.pop(trans)
    }

    /// Check if a queue is empty.
    pub fn is_empty(&self, trans: &mut Transaction) -> StmResult<bool> {
        self.queue.is_empty(trans)
    }

    /// Check if a queue is full.
    pub fn is_full(&self, trans: &mut Transaction) -> StmResult<bool> {
        let cap = self.cap.read(trans)?;
        Ok(cap == 0)
    }
}


#[cfg(test)]
mod tests {
    use stm::*;
    use super::*;

    #[test]
    fn bqueue_push_pop() {
        let queue = BoundedQueue::new(1);
        let x = atomically(|trans| {
            queue.push(trans, 42)?;
            queue.pop(trans)
        });
        assert_eq!(42, x);
    }

    #[test]
    fn bqueue_order() {
        let queue = BoundedQueue::new(3);
        let x = atomically(|trans| {
            queue.push(trans, 1)?;
            queue.push(trans, 2)?;
            queue.push(trans, 3)?;
            let x1 = queue.pop(trans)?;
            let x2 = queue.pop(trans)?;
            let x3 = queue.pop(trans)?;
            Ok((x1, x2, x3))
        });
        assert_eq!((1, 2, 3), x);
    }

    #[test]
    fn bqueue_multi_transactions() {
        let queue = BoundedQueue::new(3);
        let queue2 = queue.clone();

        atomically(|trans| {
            queue2.push(trans, 1)?;
            queue2.push(trans, 2)
        });
        atomically(|trans| queue.push(trans, 3));

        let x = atomically(|trans| {
            let x1 = queue.pop(trans)?;
            let x2 = queue.pop(trans)?;
            let x3 = queue.pop(trans)?;
            Ok((x1, x2, x3))
        });
        assert_eq!((1, 2, 3), x);
    }

    #[test]
    fn bqueue_threaded() {
        use std::thread;
        let queue = BoundedQueue::new(10);

        for i in 0..10 {
            let queue2 = queue.clone();
            thread::spawn(move || { atomically(|trans| queue2.push(trans, i)); });
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
            assert_eq!(v[i], i);
        }
    }

    /// Just like `bqueue_threaded`, but the
    /// queue is too short to hold all elements simultaneously.
    ///
    /// The threads must push the elments one after another.
    /// The main thread has to block multiple times while querying
    /// all elements.
    #[test]
    fn bqueue_threaded_short_queue() {
        use std::thread;
        let queue = BoundedQueue::new(2);

        for i in 0..10 {
            let queue2 = queue.clone();
            thread::spawn(move || { atomically(|trans| queue2.push(trans, i)); });
        }

        let mut v = Vec::new();
        for _ in 0..10 {
            v.push(atomically(|trans| queue.pop(trans)));
        }

        v.sort();
        for i in 0..10 {
            assert_eq!(v[i], i);
        }
    }
}
