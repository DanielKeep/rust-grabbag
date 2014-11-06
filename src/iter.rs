use std::cmp::min;

pub trait IteratorCloneEach<'a, E, It> where E: Clone, It: Iterator<&'a E> {
    /**
Creates an iterator which will clone each element of the input iterator.
    */
    fn clone_each(self) -> CloneItems<It>;
}

impl<'a, E, It> IteratorCloneEach<'a, E, It> for It where E: Clone, It: Iterator<&'a E> {
    fn clone_each(self) -> CloneItems<It> {
        CloneItems {
            iter: self,
        }
    }
}

pub struct CloneItems<It> {
    iter: It,
}

impl<'a, E, It> Iterator<E> for CloneItems<It> where E: 'a+Clone, It: Iterator<&'a E> {
    fn next(&mut self) -> Option<E> {
        match self.iter.next() {
            None => None,
            Some(e) => Some(e.clone())
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

#[test]
fn test_clone_each() {
    let it: Vec<int> = vec![1, 2, 3];
    let mut it = it.iter().clone_each();
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), None);
}

pub trait IteratorFoldl<E> {
    /**
Folds the elements of the iterator together, from left to right, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldl(self, f: |E, E| -> E) -> Option<E>;
}

impl<It, E> IteratorFoldl<E> for It where It: Iterator<E> {
    fn foldl(mut self, f: |E, E| -> E) -> Option<E> {
        let first = match self.next() {
            None => return None,
            Some(e) => e
        };

        Some(self.fold(first, f))
    }
}

#[test]
fn test_foldl() {
    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.into_string());
    assert_eq!(Some("((a, b), c)".into_string()), vs.foldl(|a,b| format!("({}, {})", a, b)));
}

pub trait IteratorFoldr<E> {
    /**
Folds the elements of the iterator together, from right to left, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldr(self, f: |E, E| -> E) -> Option<E>;
}

impl<It, E> IteratorFoldr<E> for It where It: DoubleEndedIterator<E> {
    fn foldr(mut self, f: |E, E| -> E) -> Option<E> {
        let mut last = match self.next_back() {
            None => return None,
            Some(e) => e
        };

        loop {
            match self.next_back() {
                None => break,
                Some(e) => last = f(e, last)
            }
        }

        Some(last)
    }
}

#[test]
fn test_foldr() {
    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.into_string());
    assert_eq!(Some("(a, (b, c))".into_string()), vs.foldr(|a,b| format!("({}, {})", a, b)));
}

pub trait IteratorRoundRobin<E, It1> where It1: Iterator<E> {
    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  It stops as soon as either iterator is exhausted.
    */
    fn round_robin(self, it1: It1) -> RoundRobinItems<E, Self, It1>;

    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  If one iterator stops before the other, it is simply skipped.
    */
    fn round_robin_longest(self, it1: It1) -> RoundRobinLongestItems<E, Self, It1>;
}

impl<E, It0, It1> IteratorRoundRobin<E, It1> for It0 where It0: Iterator<E>, It1: Iterator<E> {
    fn round_robin(self, it1: It1) -> RoundRobinItems<E, It0, It1> {
        RoundRobinItems {
            it0: self,
            it1: it1,
            phase: 0,
        }
    }

    fn round_robin_longest(self, it1: It1) -> RoundRobinLongestItems<E, It0, It1> {
        RoundRobinLongestItems {
            it0: self,
            it1: it1,
            phase: 0,
            fused: false,
        }
    }
}

pub struct RoundRobinItems<E, It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
    it0: It0,
    it1: It1,
    phase: u8,
}

impl<E, It0, It1> Iterator<E> for RoundRobinItems<E, It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        match self.phase {
            0 => match self.it0.next() {
                None => None,
                e @ _ => {
                    self.phase = 1;
                    e
                }
            },
            _ => match self.it1.next() {
                None => None,
                e @ _ => {
                    self.phase = 0;
                    e
                }
            },
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        match (self.it0.size_hint(), self.it1.size_hint()) {
            ((l0, None), (l1, _)) | ((l0, _), (l1, None)) => (l0+l1, None),
            ((l0, Some(u0)), (l1, Some(u1))) => (l0+l1, Some(2*min(u0, u1)))
        }
    }
}

pub struct RoundRobinLongestItems<E, It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
    it0: It0,
    it1: It1,
    phase: u8,
    fused: bool,
}

impl<E, It0, It1> Iterator<E> for RoundRobinLongestItems<E, It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        match (self.phase, self.fused) {
            (0, true) => match self.it0.next() {
                None => None,
                e @ _ => e,
            },
            (_, true) => match self.it1.next() {
                None => None,
                e @ _ => e,
            },
            (0, false) => match self.it0.next() {
                None => {
                    self.phase = 1;
                    self.fused = true;
                    self.next()
                },
                e @ _ => {
                    self.phase = 1;
                    e
                },
            },
            (_, false) => match self.it1.next() {
                None => {
                    self.phase = 0;
                    self.fused = true;
                    self.next()
                },
                e @ _ => {
                    self.phase = 0;
                    e
                },
            },
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        match (self.it0.size_hint(), self.it1.size_hint()) {
            ((l0, None), (l1, _)) | ((l0, _), (l1, None)) => (l0+l1, None),
            ((l0, Some(u0)), (l1, Some(u1))) => (l0+l1, Some(u0+u1))
        }
    }
}

#[test]
fn test_round_robin() {
    let v0 = vec![0u, 2, 4];
    let v1 = vec![1u, 3, 5, 7];
    let mut it = v0.into_iter().round_robin(v1.into_iter());
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.next(), None);
}

#[test]
fn test_round_robin_longest() {
    let v0 = vec![0u, 2, 4];
    let v1 = vec![1u, 3, 5, 7];
    let mut it = v0.into_iter().round_robin_longest(v1.into_iter());
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.next(), Some(7));
    assert_eq!(it.next(), None);
}

pub trait IteratorStride<E, It> where It: Iterator<E> {
    /**
Creates an iterator which yields every `n`th element of the input iterator, including the first.
    */
    fn stride(self, n: uint) -> StrideItems<It>;
}

impl<E, It> IteratorStride<E, It> for It where It: Iterator<E> {
    fn stride(self, n: uint) -> StrideItems<It> {
        StrideItems {
            iter: self,
            stride: n
        }
    }
}

pub struct StrideItems<It> {
    iter: It,
    stride: uint,
}

impl<E, It> Iterator<E> for StrideItems<It> where It: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        let v = match self.iter.next() {
            Some(v) => v,
            None => return None
        };

        for _ in range(0, self.stride - 1) {
            match self.iter.next() {
                None => break,
                _ => ()
            }
        }

        Some(v)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        match self.iter.size_hint() {
            (lb, Some(ub)) => ((lb + self.stride - 1) / self.stride,
                                Some((ub + self.stride - 1) / self.stride)),
            (lb, None) => (lb, None)
        }
    }
}

#[test]
fn test_stride() {
    let v = vec![0i, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut it = v.iter().clone_each().stride(2);
    assert_eq!(it.size_hint(), (5, Some(5)));
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), Some(6));
    assert_eq!(it.next(), Some(8));
    assert_eq!(it.next(), None);

    let v = vec![0i, 1, 2, 3, 4, 5];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (2, Some(2)));

    let v = vec![0i, 1, 2, 3, 4, 5, 6];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0i, 1, 2, 3, 4, 5, 6, 7];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0i, 1, 2, 3, 4, 5, 6, 7, 8];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0i, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (4, Some(4)));
}
