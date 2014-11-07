use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::RingBuf;
use std::mem::replace;
use std::num::Bounded;
use std::rc::Rc;

pub trait IteratorAccumulate<E, It> where It: Iterator<E> {
    /**
Creates an iterator that scans from left to right over the input sequence, returning the accumulated result of calling `f` on the entire sequence up to that point.
    */
    fn accumulate(self, f: |E, E| -> E) -> AccumulateItems<E, It>;
}

impl<E, It> IteratorAccumulate<E, It> for It where It: Iterator<E> {
    fn accumulate(self, f: |E, E| -> E) -> AccumulateItems<E, It> {
        AccumulateItems {
            iter: self,
            f: f,
            accum: None,
        }
    }
}

pub struct AccumulateItems<'a, E, It> {
    iter: It,
    f: |E, E|: 'a -> E,
    accum: Option<E>,
}

impl<'a, E, It> Iterator<E> for AccumulateItems<'a, E, It> where E: Clone, It: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        match replace(&mut self.accum, None) {
            None => match self.iter.next() {
                None => None,
                e @ _ => {
                    self.accum = e;
                    self.accum.clone()
                }
            },
            Some(accum) => match self.iter.next() {
                None => {
                    self.accum = None;
                    None
                },
                Some(rhs) => {
                    self.accum = Some((self.f)(accum, rhs));
                    self.accum.clone()
                }
            }
        }
    }
}

#[test]
fn test_accumulate() {
    let v = vec![0u, 1, 2, 3, 4];
    let r: Vec<_> = v.into_iter().accumulate(|a,b| a+b).collect();
    assert_eq!(r, vec![0, 1, 3, 6, 10]);
}

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

#[deriving(Clone, Show)]
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

impl<'a, E, It> RandomAccessIterator<E> for CloneItems<It> where E: 'a+Clone, It: RandomAccessIterator<&'a E> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<E> {
        self.iter.idx(index).map(|e| e.clone())
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

pub trait IteratorGroupBy<E> {
    /**
Creates an iterator that yields a succession of `(group, sub_iterator)` pairs.  Each `sub_iterator` yields successive elements of the input iterator that have the same `group`.  An element's `group` is computed using the `f` closure.

For example:

```
# extern crate grabbag;
# use grabbag::iter::IteratorGroupBy;
# fn main () {
let v = vec![7u, 5, 6, 2, 4, 7, 6, 1, 6, 4, 4, 6, 0, 0, 8, 8, 6, 1, 8, 7];
let is_even: |&uint| -> bool = |&n| if n & 1 == 0 { true } else { false };
for (even, mut ns) in v.into_iter().group_by(is_even) {
    println!("{}...", if even { "Evens" } else { "Odds" });
    for n in ns {
        println!(" - {}", n);
    }
}
# }
```
    */
    fn group_by<'a, G>(self, f: |&E|: 'a -> G) -> GroupByItems<'a, E, Self, G>;
}

impl<E, It> IteratorGroupBy<E> for It where It: Iterator<E> {
    fn group_by<'a, G>(self, f: |&E|: 'a -> G) -> GroupByItems<'a, E, It, G> {
        GroupByItems {
            state: Rc::new(RefCell::new(GroupByItemsShared {
                iter: self,
                group: f,
                last_group: None,
                push_back: None,
            })),
        }
    }
}

pub struct GroupByItems<'a, E, It, G> {
    state: Rc<RefCell<GroupByItemsShared<'a, E, It, G>>>,
}

pub struct GroupByItemsShared<'a, E, It, G> {
    iter: It,
    group: |&E|: 'a -> G,
    last_group: Option<G>,
    push_back: Option<(G, E)>,
}

