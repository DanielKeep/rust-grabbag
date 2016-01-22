/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
use std::cell::RefCell;
use std::cmp::min;
use std::mem::replace;
use std::rc::Rc;

/**
Sequence of iterators containing successive elements of the subject which have the same group according to a group function.
*/
pub trait GroupByIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that yields a succession of `(group, sub_iterator)` pairs.  Each `sub_iterator` yields successive elements of the input iterator that have the same `group`.  An element's `group` is computed using the `f` closure.

For example:

```
# extern crate grabbag;
# use grabbag::iter::GroupByIterator;
# fn main () {
let v = vec![7usize, 5, 6, 2, 4, 7, 6, 1, 6, 4, 4, 6, 0, 0, 8, 8, 6, 1, 8, 7];
let is_even = |n: &usize| if *n & 1 == 0 { true } else { false };
for (even, mut ns) in v.into_iter().group_by(is_even) {
    println!("{}...", if even { "Evens" } else { "Odds" });
    for n in ns {
        println!(" - {}", n);
    }
}
# }
```
    */
    fn group_by<GroupFn: FnMut(&E) -> G, G>(self, group: GroupFn) -> GroupBy<Self, GroupFn, E, G> {
        GroupBy {
            state: Rc::new(RefCell::new(GroupByShared {
                iter: self,
                group: group,
                last_group: None,
                push_back: None,
            })),
        }
    }
}

impl<E, It> GroupByIterator<E> for It where It: Iterator<Item=E> {}

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `GroupByItemsShared` value.
pub struct GroupBy<It, GroupFn, E, G> {
    state: Rc<RefCell<GroupByShared<It, GroupFn, E, G>>>,
}

pub struct GroupByShared<It, GroupFn, E, G> {
    iter: It,
    group: GroupFn,
    last_group: Option<G>,
    push_back: Option<(G, E)>,

}

impl<It, GroupFn, E, G> Iterator for GroupBy<It, GroupFn, E, G> where GroupFn: FnMut(&E) -> G, It: Iterator<Item=E>, G: Clone + Eq {
    type Item = (G, Group<It, GroupFn, E, G>);

    fn next(&mut self) -> Option<(G, Group<It, GroupFn, E, G>)> {
        // First, get a mutable borrow to the underlying state.
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

        // If we have a push-back element, immediately construct a sub iterator.
        if let Some((g, e)) = replace(&mut state.push_back, None) {
            return Some((
                g.clone(),
                Group {
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
            Group {
                state: self.state.clone(),
                group_value: g,
                first_value: Some(e),
            }
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lb, mub) = self.state.borrow().iter.size_hint();
        let lb = min(lb, 1);
        (lb, mub)
    }
}

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `GroupByShared` value.
pub struct Group<It, GroupFn, E, G> {
    state: Rc<RefCell<GroupByShared<It, GroupFn, E, G>>>,
    group_value: G,
    first_value: Option<E>,
}

impl<It, GroupFn, E, G> Iterator for Group<It, GroupFn, E, G> where GroupFn: FnMut(&E) -> G, It: Iterator<Item=E>, G: Eq {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        // If we have a first_value, consume and yield that.
        if let Some(e) = replace(&mut self.first_value, None) {
            return Some(e)
        }

        // Get a mutable borrow to the shared state.
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let state = self.state.borrow();

        let lb = if self.first_value.is_some() { 1 } else { 0 };
        let (_, mub) = state.iter.size_hint();
        (lb, mub)
    }
}

#[test]
fn test_group_by() {
    {
        let v = vec![0usize, 1, 2, 3, 5, 4, 6, 8, 7];
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
        let v = vec![0usize, 1, 2, 3, 5, 4, 6, 8, 7];
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
