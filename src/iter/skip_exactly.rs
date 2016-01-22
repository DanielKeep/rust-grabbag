/*
Copyright ⓒ 2015 grabbag contributors.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
#[test]
fn test_skip() {
    let v = vec![0isize, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0isize, 1];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![]);
}

/**
(..., <em>a</em><sub><em>i</em>-1</sub>, <em>a</em><sub><em>i</em></sub>, <em>a</em><sub><em>i</em>+1</sub>, ...), <em>i</em>
&nbsp;&rarr;&nbsp;
(<em>a</em><sub><em>i</em></sub>, <em>a</em><sub><em>i</em>+1</sub>, ...)

*/
pub trait SkipExactlyIterator: Iterator + Sized {
    /**
Skips *exactly* `n` elements from the iterator.

# Failure

This method will panic if there are less than `n` elements in the iterator.
    */
    fn skip_exactly(mut self, n: usize) -> Self {
        for i in 0..n {
            match self.next() {
                None => panic!("skip_exactly asked to skip {} elements, but only got {}", n, i),
                _ => ()
            }
        }
        self
    }
}

impl<It> SkipExactlyIterator for It where It: Iterator {}

#[test]
fn test_skip_exactly() {
    use std::thread;

    let v = vec![0isize, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip_exactly(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0isize, 1];
    let r = thread::spawn(move || { v.into_iter().skip_exactly(3).collect::<Vec<_>>(); }).join();
    assert!(r.is_err());
}
