pub trait IteratorFoldl<E> {
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
