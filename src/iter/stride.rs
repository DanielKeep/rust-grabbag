/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/**
<em>a</em>, <em>n</em>
&nbsp;&rarr;&nbsp;
(<em>a</em><sub><em>i</em></sub> | <em>a</em><sub><em>i</em></sub> &nbsp;&rarr;&nbsp; <em>a</em> : <em>i</em> &equiv; <em>mod</em> <em>n</em> )

*/
pub trait StrideIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator which yields every `n`th element of the input iterator, including the first.
    */
    fn stride(self, n: usize) -> Stride<Self> {
        Stride {
            iter: self,
            stride: n
        }
    }
}

impl<It, E> StrideIterator<E> for It where It: Iterator<Item=E> {}

#[derive(Clone, Debug)]
pub struct Stride<It> {
    iter: It,
    stride: usize,
}

impl<It> Stride<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
}

impl<It, E> Iterator for Stride<It> where It: Iterator<Item=E> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        let v = match self.iter.next() {
            Some(v) => v,
            None => return None
        };

        for _ in 0..(self.stride - 1) {
            match self.iter.next() {
                None => break,
                _ => ()
            }
        }

        Some(v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.iter.size_hint() {
            (lb, Some(ub)) => ((lb + self.stride - 1) / self.stride,
                                Some((ub + self.stride - 1) / self.stride)),
            (lb, None) => (lb, None)
        }
    }
}

#[test]
fn test_stride() {
    use super::CloneEachIterator;

    let v = vec![0isize, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut it = v.iter().clone_each().stride(2);
    assert_eq!(it.size_hint(), (5, Some(5)));
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), Some(6));
    assert_eq!(it.next(), Some(8));
    assert_eq!(it.next(), None);

    let v = vec![0isize, 1, 2, 3, 4, 5];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (2, Some(2)));

    let v = vec![0isize, 1, 2, 3, 4, 5, 6];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0isize, 1, 2, 3, 4, 5, 6, 7];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0isize, 1, 2, 3, 4, 5, 6, 7, 8];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (3, Some(3)));

    let v = vec![0isize, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let it = v.iter().clone_each().stride(3);
    assert_eq!(it.size_hint(), (4, Some(4)));
}
