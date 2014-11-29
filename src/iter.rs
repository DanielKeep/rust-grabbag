/**
This module contains a set of iterator extensions.  Rather than being named for the type of iterator they are implemented on, they are named for the method (or group of associated methods) being implemented.

# Standard Features

The iterator extensions, where possible, should implement the following features:

- `Clone`, which produces an *independent* version of the iterator.
- `DoubleEndedIterator`.
- `ExactSizeIterator`.
- `RandomAccessIterator`.
- `Show`.
- Accurate `size_hint` (depending on the transform being performed, and the accuracy of the underlying iterator).
- An `unwrap` method, which returns any owned values passed into the iterator; typically, this is the original iterator.
*/
use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::RingBuf;
use std::mem::replace;
use std::num::Int;
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

impl<'a, E, It> AccumulateItems<'a, E, It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        let AccumulateItems { iter, .. } = self;
        iter
    }
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

pub trait IteratorCartesianProduct<E0, It1> {
    /**
Creates an iterator that yields the cartesian product of two input iterators.

The element type of the first input iterator must implement Clone, as must the second iterator type.
    */
    fn cartesian_product(self, it1: It1) -> CartesianProductItems<E0, Self, It1>;
}

impl<E0, It0, It1> IteratorCartesianProduct<E0, It1> for It0 where E0: Clone, It0: Iterator<E0>, It1: Clone {
    fn cartesian_product(self, it1: It1) -> CartesianProductItems<E0, It0, It1> {
        CartesianProductItems {
            it0: self,
            it1: it1.clone(),
            it1_checkpoint: it1,
            left_value: None,
        }
    }
}

#[deriving(Clone, Show)]
pub struct CartesianProductItems<E0, It0, It1> {
    it0: It0,
    it1: It1,
    it1_checkpoint: It1,
    left_value: Option<E0>,
}

