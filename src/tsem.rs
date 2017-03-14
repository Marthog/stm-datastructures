use stm::*;

#[derive(Clone)]
pub struct TSem {
    num: TVar<usize>
}

impl TSem {
    pub fn new(n: usize) -> TSem {
        TSem {
            num: TVar::new(n)
        }
    }

    pub fn wait(&self, trans: &mut Transaction) -> StmResult<()> {
        let n = self.num.read(trans)?;
        if n==0 {
            retry()?;
        }
        self.num.write(trans, n-1)
    }

    pub fn signal(&self, trans: &mut Transaction) -> StmResult<()> {
        self.num.modify(trans, |n| n+1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stm::*;

    #[test]
    fn test_sem_wait() {
        let mut sem = TSem::new(1);
        atomically(|trans|
            sem.wait(trans)
        );
    }

    #[test]
    fn test_sem_signal_wait() {
        let mut sem = TSem::new(0);
        atomically(|trans| {
            sem.signal(trans);
            sem.wait(trans)
        });
    }

    #[test]
    fn test_sem_threaded() {
        use std::thread;
        use std::time::Duration;

        let sem = TSem::new(0);
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
    fn test_sem_threaded2() {
        use std::thread;
        use std::time::Duration;

        let sem = TSem::new(0);
        
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