impl<'a, E, It, G> Iterator<(G, GroupBySubItems<'a, E, It, G>)> for GroupByItems<'a, E, It, G> where It: Iterator<E>, G: Clone+Eq {
    fn next(&mut self) -> Option<(G, GroupBySubItems<'a, E, It, G>)> {
        // First, get a mutable borrow to the underlying state.
        let mut state = self.state.deref().borrow_mut();
        let state = state.deref_mut();

        // If we have a push-back element, immediately construct a sub iterator.
        if let Some((g, e)) = replace(&mut state.push_back, None) {
            return Some((
                g.clone(),
                GroupBySubItems {
                    state: self.state.clone(),
                    group_value: g,
                    first_value: Some(e),
                }
            ));
        }

        // Otherwise, try to pull the next element from the input iterator.
        // Complication: a sub iterator *might* stop *before* the group is exhausted.  We need to account for this (so that users can easily skip groups).
        let (e, g) = match replace(&mut state.last_group, None) {
            None => {
                // We don't *have* a previous group, just grab the next element.
                let e = match state.iter.next() {
                    Some(e) => e,
                    None => return None
                };
                let g = (state.group)(&e);
                (e, g)
            },
            Some(last_g) => {
                // We have to keep pulling elements until the group changes.
                let mut e;
                let mut g;
                loop {
                    e = match state.iter.next() {
                        Some(e) => e,
                        None => return None
                    };
                    g = (state.group)(&e);
                    if g != last_g { break; }
                }
                (e, g)
            }
        };

        // Remember this group.
        state.last_group = Some(g.clone());

        // Construct the sub-iterator and yield it.
        Some((
            g.clone(),
            GroupBySubItems {
                state: self.state.clone(),
                group_value: g,
                first_value: Some(e),
            }
        ))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (lb, mub) = self.state.deref().borrow().iter.size_hint();
        let lb = min(lb, 1);
        (lb, mub)
    }
}

pub struct GroupBySubItems<'a, E, It, G> {
    state: Rc<RefCell<GroupByItemsShared<'a, E, It, G>>>,
    group_value: G,
    first_value: Option<E>,
}

impl<'a, E, It, G> Iterator<E> for GroupBySubItems<'a, E, It, G> where It: Iterator<E>, G: Eq {
    fn next(&mut self) -> Option<E> {
        // If we have a first_value, consume and yield that.
        if let Some(e) = replace(&mut self.first_value, None) {
            return Some(e)
        }

        // Get a mutable borrow to the shared state.
        let mut state = self.state.deref().borrow_mut();
        let state = state.deref_mut();

        let e = match state.iter.next() {
            Some(e) => e,
            None => return None
        };

        let g = (state.group)(&e);

        match g == self.group_value {
            true => {
                // Still in the same group.
                Some(e)
            },
            false => {
                // Different group!  We need to push (g, e) back into the master iterator.
                state.push_back = Some((g, e));
                None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let state = self.state.deref().borrow();

        let lb = if self.first_value.is_some() { 1 } else { 0 };
        let (_, mub) = state.iter.size_hint();
        (lb, mub)
    }
}

#[test]
fn test_group_by() {
    {
        let v = vec![0u, 1, 2, 3, 5, 4, 6, 8, 7];
        let mut oi = v.into_iter().group_by(|&e| e & 1);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 0);
        assert_eq!(ii.next(), Some(0));
        assert_eq!(ii.next(), None);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 1);
        assert_eq!(ii.next(), Some(1));
        assert_eq!(ii.next(), None);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 0);
        assert_eq!(ii.next(), Some(2));
        assert_eq!(ii.next(), None);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 1);
        assert_eq!(ii.next(), Some(3));
        assert_eq!(ii.next(), Some(5));
        assert_eq!(ii.next(), None);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 0);
        assert_eq!(ii.next(), Some(4));
        assert_eq!(ii.next(), Some(6));
        assert_eq!(ii.next(), Some(8));
        assert_eq!(ii.next(), None);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 1);
        assert_eq!(ii.next(), Some(7));
        assert_eq!(ii.next(), None);

        assert!(oi.next().is_none());
    }
    {
        let v = vec![0u, 1, 2, 3, 5, 4, 6, 8, 7];
        let mut oi = v.into_iter().group_by(|&e| e & 1);

        let (g, _) = oi.next().unwrap();
        assert_eq!(g, 0);

        let (g, _) = oi.next().unwrap();
        assert_eq!(g, 1);

        let (g, _) = oi.next().unwrap();
        assert_eq!(g, 0);

        let (g, _) = oi.next().unwrap();
        assert_eq!(g, 1);

        let (g, mut ii) = oi.next().unwrap();
        assert_eq!(g, 0);
        assert_eq!(ii.next(), Some(4));
        assert_eq!(ii.next(), Some(6));

        let (g, _) = oi.next().unwrap();
        assert_eq!(g, 1);

        assert!(oi.next().is_none());
    }
}

pub trait IteratorIndexed<It, IndIt> {
    fn indexed(self, indices: IndIt) -> IndexedItems<It, IndIt>;
}

