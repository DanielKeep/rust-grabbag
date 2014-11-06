pub trait IteratorCloneEach<'a, E, It> where E: Clone, It: Iterator<&'a E> {
    /**
Creates an iterator which will clone each element of the input iterator.
    */
    fn clone_each(self) -> CloneItems<It>;
}

impl<'a, E, It> IteratorCloneEach<'a, E, It> for It where E: Clone, It: Iterator<&'a E> {
    fn clone_each(self) -> CloneItems<It> {
        CloneItems {
            iter: self,
        }
    }
}

pub struct CloneItems<It> {
    iter: It,
}

impl<'a, E, It> Iterator<E> for CloneItems<It> where E: 'a+Clone, It: Iterator<&'a E> {
    fn next(&mut self) -> Option<E> {
        match self.iter.next() {
            None => None,
            Some(e) => Some(e.clone())
        }
    }
}

#[test]
fn test_clone_each() {
    let it: Vec<int> = vec![1, 2, 3];
    let mut it = it.iter().clone_each();
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), None);
}

pub trait IteratorFoldl<E> {
    /**
Folds the elements of the iterator together, from left to right, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldl(self, f: |E, E| -> E) -> Option<E>;
}

impl<It, E> IteratorFoldl<E> for It where It: Iterator<E> {
    fn foldl(mut self, f: |E, E| -> E) -> Option<E> {
        let first = match self.next() {
            None => return None,
            Some(e) => e
        };

        Some(self.fold(first, f))
    }
}

#[test]
fn test_foldl() {
    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.into_string());
    assert_eq!(Some("((a, b), c)".into_string()), vs.foldl(|a,b| format!("({}, {})", a, b)));
}

pub trait IteratorFoldr<E> {
    /**
Folds the elements of the iterator together, from right to left, using `f`.

Returns `None` if the iterator is empty.
    */
    fn foldr(self, f: |E, E| -> E) -> Option<E>;
}

impl<It, E> IteratorFoldr<E> for It where It: DoubleEndedIterator<E> {
    fn foldr(mut self, f: |E, E| -> E) -> Option<E> {
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

#[test]
fn test_foldr() {
    let vs = vec!["a", "b", "c"];
    let vs = vs.into_iter().map(|e| e.into_string());
    assert_eq!(Some("(a, (b, c))".into_string()), vs.foldr(|a,b| format!("({}, {})", a, b)));
}
