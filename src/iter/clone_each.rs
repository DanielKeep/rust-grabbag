/**
(
<em>a</em><sub>0</sub>,
<em>a</em><sub>1</sub>,
...
)
&nbsp;&rarr;&nbsp;
(
<em>a</em><sub>0</sub>`.clone()`,
<em>a</em><sub>1</sub>`.clone()`,
...
)

*/
pub trait CloneEachIterator<'a, E>: Iterator<Item=&'a E> + Sized where E: 'a + Clone {
    /**
Creates an iterator which will clone each element of the input iterator.
    */
    fn clone_each(self) -> CloneEach<Self> {
        CloneEach {
            iter: self,
        }
    }
}

impl<'a, It, E> CloneEachIterator<'a, E> for It where It: Iterator<Item=&'a E>, E: 'a + Clone {}

#[derive(Clone, Debug)]
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, E, It> DoubleEndedIterator for CloneEach<It> where It: DoubleEndedIterator<Item=&'a E>, E: 'a + Clone {
    fn next_back(&mut self) -> Option<E> {
        self.iter.next_back().map(|e| e.clone())
    }
}

#[test]
fn test_clone_each() {
    let it: Vec<i32> = vec![1, 2, 3];
    let mut it = it.iter().clone_each();
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), None);
}