impl<E, It, IndIt> IteratorIndexed<It, IndIt> for It where It: RandomAccessIterator<E> {
    fn indexed(self, indices: IndIt) -> IndexedItems<It, IndIt> {
        IndexedItems {
            iter: self,
            indices: indices,
        }
    }
}

#[deriving(Clone, Show)]
pub struct IndexedItems<It, IndIt> {
    iter: It,
    indices: IndIt,
}

impl<E, It, IndIt> Iterator<Option<E>> for IndexedItems<It, IndIt> where It: RandomAccessIterator<E>, IndIt: Iterator<uint> {
    fn next(&mut self) -> Option<Option<E>> {
        match self.indices.next() {
            None => None,
            Some(idx) => Some(self.iter.idx(idx))
        }
    }
}

impl<E, It, IndIt> RandomAccessIterator<Option<E>> for IndexedItems<It, IndIt> where It: RandomAccessIterator<E>, IndIt: RandomAccessIterator<uint> {
    fn indexable(&self) -> uint {
        self.indices.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<Option<E>> {
        self.indices.idx(index).and_then(|i| Some(self.iter.idx(i)))
    }
}

#[test]
fn test_indexed() {
    let v = vec![0u, 1, 2, 3, 4];
    let i = vec![2u, 4, 1, 0, 2, 3, 5];
    let r: Vec<_> = v.iter().indexed(i.into_iter()).map(|e| e.map(|v| *v)).collect();
    assert_eq!(r, vec![Some(2), Some(4), Some(1), Some(0), Some(2), Some(3), None])
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

pub trait IteratorPadTailTo<E, It> where It: Iterator<E> {
    /**
Creates an iterator that ensures there are at least `n` elements in a sequence.  If the input iterator is too short, the difference is made up with a filler value.
    */
    fn pad_tail_to<'a>(self, n: uint, filler: |uint|: 'a -> E) -> PadTailToItems<'a, It, E>;
}

impl<E, It> IteratorPadTailTo<E, It> for It where It: Iterator<E> {
    /**
Creates an iterator that ensures there are at least `n` elements in a sequence.  If the input iterator is too short, the difference is made up with a filler value.
    */
    fn pad_tail_to<'a>(self, n: uint, filler: |uint|: 'a -> E) -> PadTailToItems<'a, It, E> {
        PadTailToItems {
            iter: self,
            min: n,
            pos: 0,
            filler: filler,
        }
    }
}

pub struct PadTailToItems<'a, It, E> {
    iter: It,
    min: uint,
    pos: uint,
    filler: |uint|: 'a -> E,
}

impl<'a, E, It> Iterator<E> for PadTailToItems<'a, It, E> where It: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        match self.iter.next() {
            None => {
                if self.pos < self.min {
                    let e = Some((self.filler)(self.pos));
                    self.pos += 1;
                    e
                } else {
                    None
                }
            },
            e @ _ => {
                self.pos += 1;
                e
            }
        }
    }
}

impl<'a, E, It> RandomAccessIterator<E> for PadTailToItems<'a, It, E> where It: RandomAccessIterator<E> {
    fn indexable(&self) -> uint {
        max(self.iter.indexable(), self.min)
    }

    fn idx(&mut self, index: uint) -> Option<E> {
        match (index < self.iter.indexable(), index < self.min) {
            (true, _) => self.iter.idx(index),
            (false, true) => Some((self.filler)(index)),
            _ => None
        }
    }
}

#[test]
fn test_pad_tail_to() {
    let v: Vec<uint> = vec![0, 1, 2];
    let r: Vec<_> = v.into_iter().pad_tail_to(5, |n| n).collect();
    assert_eq!(r, vec![0, 1, 2, 3, 4]);

    let v: Vec<uint> = vec![0, 1, 2];
    let r: Vec<_> = v.into_iter().pad_tail_to(1, |_| panic!()).collect();
    assert_eq!(r, vec![0, 1, 2]);
}

