use std::mem::replace;

/**
(
<em>a</em><sub>0</sub>,
<em>a</em><sub>1</sub>,
<em>a</em><sub>2</sub>,
...
),
&#x2297;
&nbsp;&rarr;&nbsp;
(
<em>a</em><sub>0</sub>,
(<em>a</em><sub>0</sub> &#x2297; <em>a</em><sub>1</sub>),
((<em>a</em><sub>0</sub> &#x2297; <em>a</em><sub>1</sub>) &#x2297; <em>a</em><sub>2</sub>),
...
)

*/
pub trait AccumulateIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that scans from left to right over the input sequence, returning the accumulated result of calling the provided function on the entire sequence up to that point.

# Example

```
let v = vec![0us, 1, 2, 3, 4];

// `r` is the sequence of partial sums of `v`.
let r: Vec<_> = v.into_iter().accumulate(|a,b| a+b).collect();
assert_eq!(r, vec![0, 1, 3, 6, 10]);
```
    */
    fn accumulate<F: FnMut(E, E) -> E>(self, f: F) -> Accumulate<Self, E, F> {
        Accumulate {
            iter: self,
            f: f,
            accum: None,
        }
    }
}

impl<E, It> AccumulateIterator<E> for It where It: Iterator<Item=E> {}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Accumulate<It, E, F> where It: Iterator<Item=E> {
    iter: It,
    f: F,
    accum: Option<E>,
}

impl<It, E, F> Accumulate<It, E, F> where It: Iterator<Item=E> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        let Accumulate { iter, .. } = self;
        iter
    }
}

impl<It, E, F> Iterator for Accumulate<It, E, F> where It: Iterator<Item=E>, F: FnMut(E, E) -> E, E: Clone {
    type Item = E;

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
    let v = vec![0us, 1, 2, 3, 4];
    let r: Vec<_> = v.into_iter().accumulate(|a,b| a+b).collect();
    assert_eq!(r, vec![0, 1, 3, 6, 10]);
}

#[test]
fn test_accumulate_unwrap() {
    let v = vec![0us, 1, 2, 3, 4];
    let mut i = v.into_iter().accumulate(|a,b| a+b);
    assert_eq!(i.next(), Some(0));
    assert_eq!(i.next(), Some(1));
    assert_eq!(i.next(), Some(3));
    let r: Vec<_> = i.unwrap().collect();
    assert_eq!(r, vec![3, 4]);
}
