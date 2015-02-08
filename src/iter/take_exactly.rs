use std::iter::RandomAccessIterator;

#[test]
fn test_take() {
    let v = vec![0is, 1, 2, 3];
    let r: Vec<_> = v.into_iter().take(3).collect();
    assert_eq!(r, vec![0, 1, 2]);

    let v = vec![0is, 1];
    let r: Vec<_> = v.into_iter().take(3).collect();
    assert_eq!(r, vec![0, 1]);
}

/**
(<em>a</em><sub>0</sub>, ..., <em>a</em><sub><em>i</em>-1</sub>, <em>a</em><sub><em>i</em></sub>, <em>a</em><sub><em>i</em>+1</sub>, ...), <em>i</em>
&nbsp;&rarr;&nbsp;
(<em>a</em><sub>0</sub>, ..., <em>a</em><sub><em>i</em>-1</sub>)

*/
pub trait TakeExactlyIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that yields *exactly* `n` elements from the subject iterator.

# Failure

The iterator will panic if there are less than `n` elements in the subject iterator.
    */
    fn take_exactly(self, n: usize) -> TakeExactly<Self> {
        TakeExactly {
            iter: self,
            left: n,
        }
    }
}

impl<It, E> TakeExactlyIterator<E> for It where It: Iterator<Item=E> {}

#[derive(Clone, Debug)]
pub struct TakeExactly<It> {
    iter: It,
    left: usize,
}

impl<It> TakeExactly<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
}

impl<It, E> Iterator for TakeExactly<It> where It: Iterator<Item=E> {
    type Item = E;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.left, Some(self.left))
    }
}

impl<It, E> RandomAccessIterator for TakeExactly<It> where It: Iterator<Item=E> + RandomAccessIterator {
    fn indexable(&self) -> usize {
        self.left
    }

    fn idx(&mut self, index: usize) -> Option<E> {
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
    use std::thread::Thread;

    let v = vec![0is, 1, 2, 3];
    let r: Vec<_> = v.into_iter().take_exactly(3).collect();
    assert_eq!(r, vec![0, 1, 2]);

    let v = vec![0is, 1];
    let r: Result<Vec<_>, _> = Thread::scoped(move || v.into_iter().take_exactly(3).collect()).join();
    assert!(r.is_err());
}
