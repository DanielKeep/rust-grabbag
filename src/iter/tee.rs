use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/**
<em>a</em> &nbsp;&rarr;&nbsp; <em>a</em>, <em>a</em>
*/
pub trait TeeIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates a pair of iterators that will yield the same sequence of values.

The element type must implement Clone.
    */
    fn tee(self) -> (Tee<E, Self>, Tee<E, Self>) {
        let tee0 = Tee {
            state: Rc::new(RefCell::new(TeeState {
                iter: self,
                iter_next: 0,
                buffer: VecDeque::new(),
            })),
            this_next: 0,
        };
        let tee1 = Tee {
            state: tee0.state.clone(),
            this_next: 0,
        };
        (tee0, tee1)
    }
}

impl<It, E> TeeIterator<E> for It where It: Iterator<Item=E> {}

// **NOTE**: Although `Clone` *can* be implemented for this, you *should not* do so, since you cannot clone the underlying `TeeState` value.
pub struct Tee<E, It> {
    /*
        /!\ Important /!\

    In order for this to *not* panic, you need to ensure that nothing is re-entrant whilst it holds a mutable reference to the `RefCell`.  This should still be safe from the user's perspective.
    */
    state: Rc<RefCell<TeeState<E, It>>>,
    this_next: usize,
}

pub struct TeeState<E, It> {
    iter: It,
    iter_next: usize,
    buffer: VecDeque<E>,
}

impl<E, It> Iterator for Tee<E, It> where It: Iterator<Item=E>, E: Clone {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        let mut state = self.state.borrow_mut();
        let state = &mut *state;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let state = self.state.borrow_mut();
        let state = &*state;

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
    let v = vec![0usize, 1, 2, 3];
    let (a, b) = v.into_iter().tee();
    assert_eq!(a.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(b.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

    let v = vec![0usize, 1, 2, 3];
    let (a, b) = v.into_iter().tee();
    assert_eq!(b.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!(a.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

    let v = vec![0usize, 1, 2, 3];
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
