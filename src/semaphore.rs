use stm::*;

/// `Semaphore` is an implementation of semaphores on top of software transactional 
/// memory.
///
/// This is a very simple datastructure and serves as a simple thread 
/// synchronization primitive.
#[derive(Clone)]
pub struct Semaphore {
    /// Semaphores are internally just a number.
    num: TVar<u32>
}

impl Semaphore {
    /// Create a new semaphore with `n` initial tokens.
    pub fn new(n: u32) -> Semaphore {
        Semaphore {
            num: TVar::new(n)
        }
    }

    /// Take a token from the semaphore and if none left,
    /// wait for it.
    pub fn wait(&self, trans: &mut Transaction) -> StmResult<()> {
        let n = self.num.read(trans)?;
        if n==0 {
            retry()?;
        }
        self.num.write(trans, n-1)
    }

    /// Free a token.
    pub fn signal(&self, trans: &mut Transaction) -> StmResult<()> {
        self.num.modify(trans, |n| n+1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stm::*;

    // Test if wait with start value of 1 works.
    #[test]
    fn sem_wait() {
        let mut sem = Semaphore::new(1);
        atomically(|trans|
            sem.wait(trans)
        );
    }

    #[test]
    fn sem_signal_wait() {
        let mut sem = Semaphore::new(0);
        atomically(|trans| {
            sem.signal(trans);
            sem.wait(trans)
        });
    }

    #[test]
    fn sem_threaded() {
        use std::thread;
        use std::time::Duration;

        let sem = Semaphore::new(0);
        let sem2 = sem.clone();
        
        thread::spawn(move || {
            for i in 0..10 {
                atomically(|trans| 
                    sem2.signal(trans)
                );
            }
        });

        for i in 0..10 {
            atomically(|trans| {
                sem.wait(trans)
            });
        }
    }

    #[test]
    fn sem_threaded2() {
        use std::thread;
        use std::time::Duration;

        let sem = Semaphore::new(0);
        
        for i in 0..10 {
            let sem2 = sem.clone();
            thread::spawn(move || {
                atomically(|trans| 
                    sem2.signal(trans)
                );
            });
        }

        for i in 0..10 {
            atomically(|trans| {
                sem.wait(trans)
            });
        }
    }
}