impl<E0, E1, It0, It1> Iterator<(E0, E1)> for CartesianProductItems<E0, It0, It1> where E0: Clone, It0: Iterator<E0>, It1: Clone+Iterator<E1> {
    fn next(&mut self) -> Option<(E0, E1)> {
        loop {
            if let Some(e0) = self.left_value.clone() {
                match self.it1.next() {
                    Some(e1) => return Some((e0, e1)),
                    None => {
                        self.left_value = None;
                        self.it1 = self.it1_checkpoint.clone();
                    }
                }
            }

            match self.it0.next() {
                Some(e0) => {
                    self.left_value = Some(e0);
                }
                None => return None
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (l0, mu0) = self.it0.size_hint();
        let (l1, mu1) = self.it1.size_hint();
        let (lc, muc) = self.it1_checkpoint.size_hint();

        let combine_bounds: |uint, uint, uint| -> uint;

        if self.left_value.is_some() {
            combine_bounds = |b0, b1, bc| b1 + b0*bc;
        } else {
            combine_bounds = |b0, _, bc| b0*bc;
        }

        let l = combine_bounds(l1, l0, lc);
        let mu = match (mu0, mu1, muc) {
            (None, _, _) | (_, None, _) | (_, _, None) => None,
            (Some(u0), Some(u1), Some(uc)) => Some(combine_bounds(u1, u0, uc))
        };

        (l, mu)
    }
}

#[test]
fn test_cartesian_product() {
    let a = vec![0u, 1, 2];
    let b = vec![3u, 4];
    let r: Vec<_> = a.into_iter().cartesian_product(b.iter().clone_each()).collect();
    assert_eq!(r, vec![(0,3),(0,4),(1,3),(1,4),(2,3),(2,4)]);
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

impl<'a, E, It> CloneItems<It> where E: 'a+Clone, It: Iterator<&'a E> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
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

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `GroupByItemsShared` value.
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

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `GroupByItemsShared` value.
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
    /**
Creates an iterator which uses `indices` as an indexer to the subject iterator.
    */
    fn indexed(self, indices: IndIt) -> IndexedItems<It, IndIt>;

    /**
Creates an iterator which uses `indices` as an indexer to the subject iterator, where the subject is behind a mutable reference.
    */
    fn indexed_view(&mut self, indices: IndIt) -> IndexedViewItems<It, IndIt>;
}

impl<E, It, IndIt> IteratorIndexed<It, IndIt> for It where It: RandomAccessIterator<E> {
    fn indexed(self, indices: IndIt) -> IndexedItems<It, IndIt> {
        IndexedItems {
            iter: self,
            indices: indices,
        }
    }

    fn indexed_view(&mut self, indices: IndIt) -> IndexedViewItems<It, IndIt> {
        IndexedViewItems {
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

impl<It, IndIt> IndexedItems<It, IndIt> {
    /**
Unwraps the iterator, returning the underlying indexed and indexer iterators.
    */
    pub fn unwrap(self) -> (It, IndIt) {
        let IndexedItems { iter, indices } = self;
        (iter, indices)
    }
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

#[deriving(Show)]
pub struct IndexedViewItems<'a, It, IndIt> where It: 'a {
    iter: &'a mut It,
    indices: IndIt,
}

impl<'a, It, IndIt> IndexedViewItems<'a, It, IndIt> {
    /**
Unwraps the iterator, returning the underlying indexed and indexer iterators.
    */
    pub fn unwrap(self) -> (&'a mut It, IndIt) {
        let IndexedViewItems { iter, indices } = self;
        (iter, indices)
    }
}

impl<'a, E, It, IndIt> Iterator<Option<E>> for IndexedViewItems<'a, It, IndIt> where It: 'a+RandomAccessIterator<E>, IndIt: Iterator<uint> {
    fn next(&mut self) -> Option<Option<E>> {
        match self.indices.next() {
            None => None,
            Some(idx) => Some(self.iter.idx(idx))
        }
    }
}

impl<'a, E, It, IndIt> RandomAccessIterator<Option<E>> for IndexedViewItems<'a, It, IndIt> where It: 'a+RandomAccessIterator<E>, IndIt: RandomAccessIterator<uint> {
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

#[test]
fn test_indexed_view() {
    let v = vec![0u, 1, 2, 3, 4];
    let mut v = v.iter().clone_each();
    let i = vec![2u, 4, 1, 0, 2, 3, 5];
    let r: Vec<_> = v.indexed_view(i.into_iter()).collect();
    assert_eq!(r, vec![Some(2), Some(4), Some(1), Some(0), Some(2), Some(3), None]);
    assert_eq!(v.collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
}

pub trait IteratorFoldl<E> {
    /**
Folds the elements of the iterator together, from left to right, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldl(self, f: |E, E| -> E) -> Option<E>;

    /**
Folds the elements of the iterator together, from left to right, using `f.

In addition, the first element is transformed using `map` before folding begins.

Returns `None` if the iterator is empty.
    */
    fn foldl_map<F>(self, map: |E| -> F, f: |F, E| -> F) -> Option<F>;
}

impl<It, E> IteratorFoldl<E> for It where It: Iterator<E> {
    fn foldl(mut self, f: |E, E| -> E) -> Option<E> {
        let first = match self.next() {
            None => return None,
            Some(e) => e
        };

        Some(self.fold(first, f))
    }

    fn foldl_map<F>(mut self, map: |E| -> F, f: |F, E| -> F) -> Option<F> {
        let first = match self.next() {
            None => return None,
            Some(e) => map(e)
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

#[test]
fn test_foldl_map() {
    let v = vec!["a", "b", "c"];
    let r = v.into_iter().foldl_map(|e| e.into_string(), |e,f| (e+", ")+f);
    assert_eq!(r, Some("a, b, c".into_string()));
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

pub trait IteratorIntersperse<E> {
    /**
Creates an iterator that yields `inject` between each element of the input iterator.  `inject` will not appear as the first or last element of the resulting iterator.
    */
    fn intersperse(self, inject: E) -> IntersperseItems<E, Self>;
}

impl<E, It> IteratorIntersperse<E> for It where It: Iterator<E> {
    fn intersperse(mut self, inject: E) -> IntersperseItems<E, It> {
        let look_ahead = self.next();
        IntersperseItems {
            iter: self,
            look_ahead: look_ahead,
            inject: inject,
            next_is_inject: false,
        }
    }
}

#[deriving(Clone, Show)]
pub struct IntersperseItems<E, It> {
    iter: It,
    look_ahead: Option<E>,
    inject: E,
    next_is_inject: bool,
}

impl<E, It> Iterator<E> for IntersperseItems<E, It> where E: Clone, It: Iterator<E> {
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

    fn size_hint(&self) -> (uint, Option<uint>) {
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

impl<E, It> RandomAccessIterator<E> for IntersperseItems<E, It> where E: Clone, It: RandomAccessIterator<E> {
    fn indexable(&self) -> uint {
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

    fn idx(&mut self, index: uint) -> Option<E> {
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
    let v: Vec<&str> = vec![];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec![]);

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

pub trait IteratorKeepSome {
    /**
Creates an iterator that, given a sequence of `Option<E>` values, unwraps all `Some(E)`s, and discards all `None`s.
    */
    fn keep_some(self) -> KeepSomeItems<Self>;
}

impl<E, It> IteratorKeepSome for It where It: Iterator<Option<E>> {
    fn keep_some(self) -> KeepSomeItems<It> {
        KeepSomeItems {
            iter: self,
        }
    }
}

#[deriving(Clone, Show)]
pub struct KeepSomeItems<It> {
    iter: It,
}

impl<It> KeepSomeItems<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
}

impl<E, It> Iterator<E> for KeepSomeItems<It> where It: Iterator<Option<E>> {
    fn next(&mut self) -> Option<E> {
        loop {
            match self.iter.next() {
                Some(v @ Some(_)) => return v,
                Some(None) => { /* do nothing */ },
                None => return None,
            }
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (_, mu) = self.iter.size_hint();
        (0, mu)
    }
}

#[test]
fn test_keep_some() {
    let v = vec![None, Some(0u), Some(1), None, Some(2), None, None, Some(3), None];
    let r: Vec<_> = v.into_iter().keep_some().collect();
    assert_eq!(r, vec![0, 1, 2, 3]);
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

impl<'a, It, E> PadTailToItems<'a, It, E> {
    /**
Unwraps the iterator, returning the underlying iterator and filler closure.
    */
    pub fn unwrap(self) -> (It, |uint|: 'a -> E) {
        let PadTailToItems { iter, filler, .. } = self;
        (iter, filler)
    }
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

impl<It> PacingWalkItems<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        let PacingWalkItems { iter, .. } = self;
        iter
    }
}

impl<E, It> Iterator<E> for PacingWalkItems<It> where E: ::std::fmt::Show, It: RandomAccessIterator<E> {
    fn next(&mut self) -> Option<E> {
        // Figure out if we need to stop.  This isn't immediately obvious, due to the way we handle the spiralling.
        let uint_max: uint = Int::max_value();
        let iter_len = self.iter.indexable();
        let left_len = iter_len - self.start_at;
        let stop_pos = (
            2.checked_mul(max(left_len, iter_len - left_len)).and_then(|l| l.checked_add(1))
        ).unwrap_or(uint_max);
        if self.pos >= stop_pos { return None }

        // Gives us 0 (jitter left) or 1 (jitter right).
        let jitter_right: uint = self.pos & 1;

        // Gives us the magnitude of the jitter.
        let mag = self.pos.checked_add(1).map(|l| l / 2);

        // If `mag` has overflowed, it's because `self.pos == uint::MAX`.  However, we know the answer to this...
        let mag: uint = mag.unwrap_or((uint_max / 2) + 1);

        // We can now compute the actual index into `iter` we want to use.  Of course, this may very well be out of bounds!
        // This could *possibly* be improved by computing when we need to stop doing the radial walk and just continue with a linear one instead.  For now, I'm just going to skip this position.

        if jitter_right == 0 && mag > self.start_at {
            return match self.pos.checked_add(1) {
                None => None,
                Some(pos) => {
                    self.pos = pos;
                    self.next()
                }
            }
        }

        if jitter_right > 0 && mag >= self.iter.indexable() - self.start_at {
            return match self.pos.checked_add(1) {
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
                match self.pos.checked_add(1) {
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

impl<It0, It1> RoundRobinItems<It0, It1> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (It0, It1) {
        let RoundRobinItems { it0, it1, .. } = self;
        (it0, it1)
    }
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

impl<It0, It1> RoundRobinLongestItems<It0, It1> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (It0, It1) {
        let RoundRobinLongestItems { it0, it1, .. } = self;
        (it0, it1)
    }
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
    /**
Skips *exactly* `n` elements from the iterator.

# Failure

This method will panic if there are less than `n` elements in the iterator.
    */
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
Returns a `Vec` with the elements of the input iterator in sorted order.
    */
    fn sorted(self) -> Vec<E>;

    /**
Returns a `Vec` with the elements of the input iterator in sorted order, as specified by a comparison function.
    */
    fn sorted_by(self, compare: |&E, &E| -> Ordering) -> Vec<E>;
}

impl<E, It> IteratorSorted<E> for It where E: Ord, It: Iterator<E> {
    fn sorted(self) -> Vec<E> {
        let mut v = self.collect::<Vec<_>>();
        v.sort();
        v
    }

    fn sorted_by(self, compare: |&E, &E| -> Ordering) -> Vec<E> {
        let mut v = self.collect::<Vec<_>>();
        v.sort_by(compare);
        v
    }
}

#[test]
fn test_sorted() {
    let v = vec![1u, 3, 2, 0, 4];
    let s = v.into_iter().sorted();
    assert_eq!(s, vec![0u, 1, 2, 3, 4]);
}

#[test]
fn test_sorted_by() {
    let v = vec![1u, 3, 2, 0, 4];
    let s = v.into_iter().sorted_by(|a,b| (!*a).cmp(&!*b));
    assert_eq!(s, vec![4, 3, 2, 1, 0u]);
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

impl<It> StrideItems<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
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
        index.checked_mul(self.stride).and_then(|i| self.iter.idx(i))
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
    /**
Creates an iterator that yields *exactly* `n` elements from the subject iterator.

# Failure

The iterator will panic if there are less than `n` elements in the subject iterator.
    */
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

impl<It> TakeExactlyItems<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
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

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `TeeItemsState` value.
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
                        state.buffer.push_back(e.clone());
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
    let (a, b) = v.into_iter().tee();
    assert_eq!(a.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(b.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

    let v = vec![0u, 1, 2, 3];
    let (a, b) = v.into_iter().tee();
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

#[deriving(Clone, Show)]
pub struct ZipLongestItems<It0, It1> {
    it0: It0,
    it1: It1,
}

impl<It0, It1> ZipLongestItems<It0, It1> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (It0, It1) {
        let ZipLongestItems { it0, it1 } = self;
        (it0, it1)
    }
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
