#[test]
fn test_skip() {
    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0i, 1];
    let r: Vec<_> = v.into_iter().skip(3).collect();
    assert_eq!(r, vec![]);
}

pub trait SkipExactlyIterator: Iterator + Sized {
    /**
Skips *exactly* `n` elements from the iterator.

# Failure

This method will panic if there are less than `n` elements in the iterator.
    */
    fn skip_exactly(mut self, n: uint) -> Self {
        for i in range(0, n) {
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
    use std::thread::Thread;

    let v = vec![0i, 1, 2, 3];
    let r: Vec<_> = v.into_iter().skip_exactly(3).collect();
    assert_eq!(r, vec![3]);

    let v = vec![0i, 1];
    let r: Result<Vec<_>, _> = Thread::spawn(move || v.into_iter().skip_exactly(3).collect()).join();
    assert!(r.is_err());
}
