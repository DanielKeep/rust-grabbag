use std::iter::RandomAccessIterator;
use std::mem::replace;

pub trait IntersperseIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that yields `inject` between each element of the input iterator.  `inject` will not appear as the first or last element of the resulting iterator.
    */
    fn intersperse(self, inject: E) -> Intersperse<Self, E>;
}

impl<It, E> IntersperseIterator<E> for It where It: Iterator<Item=E> {
    fn intersperse(mut self, inject: E) -> Intersperse<It, E> {
        let look_ahead = self.next();
        Intersperse {
            iter: self,
            look_ahead: look_ahead,
            inject: inject,
            next_is_inject: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Intersperse<It, E> {
    iter: It,
    look_ahead: Option<E>,
    inject: E,
    next_is_inject: bool,
}

impl<It, E> Iterator for Intersperse<It, E> where It: Iterator<Item=E>, E: Clone {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        match self.next_is_inject {
            false => match replace(&mut self.look_ahead, None) {
                None => None,
                Some(e) => {
                    self.look_ahead = self.iter.next();
                    self.next_is_inject = true;
                    Some(e)
                }
            },
            true => match self.look_ahead.is_some() {
                true => {
                    self.next_is_inject = false;
                    Some(self.inject.clone())
                },
                false => {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (li, mui) = self.iter.size_hint();
        let l = match li {
            0 => 0,
            n => n + n - 1
        };
        let mu = match mui {
            Some(0) => Some(0),
            Some(n) => Some(n + n - 1),
            None => None
        };
        (l, mu)
    }
}

impl<It, E> RandomAccessIterator for Intersperse<It, E> where It: Iterator<Item=E> + RandomAccessIterator, E: Clone {
    fn indexable(&self) -> usize {
        match (self.look_ahead.is_some(), self.iter.indexable(), self.next_is_inject) {
            (false, _, _    ) => 0,
            (true,  0, false) => 1,
            (true,  0, true ) => 2,
            (true,  1, false) => 3,
            (true,  1, true ) => 4,
            (true,  n, false) => 2 + n + (n - 1),
            (true,  n, true ) => 1 + 2 + n + (n - 1),
        }
    }

    fn idx(&mut self, index: usize) -> Option<E> {
        match (index, self.look_ahead.is_some(), self.iter.indexable(), self.next_is_inject, index & 1) {
            (_,     false, _, _,     _) => None,

            (0,     true,  _, false, _) => self.look_ahead.clone(),
            (1,     true,  0, false, _) => None,
            (1,     true,  _, false, _) => Some(self.inject.clone()),
            (i,     true,  _, false, 0) => self.iter.idx(i / 2 - 1),
            (i,     true,  _, false, _) => if i < self.indexable() { Some(self.inject.clone()) } else { None },

            (0,     true,  _, true,  _) => Some(self.inject.clone()),
            (1,     true,  _, true,  _) => self.look_ahead.clone(),
            (i,     true,  _, true,  0) => if i < self.indexable() { Some(self.inject.clone()) } else { None },
            (i,     true,  _, true,  _) => self.iter.idx((i - 1) / 2 - 1),
        }
    }
}

#[test]
fn test_intersperse() {
    use super::CloneEachIterator;

    let v: Vec<&str> = vec![];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, Vec::<&str>::new());

    let v = vec!["a"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a"]);

    let v = vec!["a", "b"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a", ",", "b"]);

    let v = vec!["a", "b", "c"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a", ",", "b", ",", "c"]);

    let v = vec!["a", "b", "c"];
    let mut r = v.iter().clone_each().intersperse(",");
    assert_eq!(r.idx(0), Some("a"));
    assert_eq!(r.idx(1), Some(","));
    assert_eq!(r.idx(2), Some("b"));
    assert_eq!(r.idx(3), Some(","));
    assert_eq!(r.idx(4), Some("c"));
    assert_eq!(r.idx(5), None);
    assert_eq!(r.next(), Some("a"));
    assert_eq!(r.idx(0), Some(","));
    assert_eq!(r.idx(1), Some("b"));
    assert_eq!(r.idx(2), Some(","));
    assert_eq!(r.idx(3), Some("c"));
    assert_eq!(r.idx(4), None);
    assert_eq!(r.next(), Some(","));
    assert_eq!(r.idx(0), Some("b"));
    assert_eq!(r.idx(1), Some(","));
    assert_eq!(r.idx(2), Some("c"));
    assert_eq!(r.idx(3), None);
    assert_eq!(r.next(), Some("b"));
    assert_eq!(r.idx(0), Some(","));
    assert_eq!(r.idx(1), Some("c"));
    assert_eq!(r.idx(2), None);
    assert_eq!(r.next(), Some(","));
    assert_eq!(r.idx(0), Some("c"));
    assert_eq!(r.idx(1), None);
    assert_eq!(r.next(), Some("c"));
    assert_eq!(r.next(), None);
    assert_eq!(r.idx(0), None);
    assert_eq!(r.idx(1), None);
}
