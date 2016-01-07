/**
<em>a</em> &nbsp;&rarr;&nbsp;
(<em>e</em> | <em>e</em> &nbsp;&rarr;&nbsp; <em>a</em> : <em>Some</em>(<em>e</em>))

*/
pub trait KeepSomeIterator: Sized {
    /**
Creates an iterator that, given a sequence of `Option<E>` values, unwraps all `Some(E)`s, and discards all `None`s.
    */
    fn keep_some(self) -> KeepSome<Self>;
}

impl<It> KeepSomeIterator for It where It: Iterator {
    fn keep_some(self) -> KeepSome<It> {
        KeepSome {
            iter: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeepSome<It> {
    iter: It,
}

impl<It> KeepSome<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
}

impl<E, It> Iterator for KeepSome<It> where It: Iterator<Item=Option<E>> {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        loop {
            match self.iter.next() {
                Some(v @ Some(_)) => return v,
                Some(None) => { /* do nothing */ },
                None => return None,
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, mu) = self.iter.size_hint();
        (0, mu)
    }
}

impl<E, It> DoubleEndedIterator for KeepSome<It> where It: DoubleEndedIterator + Iterator<Item=Option<E>> {
    fn next_back(&mut self) -> Option<E> {
        loop {
            match self.iter.next_back() {
                Some(v @ Some(_)) => return v,
                Some(None) => (),
                None => return None,
            }
        }
    }
}

#[test]
fn test_keep_some() {
    let v = vec![None, Some(0usize), Some(1), None, Some(2), None, None, Some(3), None];
    let r: Vec<_> = v.into_iter().keep_some().collect();
    assert_eq!(r, vec![0, 1, 2, 3]);
}
