use std::iter::RandomAccessIterator;

pub trait IndexedIterator<IndIt>: Iterator + Sized {
    /**
Creates an iterator which uses `indices` as an indexer to the subject iterator.
    */
    fn indexed(self, indices: IndIt) -> Indexed<Self, IndIt> {
        Indexed {
            iter: self,
            indices: indices,
        }
    }

    /**
Creates an iterator which uses `indices` as an indexer to the subject iterator, where the subject is behind a mutable reference.
    */
    fn indexed_view(&mut self, indices: IndIt) -> IndexedView<Self, IndIt> {
        IndexedView {
            iter: self,
            indices: indices,
        }
    }
}

impl<It, IndIt, E> IndexedIterator<IndIt> for It where It: RandomAccessIterator<Item=E> {}

#[derive(Clone, Show)]
pub struct Indexed<It, IndIt> {
    iter: It,
    indices: IndIt,
}

impl<It, IndIt> Indexed<It, IndIt> {
    /**
Unwraps the iterator, returning the underlying indexed and indexer iterators.
    */
    pub fn unwrap(self) -> (It, IndIt) {
        let Indexed { iter, indices } = self;
        (iter, indices)
    }
}

impl<It, IndIt, E> Iterator for Indexed<It, IndIt> where It: RandomAccessIterator<Item=E>, IndIt: Iterator<Item=usize> {
    type Item = Option<E>;

    fn next(&mut self) -> Option<Option<E>> {
        match self.indices.next() {
            None => None,
            Some(idx) => Some(self.iter.idx(idx))
        }
    }
}

impl<It, IndIt, E> DoubleEndedIterator for Indexed<It, IndIt> where It: RandomAccessIterator<Item=E>, IndIt: DoubleEndedIterator<Item=usize> {
    fn next_back(&mut self) -> Option<Option<E>> {
        self.indices.next_back().map(|e| self.iter.idx(e))
    }
}

impl<It, IndIt, E> RandomAccessIterator for Indexed<It, IndIt> where It: RandomAccessIterator<Item=E>, IndIt: RandomAccessIterator<Item=usize> {
    fn indexable(&self) -> usize {
        self.indices.indexable()
    }

    fn idx(&mut self, index: usize) -> Option<Option<E>> {
        self.indices.idx(index).and_then(|i| Some(self.iter.idx(i)))
    }
}

#[derive(Show)]
pub struct IndexedView<'a, It, IndIt> where It: 'a {
    iter: &'a mut It,
    indices: IndIt,
}

impl<'a, It, IndIt> IndexedView<'a, It, IndIt> {
    /**
Unwraps the iterator, returning the underlying indexed and indexer iterators.
    */
    pub fn unwrap(self) -> (&'a mut It, IndIt) {
        let IndexedView { iter, indices } = self;
        (iter, indices)
    }
}

impl<'a, It, IndIt, E> Iterator for IndexedView<'a, It, IndIt> where It: 'a + RandomAccessIterator<Item=E>, IndIt: Iterator<Item=usize> {
    type Item = Option<E>;

    fn next(&mut self) -> Option<Option<E>> {
        match self.indices.next() {
            None => None,
            Some(idx) => Some(self.iter.idx(idx))
        }
    }
}

impl<'a, It, IndIt, E> DoubleEndedIterator for IndexedView<'a, It, IndIt> where It: 'a + RandomAccessIterator<Item=E>, IndIt: DoubleEndedIterator<Item=usize> {
    fn next_back(&mut self) -> Option<Option<E>> {
        self.indices.next_back().map(|e| self.iter.idx(e))
    }
}

impl<'a, It, IndIt, E> RandomAccessIterator for IndexedView<'a, It, IndIt> where It: 'a + RandomAccessIterator<Item=E>, IndIt: RandomAccessIterator<Item=usize> {
    fn indexable(&self) -> usize {
        self.indices.indexable()
    }

    fn idx(&mut self, index: usize) -> Option<Option<E>> {
        self.indices.idx(index).and_then(|i| Some(self.iter.idx(i)))
    }
}

#[test]
fn test_indexed() {
    let v = vec![0us, 1, 2, 3, 4];
    let i = vec![2us, 4, 1, 0, 2, 3, 5];
    let r: Vec<_> = v.iter().indexed(i.into_iter()).map(|e| e.map(|v| *v)).collect();
    assert_eq!(r, vec![Some(2), Some(4), Some(1), Some(0), Some(2), Some(3), None])
}

#[test]
fn test_indexed_view() {
    use super::CloneEachIterator;

    let v = vec![0us, 1, 2, 3, 4];
    let mut v = v.iter().clone_each();
    let i = vec![2us, 4, 1, 0, 2, 3, 5];
    let r: Vec<_> = v.indexed_view(i.into_iter()).collect();
    assert_eq!(r, vec![Some(2), Some(4), Some(1), Some(0), Some(2), Some(3), None]);
    assert_eq!(v.collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
}
