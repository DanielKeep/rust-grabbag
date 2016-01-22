/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
use std::mem::replace;

/**
(<em>a</em><sub>0</sub>, <em>a</em><sub>1</sub>, ..., <em>a</em><sub>n</sub>), <em>i</em>
&nbsp;&rarr;&nbsp;
(<em>a</em><sub>0</sub>, <em>i</em>, <em>a</em><sub>1</sub>, <em>i</em>, ..., <em>i</em>, <em>a</em><sub>n</sub>)

*/
pub trait IntersperseIterator<E>: Iterator<Item=E> + Sized {
    /**
Creates an iterator that yields `inject` between each element of the input iterator.  `inject` will not appear as the first or last element of the resulting iterator.
    */
    fn intersperse(self, inject: E) -> Intersperse<Self, E>;
}

impl<It, E> IntersperseIterator<E> for It where It: Iterator<Item=E> {
    fn intersperse(mut self, inject: E) -> Intersperse<It, E> {
        let look_ahead = self.next();
        Intersperse {
            iter: self,
            look_ahead: look_ahead,
            inject: inject,
            next_is_inject: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Intersperse<It, E> {
    iter: It,
    look_ahead: Option<E>,
    inject: E,
    next_is_inject: bool,
}

impl<It, E> Iterator for Intersperse<It, E> where It: Iterator<Item=E>, E: Clone {
    type Item = E;

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

    fn size_hint(&self) -> (usize, Option<usize>) {
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

#[test]
fn test_intersperse() {
    let v: Vec<&str> = vec![];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, Vec::<&str>::new());

    let v = vec!["a"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a"]);

    let v = vec!["a", "b"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a", ",", "b"]);

    let v = vec!["a", "b", "c"];
    let r: Vec<_> = v.into_iter().intersperse(",").collect();
    assert_eq!(r, vec!["a", ",", "b", ",", "c"]);
}
