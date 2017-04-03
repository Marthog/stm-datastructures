//#![feature(test)]

extern crate stm;

mod arclist;
pub mod queue;
pub mod bounded_queue;
pub mod semaphore;

//#[cfg(test)]
//mod bench;

use stm::*;

pub use queue::Queue;
pub use bounded_queue::BoundedQueue;
pub use semaphore::Semaphore;


/// Unwrap `Option` or call retry.
pub fn unwrap_or_retry<T>(option: Option<T>) 
    -> StmResult<T> {
    match option {
        Some(x) => Ok(x),
        None    => retry()
    }
}

/// Retry until a the condition holds.
pub fn guard(cond: bool) -> StmResult<()> {
    if cond {
        Ok(())
    } else {
        retry()
    }
}

/// Optionally run a STM action. If `f` fails with a `retry()`, it does 
/// not cancel the whole transaction, but returns 
pub fn optionally<T,F>(trans: &mut Transaction, f: F) -> StmResult<Option<T>>
    where F: Fn(&mut Transaction) -> StmResult<T>
{
    trans.or( 
        |t| f(t).map(|x| Some(x)),
        |_| Ok(None))

}

#[cfg(test)]
mod test {
    use stm::*;
    use super::*;

    #[test]
    fn unwrap_some() {
        let x = Some(42);
        let y = atomically(|_| unwrap_or_retry(x));
        assert_eq!(y, 42);
    }

    #[test]
    fn unwrap_none() {
        let x: Option<i32> = None;
        assert_eq!(unwrap_or_retry(x), retry());
    }

    #[test]
    fn guard_true() {
        let x = guard(true);
        assert_eq!(x, Ok(()));
    }

    #[test]
    fn guard_false() {
        let x = guard(false);
        assert_eq!(x, retry());
    }

    #[test]
    fn optionally_succeed() {
        let x = atomically(|t| 
            optionally(t, |_| Ok(42)));
        assert_eq!(x, Some(42));
    }

    #[test]
    fn optionally_fail() {
        let x:Option<i32> = atomically(|t| 
            optionally(t, |_| retry()));
        assert_eq!(x, None);
    }
}
