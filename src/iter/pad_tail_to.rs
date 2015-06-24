/**
Pads a sequence to a minimum length.
*/
pub trait PadTailToIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that ensures there are at least `n` elements in a sequence.  If the input iterator is too short, the difference is made up with a filler value.
    */
    fn pad_tail_to<F: FnMut(usize) -> E>(self, n: usize, filler: F) -> PadTailTo<Self, F> {
        PadTailTo {
            iter: self,
            min: n,
            pos: 0,
            filler: filler,
        }
    }
}

impl<It, E> PadTailToIterator<E> for It where It: Iterator<Item=E> {}

pub struct PadTailTo<It, F> {
    iter: It,
    min: usize,
    pos: usize,
    filler: F,
}

impl<It, F> PadTailTo<It, F> {
    /**
Unwraps the iterator, returning the underlying iterator and filler closure.
    */
    pub fn unwrap(self) -> (It, F) {
        let PadTailTo { iter, filler, .. } = self;
        (iter, filler)
    }
}

impl<It, E, F> Iterator for PadTailTo<It, F> where It: Iterator<Item=E>, F: FnMut(usize) -> E {
    type Item = E;

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

impl<It, E, F> DoubleEndedIterator for PadTailTo<It, F> where It: DoubleEndedIterator + ExactSizeIterator + Iterator<Item=E>, F: FnMut(usize) -> E {
    fn next_back(&mut self) -> Option<E> {
        if self.min == 0 {
            self.next_back()
        } else if self.iter.len() >= self.min {
            self.min -= 1;
            self.next_back()
        } else {
            self.min -= 1;
            Some((self.filler)(self.pos))
        }
    }
}

#[test]
fn test_pad_tail_to() {
    let v: Vec<usize> = vec![0, 1, 2];
    let r: Vec<_> = v.into_iter().pad_tail_to(5, |n| n).collect();
    assert_eq!(r, vec![0, 1, 2, 3, 4]);

    let v: Vec<usize> = vec![0, 1, 2];
    let r: Vec<_> = v.into_iter().pad_tail_to(1, |_| panic!()).collect();
    assert_eq!(r, vec![0, 1, 2]);
}
