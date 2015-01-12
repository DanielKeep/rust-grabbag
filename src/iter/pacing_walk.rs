use std::cmp::max;
use std::iter::RandomAccessIterator;
use std::num::Int;

pub trait PacingWalkIterator: Iterator + RandomAccessIterator + Sized {
    /**
Creates an iterator that performs a back-and-forth walk of the input iterator.

For example:

```
# use grabbag::iter::{CloneEachIterator, PacingWalkIterator};
let v: Vec<usize> = vec![0, 1, 2, 3, 4];

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
    fn pacing_walk(self, start_at: usize) -> PacingWalk<Self>;
}

impl<It, E> PacingWalkIterator for It where It: Iterator<Item=E> + RandomAccessIterator {
    fn pacing_walk(self, start_at: usize) -> PacingWalk<It> {
        PacingWalk {
            iter: self,
            start_at: start_at,
            pos: 0,
        }
    }
}

#[derive(Clone, Show)]
pub struct PacingWalk<It> {
    iter: It,
    start_at: usize,
    pos: usize,
}

impl<It> PacingWalk<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        let PacingWalk { iter, .. } = self;
        iter
    }
}

impl<It, E> Iterator for PacingWalk<It> where It: Iterator<Item=E> + RandomAccessIterator {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        // Figure out if we need to stop.  This isn't immediately obvious, due to the way we handle the spiralling.
        let uint_max: usize = ::std::usize::MAX;
        let iter_len = self.iter.indexable();
        let left_len = iter_len - self.start_at;
        let stop_pos = (
            2.checked_mul(max(left_len, iter_len - left_len)).and_then(|l| l.checked_add(1))
        ).unwrap_or(uint_max);
        if self.pos >= stop_pos { return None }

        // Gives us 0 (jitter left) or 1 (jitter right).
        //let jitter_right: usize = self.pos & 1;
        let jitter_right = (self.pos & 1) != 0;

        // Gives us the magnitude of the jitter.
        let mag = self.pos.checked_add(1).map(|l| l / 2);

        // If `mag` has overflowed, it's because `self.pos == usize::MAX`.  However, we know the answer to this...
        let mag: usize = mag.unwrap_or((uint_max / 2) + 1);

        // We can now compute the actual index into `iter` we want to use.  Of course, this may very well be out of bounds!
        // This could *possibly* be improved by computing when we need to stop doing the radial walk and just continue with a linear one instead.  For now, I'm just going to skip this position.

        if !jitter_right && mag > self.start_at {
            return match self.pos.checked_add(1) {
                None => None,
                Some(pos) => {
                    self.pos = pos;
                    self.next()
                }
            }
        }

        if jitter_right && mag >= self.iter.indexable() - self.start_at {
            return match self.pos.checked_add(1) {
                None => None,
                Some(pos) => {
                    self.pos = pos;
                    self.next()
                }
            }
        }

        let idx = match jitter_right { false => self.start_at - mag, true => self.start_at + mag };
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[test]
fn test_pacing_walk() {
    use super::CloneEachIterator;

    let v: Vec<usize> = vec![0, 1, 2, 3, 4];

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
