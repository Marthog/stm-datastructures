use stm::*;
use std::sync::Arc;
use std::any::Any;
use super::arclist::*;

#[derive(Clone)]
pub struct Stack<T> {
    stack: TVar<ArcList<T>>,
}

/// A threadsafe stack using transactional memory.
impl<T: Any+Sync+Clone+Send> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            stack: TVar::new(End),
        }
    }

    pub fn push(&self, trans: &mut Transaction, value: T) -> StmResult<()> {
        self.stack.modify(trans, |s| Elem(value, Arc::new(s)))
    }

    pub fn pop(&self, trans: &mut Transaction) -> StmResult<T> {
        match self.stack.read(trans)? {
            End     =>  retry()?,
            Elem(x, xs)     => {
                self.stack.write(trans, (*xs).clone())?;
                Ok(x)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_push_pop() {
        let stack = Stack::new();
        let x = atomically(|trans| {
            stack.push(trans, 42)?;
            stack.pop(trans)
        });
        assert_eq!(42, x);
    }

    #[test]
    fn test_stack_order() {
        let stack = Stack::new();
        let x = atomically(|trans| {
            stack.push(trans, 1)?;
            stack.push(trans, 2)?;
            stack.push(trans, 3)?;
            let x1 = stack.pop(trans)?;
            let x2 = stack.pop(trans)?;
            let x3 = stack.pop(trans)?;
            Ok((x1,x2,x3))
        });
        assert_eq!((3,2,1), x);
    }

    #[test]
    fn test_stack_multi_transactions() {
        let stack = Stack::new();
        let stack2 = stack.clone();

        atomically(|trans| {
            stack2.push(trans, 1)?;
            stack2.push(trans, 2)
        });
        atomically(|trans| {
            stack.push(trans, 3)
        });

        let x = atomically(|trans| {
            let x1 = stack.pop(trans)?;
            let x2 = stack.pop(trans)?;
            let x3 = stack.pop(trans)?;
            Ok((x1,x2,x3))
        });
        assert_eq!((3,2,1), x);
    }

    #[test]
    fn test_stack_threaded() {
        use std::thread;
        use std::time::Duration;
        let stack = Stack::new();

        for i in 0..10 {
            let stack2 = stack.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(20));
                atomically(|trans| 
                    stack2.push(trans, i)
                );
            });
        }

        let mut v = atomically(|trans| {
            let mut v = Vec::new();
            for i in 0..10 {
                v.push(stack.pop(trans)?);
            }
            Ok(v)
        });

        v.sort();
        for i in 0..10 {
            assert_eq!(v[i],i);
        }
    }
}

