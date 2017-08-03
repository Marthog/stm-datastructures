use std::sync::Arc;
use std::mem;

#[derive(Clone, Debug)]
enum Prim<T> {
    Elem(T, Arc<Prim<T>>),
    End,
}
use self::Prim::*;

impl<T> Prim<T> {
    pub fn prepend(self, t: T) -> Self {
        Elem(t, Arc::new(self))
    }

    #[allow(dead_code)]
    #[inline]
    /// Destroy the whole list.
    /// 
    /// Using `Drop` would break the ability to do destructive pattern
    /// matches on the constructor.
    pub fn destroy(self) {
        let mut slf = self;
        while let Elem(_,tail) = slf {
            slf = match Arc::try_unwrap(tail) {
                Ok(t)   => t,
                Err(_)  => return ()
            }
        }
    }
}

impl<T: Clone> Prim<T> {
    pub fn reverse(&self) -> Self {
        let mut new_list = End;
        let mut ls = self.clone();
        while let Elem(x, xs) = ls {
            new_list = new_list.prepend(x);
            ls = (*xs).clone();
        }
        new_list
    }
}

impl<T> Drop for ArcList<T> {
    fn drop(&mut self) {
        use std::mem;
        let slf = mem::replace(&mut self.prim, End);
        slf.destroy();
    }
}


#[derive(Clone, Debug)]
pub struct ArcList<T> {
    prim: Prim<T>
}

impl<T> ArcList<T> {
    pub fn new() -> Self {
        ArcList{ prim: End }
    }

    fn from_prim(prim: Prim<T>) -> Self {
        ArcList{ prim }
    }

    fn take_prim(&mut self) -> Prim<T> {
        mem::replace(&mut self.prim, End)
    }

    fn map<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Prim<T>) -> Prim<T>
    {
        ArcList{ prim: f(self.take_prim()) }
    }

    pub fn prepend(self, t: T) -> Self {
        self.map(|s| s.prepend(t))
    }

    pub fn is_empty(&self) -> bool {
        match &self.prim {
            &Elem(_, _) => false,
            &End => true,
        }
    }

    #[allow(dead_code)]
    pub fn head(&self) -> Option<&T> {
        match &self.prim {
            &Elem(ref x, _) => Some(x),
            &End            => None
        }
    }
}

impl<T: Clone> ArcList<T> {
    pub fn reverse(self) -> Self {
        self.map(|s| s.reverse())
    }

    pub fn split(mut self) -> Option<(T, ArcList<T>)> {
        match self.take_prim() {
            Elem(x,xs)  => Some((x, ArcList::from_prim((*xs).clone()))),
            End         => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arclist_prepend() {
        let list = ArcList::new().prepend(1).prepend(2).prepend(3);

        assert_eq!(Some(&3), list.head());
    }

    /// Test if the destructor runs correctly.
    /// The naive implementation of linked lists creates stack overflows.
    #[test]
    fn test_long_list() {
        use std::mem;
        let mut list = ArcList::new();
        for i in 0..100000 {
            list = list.prepend(i);
        }
    }


    #[test]
    fn test_arclist_reverse() {
        let list = ArcList::new().prepend(1).prepend(2).prepend(3).reverse();

        assert_eq!(Some(&1), list.head());
    }
}