pub trait IteratorPacingWalk<E, It> where It: RandomAccessIterator<E> {
    /**
Creates an iterator that performs a back-and-forth walk of the input iterator.

For example:

```
# use grabbag::iter::{IteratorCloneEach, IteratorPacingWalk};
let v: Vec<uint> = vec![0, 1, 2, 3, 4];

let w0: Vec<_> = v.iter().pacing_walk(0).clone_each().collect();
let w1: Vec<_> = v.iter().pacing_walk(1).clone_each().collect();
let w2: Vec<_> = v.iter().pacing_walk(2).clone_each().collect();
let w3: Vec<_> = v.iter().pacing_walk(3).clone_each().collect();
let w4: Vec<_> = v.iter().pacing_walk(4).clone_each().collect();

assert_eq!(w0, vec![0, 1, 2, 3, 4]);
assert_eq!(w1, vec![1, 2, 0, 3, 4]);
assert_eq!(w2, vec![2, 3, 1, 4, 0]);
assert_eq!(w3, vec![3, 4, 2, 1, 0]);
assert_eq!(w4, vec![4, 3, 2, 1, 0]);
```
    */
    fn pacing_walk(self, start_at: uint) -> PacingWalkItems<It>;
}

impl<E, It> IteratorPacingWalk<E, It> for It where It: RandomAccessIterator<E> {
    fn pacing_walk(self, start_at: uint) -> PacingWalkItems<It> {
        PacingWalkItems {
            iter: self,
            start_at: start_at,
            pos: 0,
        }
    }
}

#[deriving(Clone, Show)]
pub struct PacingWalkItems<It> {
    iter: It,
    start_at: uint,
    pos: uint,
}

impl<E, It> Iterator<E> for PacingWalkItems<It> where E: ::std::fmt::Show, It: RandomAccessIterator<E> {
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
fn test_pacing_walk() {
    let v: Vec<uint> = vec![0, 1, 2, 3, 4];

    let w0: Vec<_> = v.iter().pacing_walk(0).clone_each().collect();
    let w1: Vec<_> = v.iter().pacing_walk(1).clone_each().collect();
    let w2: Vec<_> = v.iter().pacing_walk(2).clone_each().collect();
    let w3: Vec<_> = v.iter().pacing_walk(3).clone_each().collect();
    let w4: Vec<_> = v.iter().pacing_walk(4).clone_each().collect();

    assert_eq!(w0, vec![0, 1, 2, 3, 4]);
    assert_eq!(w1, vec![1, 2, 0, 3, 4]);
    assert_eq!(w2, vec![2, 3, 1, 4, 0]);
    assert_eq!(w3, vec![3, 4, 2, 1, 0]);
    assert_eq!(w4, vec![4, 3, 2, 1, 0]);
}

pub trait IteratorRoundRobin<It1> {
    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  It stops as soon as either iterator is exhausted.
    */
    fn round_robin(self, it1: It1) -> RoundRobinItems<Self, It1>;

    /**
Creates an iterator that alternates between yielding elements of the two input iterators.  If one iterator stops before the other, it is simply skipped.
    */
    fn round_robin_longest(self, it1: It1) -> RoundRobinLongestItems<Self, It1>;
}

impl<E, It0, It1> IteratorRoundRobin<It1> for It0 where It0: Iterator<E> {
    fn round_robin(self, it1: It1) -> RoundRobinItems<It0, It1> {
        RoundRobinItems {
            it0: self,
            it1: it1,
            phase: 0,
        }
    }

    fn round_robin_longest(self, it1: It1) -> RoundRobinLongestItems<It0, It1> {
        RoundRobinLongestItems {
            it0: self,
            it1: it1,
            phase: 0,
            fused: false,
        }
    }
}

#[deriving(Clone, Show)]
pub struct RoundRobinItems<It0, It1> {
    it0: It0,
    it1: It1,
    phase: u8,
}

impl<E, It0, It1> Iterator<E> for RoundRobinItems<It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
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

#[deriving(Clone, Show)]
pub struct RoundRobinLongestItems<It0, It1> {
    it0: It0,
    it1: It1,
    phase: u8,
    fused: bool,
}

impl<E, It0, It1> Iterator<E> for RoundRobinLongestItems<It0, It1> where It0: Iterator<E>, It1: Iterator<E> {
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

#[test]
fn test_skip() {
    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0i, 1];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![]);
}

pub trait IteratorSkipExactly {
    fn skip_exactly(self, n: uint) -> Self;
}

impl<E, It> IteratorSkipExactly for It where It: Iterator<E> {
    fn skip_exactly(mut self, n: uint) -> It {
        for i in range(0, n) {
            match self.next() {
                None => panic!("skip_exactly asked to skip {} elements, but only got {}", n, i),
                _ => ()
            }
        }
        self
    }
}

