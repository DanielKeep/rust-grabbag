use std::cmp::min;

pub trait RoundRobinIterator: Iterator + Sized {
    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  It stops as soon as either iterator is exhausted.
    */
    fn round_robin<OtherIt: Iterator<Item=<Self as Iterator>::Item>>(self, other_it: OtherIt) -> RoundRobin<Self, OtherIt> {
        RoundRobin {
            it0: self,
            it1: other_it,
            phase: 0,
        }
    }

    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  If one iterator stops before the other, it is simply skipped.
    */
    fn round_robin_longest<OtherIt: Iterator<Item=<Self as Iterator>::Item>>(self, other_it: OtherIt) -> RoundRobinLongest<Self, OtherIt> {
        RoundRobinLongest {
            it0: self,
            it1: other_it,
            phase: 0,
            fused: false,
        }
    }
}

impl<It> RoundRobinIterator for It where It: Iterator {}

#[derive(Clone, Show)]
pub struct RoundRobin<It0, It1> {
    it0: It0,
    it1: It1,
    phase: u8,
}

impl<It0, It1> RoundRobin<It0, It1> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (It0, It1) {
        let RoundRobin { it0, it1, .. } = self;
        (it0, it1)
    }
}

impl<It0, It1, E> Iterator for RoundRobin<It0, It1> where It0: Iterator<Item=E>, It1: Iterator<Item=E> {
    type Item = E;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        match (self.it0.size_hint(), self.it1.size_hint()) {
            ((l0, None), (l1, _)) | ((l0, _), (l1, None)) => (l0+l1, None),
            ((l0, Some(u0)), (l1, Some(u1))) => (l0+l1, Some(2*min(u0, u1)))
        }
    }
}

#[derive(Clone, Show)]
pub struct RoundRobinLongest<It0, It1> {
    it0: It0,
    it1: It1,
    phase: u8,
    fused: bool,
}

impl<It0, It1> RoundRobinLongest<It0, It1> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (It0, It1) {
        let RoundRobinLongest { it0, it1, .. } = self;
        (it0, it1)
    }
}

impl<It0, It1, E> Iterator for RoundRobinLongest<It0, It1> where It0: Iterator<Item=E>, It1: Iterator<Item=E> {
    type Item = E;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        match (self.it0.size_hint(), self.it1.size_hint()) {
            ((l0, None), (l1, _)) | ((l0, _), (l1, None)) => (l0+l1, None),
            ((l0, Some(u0)), (l1, Some(u1))) => (l0+l1, Some(u0+u1))
        }
    }
}

#[test]
fn test_round_robin() {
    let v0 = vec![0us, 2, 4];
    let v1 = vec![1us, 3, 5, 7];
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
    let v0 = vec![0us, 2, 4];
    let v1 = vec![1us, 3, 5, 7];
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
