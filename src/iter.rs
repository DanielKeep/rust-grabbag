use std::cmp::{max, min};
use std::num::Bounded;

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

pub trait IteratorRadialWalk<E, It> where It: RandomAccessIterator<E> {
    /**
Creates an iterator that performs a radial walk of the input iterator.

For example:

```
# use grabbag::iter::{IteratorCloneEach, IteratorRadialWalk};
let v: Vec<uint> = vec![0, 1, 2, 3, 4];

let w0: Vec<_> = v.iter().radial_walk(0).clone_each().collect();
let w1: Vec<_> = v.iter().radial_walk(1).clone_each().collect();
let w2: Vec<_> = v.iter().radial_walk(2).clone_each().collect();
let w3: Vec<_> = v.iter().radial_walk(3).clone_each().collect();
let w4: Vec<_> = v.iter().radial_walk(4).clone_each().collect();

assert_eq!(w0, vec![0, 1, 2, 3, 4]);
assert_eq!(w1, vec![1, 2, 0, 3, 4]);
assert_eq!(w2, vec![2, 3, 1, 4, 0]);
assert_eq!(w3, vec![3, 4, 2, 1, 0]);
assert_eq!(w4, vec![4, 3, 2, 1, 0]);
```
    */
    fn radial_walk(self, start_at: uint) -> RadialWalkItems<It>;
}

impl<E, It> IteratorRadialWalk<E, It> for It where It: RandomAccessIterator<E> {
    fn radial_walk(self, start_at: uint) -> RadialWalkItems<It> {
        RadialWalkItems {
            iter: self,
            start_at: start_at,
            pos: 0,
        }
    }
}

pub struct RadialWalkItems<It> {
    iter: It,
    start_at: uint,
    pos: uint,
}

impl<E, It> Iterator<E> for RadialWalkItems<It> where E: ::std::fmt::Show, It: RandomAccessIterator<E> {
    fn next(&mut self) -> Option<E> {
        // Figure out if we need to stop.  This isn't immediately obvious, due to the way we handle the spiralling.
        let uint_max: uint = Bounded::max_value();
        let iter_len = self.iter.indexable();
        let left_len = iter_len - self.start_at;
        let stop_pos = (
            2.checked_mul(&max(left_len, iter_len - left_len)).and_then(|l| l.checked_add(&1))
        ).unwrap_or(uint_max);
        if self.pos >= stop_pos { return None }

        // Gives us 0 (jitter left) or 1 (jitter right).
        let jitter_right: uint = self.pos & 1;

        // Gives us the magnitude of the jitter.
        let mag = self.pos.checked_add(&1).map(|l| l / 2);

        // If `mag` has overflowed, it's because `self.pos == uint::MAX`.  However, we know the answer to this...
        let mag: uint = mag.unwrap_or((uint_max / 2) + 1);

        // We can now compute the actual index into `iter` we want to use.  Of course, this may very well be out of bounds!
        // This could *possibly* be improved by computing when we need to stop doing the radial walk and just continue with a linear one instead.  For now, I'm just going to skip this position.

        if jitter_right == 0 && mag > self.start_at {
            return match self.pos.checked_add(&1) {
                None => None,
                Some(pos) => {
                    self.pos = pos;
                    self.next()
                }
            }
        }

        if jitter_right > 0 && mag >= self.iter.indexable() - self.start_at {
            return match self.pos.checked_add(&1) {
                None => None,
                Some(pos) => {
                    self.pos = pos;
                    self.next()
                }
            }
        }

        let idx = match jitter_right { 0 => self.start_at - mag, _ => self.start_at + mag };
        match self.iter.idx(idx) {
            None => None,
            e @ _ => {
                match self.pos.checked_add(&1) {
                    // Sadly, we can't represent the next position, so we're kinda stuck.
                    None => None,
                    Some(pos) => {
                        self.pos = pos;
                        e
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

#[test]
fn test_radial_walk() {
    let v: Vec<uint> = vec![0, 1, 2, 3, 4];

    let w0: Vec<_> = v.iter().radial_walk(0).clone_each().collect();
    let w1: Vec<_> = v.iter().radial_walk(1).clone_each().collect();
    let w2: Vec<_> = v.iter().radial_walk(2).clone_each().collect();
    let w3: Vec<_> = v.iter().radial_walk(3).clone_each().collect();
    let w4: Vec<_> = v.iter().radial_walk(4).clone_each().collect();

    assert_eq!(w0, vec![0, 1, 2, 3, 4]);
    assert_eq!(w1, vec![1, 2, 0, 3, 4]);
    assert_eq!(w2, vec![2, 3, 1, 4, 0]);
    assert_eq!(w3, vec![3, 4, 2, 1, 0]);
    assert_eq!(w4, vec![4, 3, 2, 1, 0]);
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

pub trait IteratorSorted<E> where E: Ord {
    /**
Creates an iterator that yields the elements of the input iterator in sorted order.
    */
    fn sorted(self) -> Vec<E>;
}

impl<E, It> IteratorSorted<E> for It where E: Ord, It: Iterator<E> {
    fn sorted(mut self) -> Vec<E> {
        let mut v = self.collect::<Vec<_>>();
        v.sort();
        v
    }
}

#[test]
fn test_sorted() {
    let v = vec![1u, 3, 2, 0, 4];
    let s = v.into_iter().sorted();
    assert_eq!(s, vec![0u, 1, 2, 3, 4]);
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