#[test]
fn test_skip_exactly() {
    use std::task::try;

    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip_exactly(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0i, 1];
    let r: Result<Vec<_>, _> = try(proc() {
        v.into_iter().skip_exactly(3).collect()
    });
    assert!(r.is_err());
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

#[deriving(Clone, Show)]
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

impl<E, It> RandomAccessIterator<E> for StrideItems<It> where It: RandomAccessIterator<E> {
    fn indexable(&self) -> uint {
        (self.iter.indexable() + self.stride - 1) / self.stride
    }

    fn idx(&mut self, index: uint) -> Option<E> {
        index.checked_mul(&self.stride).and_then(|i| self.iter.idx(i))
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

#[test]
fn test_take() {
    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().take(3).collect();
    assert_eq!(r, vec![0, 1, 2]);

    let v = vec![0i, 1];
    let r: Vec<_> = v.into_iter().take(3).collect();
    assert_eq!(r, vec![0, 1]);
}

pub trait IteratorTakeExactly<E, It> where It: Iterator<E> {
    fn take_exactly(self, n: uint) -> TakeExactlyItems<It>;
}

impl<E, It> IteratorTakeExactly<E, It> for It where It: Iterator<E> {
    fn take_exactly(self, n: uint) -> TakeExactlyItems<It> {
        TakeExactlyItems {
            iter: self,
            left: n,
        }
    }
}

#[deriving(Clone, Show)]
pub struct TakeExactlyItems<It> {
    iter: It,
    left: uint,
}

impl<E, It> Iterator<E> for TakeExactlyItems<It> where It: Iterator<E> {
    fn next(&mut self) -> Option<E> {
        match self.left {
            0 => None,
            _ => match self.iter.next() {
                None => panic!("take_exactly expected {} more elements from iterator, but ran out", self.left),
                e @ _ => {
                    self.left -= 1;
                    e
                }
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.left, Some(self.left))
    }
}

impl<E, It> RandomAccessIterator<E> for TakeExactlyItems<It> where It: RandomAccessIterator<E> {
    fn indexable(&self) -> uint {
        self.left
    }

    fn idx(&mut self, index: uint) -> Option<E> {
        if index < self.left {
            match self.iter.idx(index) {
                None => panic!("take_exactly expected {} more elements from iterator"),
                e @ _ => e
            }
        } else {
            None
        }
    }
}

#[test]
fn test_take_exactly() {
    use std::task::try;

    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().take_exactly(3).collect();
    assert_eq!(r, vec![0, 1, 2]);

    let v = vec![0i, 1];
    let r: Result<Vec<_>, _> = try(proc() {
        v.into_iter().take_exactly(3).collect()
    });
    assert!(r.is_err());
}

pub trait IteratorTee<E> {
    /**
Creates a pair of iterators that will yield the same sequence of values.

The element type must implement Clone.
    */
    fn tee(self) -> (TeeItems<E, Self>, TeeItems<E, Self>);
}

impl<E, It> IteratorTee<E> for It where It: Iterator<E> {
    fn tee(self) -> (TeeItems<E, It>, TeeItems<E, It>) {
        let tee0 = TeeItems {
            state: Rc::new(RefCell::new(TeeItemsState {
                iter: self,
                iter_next: 0,
                buffer: RingBuf::new(),
            })),
            this_next: 0,
        };
        let tee1 = TeeItems {
            state: tee0.state.clone(),
            this_next: 0,
        };
        (tee0, tee1)
    }
}

pub struct TeeItems<E, It> {
    /*
        /!\ Important /!\

    In order for this to *not* panic, you need to ensure that nothing is re-entrant whilst it holds a mutable reference to the `RefCell`.  This should still be safe from the user's perspective.
    */
    state: Rc<RefCell<TeeItemsState<E, It>>>,
    this_next: uint,
}

pub struct TeeItemsState<E, It> {
    iter: It,
    iter_next: uint,
    buffer: RingBuf<E>,
}

impl<E, It> Iterator<E> for TeeItems<E, It> where It: Iterator<E>, E: Clone {
    fn next(&mut self) -> Option<E> {
        let mut state = self.state.deref().borrow_mut();
        let state = state.deref_mut();

        match (self.this_next, state.iter_next, state.buffer.len()) {
            // Tee is even with iter.
            (i, j, _) if i == j => {
                match state.iter.next() {
                    None => None,
                    Some(e) => {
                        self.this_next += 1;
                        state.iter_next += 1;
                        state.buffer.push(e.clone());
                        Some(e)
                    }
                }
            },
            // Error: Tee is behind iter, nothing in the buffer.
            (i, j, 0) if i < j => {
                panic!("tee fell behind iterator, but buffer is empty");
            },
            // Tee is behind iter, elements in buffer.
            (i, j, _) if i < j => {
                let e = state.buffer.pop_front().unwrap();
                self.this_next += 1;
                Some(e)
            },
            // Error: Tee is ahead of iter.
            _ /*(i, j, _) if i > j*/ => {
                panic!("tee got ahead of iterator");
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let state = self.state.deref().borrow_mut();
        let state = state.deref();

        match (self.this_next, state.iter_next, state.buffer.len()) {
            // Tee is even with iter.
            (i, j, _) if i == j => {
                state.iter.size_hint()
            },
            // Error: Tee is behind iter, nothing in the buffer.
            (i, j, 0) if i < j => {
                panic!("tee fell behind iterator, but buffer is empty");
            },
            // Tee is behind iter, elements in buffer.
            (i, j, n) if i < j => {
                match state.iter.size_hint() {
                    (lb, None) => (lb+n, None),
                    (lb, Some(ub)) => (lb+n, Some(ub+n))
                }
            },
            // Error: Tee is ahead of iter.
            _ /*(i, j, _) if i > j*/ => {
                panic!("tee got ahead of iterator");
            }
        }
    }
}

#[test]
fn test_tee() {
    let v = vec![0u, 1, 2, 3];
    let (mut a, mut b) = v.into_iter().tee();
    assert_eq!(a.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(b.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

    let v = vec![0u, 1, 2, 3];
    let (mut a, mut b) = v.into_iter().tee();
    assert_eq!(b.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(a.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

    let v = vec![0u, 1, 2, 3];
    let (mut a, mut b) = v.into_iter().tee();
    assert_eq!(a.next(), Some(0));
    assert_eq!(a.next(), Some(1));
    assert_eq!(b.next(), Some(0));
    assert_eq!(b.next(), Some(1));
    assert_eq!(b.next(), Some(2));
    assert_eq!(a.next(), Some(2));
    assert_eq!(b.next(), Some(3));
    assert_eq!(a.next(), Some(3));
    assert_eq!(a.next(), None);
    assert_eq!(b.next(), None);
}

pub trait IteratorZipLongest {
    /**
Creates an iterator which yields elements from both input iterators in lockstep.  If one iterator ends before the other, the elements from that iterator will be replaced with `None`.
    */
    fn zip_longest<It1>(self, it1: It1) -> ZipLongestItems<Self, It1>;
}

impl<E0, It0> IteratorZipLongest for It0 where It0: Iterator<E0> {
    fn zip_longest<It1>(self, it1: It1) -> ZipLongestItems<It0, It1> {
        ZipLongestItems {
            it0: self,
            it1: it1,
        }
    }
}

#[deriving(Clone)]
pub struct ZipLongestItems<It0, It1> {
    it0: It0,
    it1: It1,
}

impl<E0, E1, It0, It1> Iterator<(Option<E0>, Option<E1>)> for ZipLongestItems<It0, It1> where It0: Iterator<E0>, It1: Iterator<E1> {
    fn next(&mut self) -> Option<(Option<E0>, Option<E1>)> {
        match (self.it0.next(), self.it1.next()) {
            (None, None) => None,
            e @ _ => Some(e)
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (l0, mu0) = self.it0.size_hint();
        let (l1, mu1) = self.it1.size_hint();
        let l = max(l0, l1);
        let mu = match (mu0, mu1) {
            (None, _) | (_, None) => None,
            (Some(u0), Some(u1)) => Some(max(u0, u1))
        };
        (l, mu)
    }
}

#[test]
fn test_zip_longest() {
    let a = vec![0u, 1, 2, 3];
    let b = vec!["a", "b", "c"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
        (Some(3), None),
    ]);

    let a = vec![0u, 1, 2];
    let b = vec!["a", "b", "c"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
    ]);

    let a = vec![0u, 1, 2];
    let b = vec!["a", "b", "c", "d"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
        (None, Some("d")),
    ]);
}
