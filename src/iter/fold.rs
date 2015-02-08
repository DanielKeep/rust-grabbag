/**
(
<em>a</em><sub>0</sub>,
<em>a</em><sub>1</sub>,
<em>a</em><sub>2</sub>,
...
),
&nbsp;&rarr;&nbsp;
&nbsp;&rarr;&nbsp;
((<em>a</em><sub>0</sub>) &#x2297; <em>a</em><sub>1</sub>) &#x2297; <em>a</em><sub>2</sub>) &#x2297; ...

*/
pub trait FoldlIterator<E>: Iterator<Item=E> + Sized {
    /**
Folds the elements of the iterator together, from left to right, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldl<F: FnMut(E, E) -> E>(mut self, f: F) -> Option<E> {
        let first = match self.next() {
            None => return None,
            Some(e) => e
        };

        Some(self.fold(first, f))
    }

    /**
Folds the elements of the iterator together, from left to right, using `f.

In addition, the first element is transformed using `map` before folding begins.

Returns `None` if the iterator is empty.
    */
    fn foldl_map<E1, F: FnMut(E1, E) -> E1, MapFn: FnOnce(E) -> E1>(mut self, map: MapFn, f: F) -> Option<E1> {
        let first = match self.next() {
            None => return None,
            Some(e) => map(e)
        };

        Some(self.fold(first, f))
    }
}

impl<It, E> FoldlIterator<E> for It where It: Iterator<Item=E> {}

#[test]
fn test_foldl() {
    use std::borrow::ToOwned;

    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.to_owned());
    assert_eq!(Some("((a, b), c)".to_owned()), vs.foldl(|a,b| format!("({}, {})", a, b)));
}

#[test]
fn test_foldl_map() {
    use std::borrow::ToOwned;

    let v = vec!["a", "b", "c"];
    let r = v.into_iter().foldl_map(|e| e.to_owned(), |e,f| (e+", ")+f);
    assert_eq!(r, Some("a, b, c".to_owned()));
}

/**
(
...,
<em>a</em><sub><em>n</em>-2</sub>,
<em>a</em><sub><em>n</em>-1</sub>,
<em>a</em><sub><em>n</em></sub>,
),
&#x2297;
&rarr;
... &#x2297; (<em>a</em><sub><em>n</em>-2</sub> &#x2297; (<em>a</em><sub><em>n</em>-1</sub> &#x2297; (<em>a</em><sub><em>n</em></sub>)))

*/
pub trait FoldrIterator<E>: DoubleEndedIterator + Iterator<Item=E> + Sized {
    /**
Folds the elements of the iterator together, from right to left, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldr<F: FnMut(E, E) -> E>(mut self, mut f: F) -> Option<E> {
        let mut last = match self.next_back() {
            None => return None,
            Some(e) => e
        };

        loop {
            match self.next_back() {
                None => break,
                Some(e) => last = f(e, last)
            }
        }

        Some(last)
    }
}

impl<It, E> FoldrIterator<E> for It where It: DoubleEndedIterator + Iterator<Item=E> {}

#[test]
fn test_foldr() {
    use std::borrow::ToOwned;

    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.to_owned());
    assert_eq!(Some("(a, (b, c))".to_owned()), vs.foldr(|a,b| format!("({}, {})", a, b)));
}
