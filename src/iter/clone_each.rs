use std::iter::RandomAccessIterator;

pub trait CloneEachIterator<'a, E>: Iterator<Item=&'a E> + Sized where E: Clone {
    /**
Creates an iterator which will clone each element of the input iterator.
    */
    fn clone_each(self) -> CloneEach<Self> {
        CloneEach {
            iter: self,
        }
    }
}

impl<'a, It, E> CloneEachIterator<'a, E> for It where It: Iterator<Item=&'a E>, E: Clone {}

#[derive(Clone, Show)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CloneEach<It> {
    iter: It,
}

impl<It> CloneEach<It> {
    /**
Unwraps the iterator, returning the underlying iterator.
    */
    pub fn unwrap(self) -> It {
        self.iter
    }
}

impl<'a, It, E> Iterator for CloneEach<It> where It: Iterator<Item=&'a E>, E: 'a + Clone {
    type Item = E;

    fn next(&mut self) -> Option<E> {
        match self.iter.next() {
            None => None,
            Some(e) => Some(e.clone())
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, E, It> DoubleEndedIterator for CloneEach<It> where It: DoubleEndedIterator<Item=&'a E>, E: 'a + Clone {
    fn next_back(&mut self) -> Option<E> {
        self.iter.next_back().map(|e| e.clone())
    }
}

impl<'a, E, It> RandomAccessIterator for CloneEach<It> where It: RandomAccessIterator<Item=&'a E>, E: 'a + Clone {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<E> {
        self.iter.idx(index).map(|e| e.clone())
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
