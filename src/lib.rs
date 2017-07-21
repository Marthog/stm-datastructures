#![feature(test)]

extern crate stm;

mod arclist;
pub mod queue;
pub mod bounded_queue;
pub mod semaphore;

#[cfg(test)]
mod bench;

use stm::*;

pub use queue::Queue;
pub use bounded_queue::BoundedQueue;
pub use semaphore::Semaphore;

#[inline]
/// Unwrap `Option` or call retry if it is `None`.
///
/// # Example
///
/// ```
/// # extern crate stm;
/// # extern crate stm_datastructures;
/// use stm::*;
/// # use stm_datastructures::*;
///
/// # fn main() {
/// let x = atomically(|_tx|
///     unwrap_or_retry(Some(42))
/// );
/// assert_eq!(x, 42);
/// # }
/// ```
///
/// Very likely to be merged into stm library.
pub fn unwrap_or_retry<T>(option: Option<T>) 
    -> StmResult<T> {
    match option {
        Some(x) => Ok(x),
        None    => retry()
    }
}

#[inline]
/// Retry until a the condition holds.
///
/// # Example
///
/// ```
/// # extern crate stm;
/// # extern crate stm_datastructures;
/// use stm::*;
/// # use stm_datastructures::*;
///
/// # fn main() {
/// let x = atomically(|_tx| {
///     guard(true)?; // guard(true) always succeeds.
///     Ok(42)
/// });
/// assert_eq!(x, 42);
/// # }
/// ```
///
/// Very likely to be merged into stm library.
pub fn guard(cond: bool) -> StmResult<()> {
    if cond {
        Ok(())
    } else {
        retry()
    }
}

#[inline]
/// Optionally run a STM action. If `f` fails with a `retry()`, it does 
/// not cancel the whole transaction, but returns `None`.
///
/// # Example
///
/// ```
/// # extern crate stm;
/// # extern crate stm_datastructures;
/// use stm::*;
/// # use stm_datastructures::*;
///
/// # fn main() {
/// let x:Option<i32> = atomically(|tx| 
///     optionally(tx, |_| retry()));
/// assert_eq!(x, None);
/// # }
/// ```
///
/// Very likely to be merged into stm library.
pub fn optionally<T,F>(tx: &mut Transaction, f: F) -> StmResult<Option<T>>
    where F: Fn(&mut Transaction) -> StmResult<T>
{
    tx.or( 
        |t| f(t).map(Some),
        |_| Ok(None)
    )
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
