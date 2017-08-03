use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum ArcList<T> {
    Elem(T, Arc<ArcList<T>>),
    End,
}

impl<T> ArcList<T> {
    pub fn is_empty(&self) -> bool {
        match self {
            &Elem(_, _) => false,
            &End => true,
        }
    }

    pub fn prepend(self, t: T) -> Self {
        Elem(t, Arc::new(self))
    }

    #[allow(dead_code)]
    pub fn head(self) -> Option<T> {
        match self {
            Elem(x, _) => Some(x),
            _ => None,
        }
    }

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


impl<T: Clone> ArcList<T> {
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

/*
impl<T> Drop for ArcList<T> {
    fn drop(&mut self) {
        self.destroy();
        self = End;
    }
}
*/

pub use self::ArcList::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arclist_prepend() {
        let list = End.prepend(1).prepend(2).prepend(3);

        assert_eq!(Some(3), list.head());
    }

    #[test]
    fn test_arclist_reverse() {
        let list = End.prepend(1).prepend(2).prepend(3).reverse();

        assert_eq!(Some(1), list.head());
    }
}
