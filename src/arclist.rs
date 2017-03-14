use std::sync::Arc;
use std::mem;

#[derive(Clone)]
pub enum ArcList<T> {
    Elem(T, Arc<ArcList<T>>),
    End
}

impl<T:Clone> ArcList<T> {
    pub fn reverse(&self) -> Self {
        let mut new_list = End;
        let mut ls = self.clone();
        while let Elem(x, xs) = ls {
            new_list = new_list.prepend(x);
            ls = (*xs).clone();
        }
        new_list
    }

    pub fn prepend(self, t: T) -> Self {
        Elem(t, Arc::new(self))
    }

    pub fn head(self) -> Option<T> {
        match self {
            Elem(x,_) => Some(x),
            _           => None,
        }
    }
}

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

