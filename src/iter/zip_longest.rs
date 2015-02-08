use std::cmp::max;

/**
(..., <em>a</em><sub><em>m</em></sub>),
(..., <em>b</em><sub><em>m</em></sub>, <em>b</em><sub><em>m</em>+1</sub>, ..., <em>b</em><sub><em>n</em></sub>)
&nbsp;&rarr;&nbsp;
(
...,
(<em>Some</em>(<em>a</em><sub><em>m</em></sub>), <em>Some</em>(<em>b</em><sub><em>m</em></sub>)),
(<em>None</em>, <em>Some</em>(<em>b</em><sub><em>m</em>+1</sub>)),
...,
(<em>None</em>, <em>Some</em>(<em>b</em><sub><em>n</em></sub>))
)

*/
pub trait ZipLongestIterator: Iterator + Sized {
    /**
Creates an iterator which yields elements from both input iterators in lockstep.  If one iterator ends before the other, the elements from that iterator will be replaced with `None`.
    */
    fn zip_longest<RightIt>(self, right: RightIt) -> ZipLongest<Self, RightIt> {
        ZipLongest {
            left: self,
            right: right,
        }
    }
}

impl<LeftIt> ZipLongestIterator for LeftIt where LeftIt: Iterator {}

#[derive(Clone, Debug)]
pub struct ZipLongest<LeftIt, RightIt> {
    left: LeftIt,
    right: RightIt,
}

impl<LeftIt, RightIt> ZipLongest<LeftIt, RightIt> {
    /**
Unwraps the iterator, returning the underlying iterators.
    */
    pub fn unwrap(self) -> (LeftIt, RightIt) {
        let ZipLongest { left, right } = self;
        (left, right)
    }
}

impl<LeftIt, RightIt, LeftE, RightE> Iterator for ZipLongest<LeftIt, RightIt> where LeftIt: Iterator<Item=LeftE>, RightIt: Iterator<Item=RightE> {
    type Item = (Option<LeftE>, Option<RightE>);

    fn next(&mut self) -> Option<(Option<LeftE>, Option<RightE>)> {
        match (self.left.next(), self.right.next()) {
            (None, None) => None,
            e @ _ => Some(e)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l0, mu0) = self.left.size_hint();
        let (l1, mu1) = self.right.size_hint();
        let l = max(l0, l1);
        let mu = match (mu0, mu1) {
            (None, _) | (_, None) => None,
            (Some(u0), Some(u1)) => Some(max(u0, u1))
        };
        (l, mu)
    }
}

#[test]
fn test_zip_longest() {
    let a = vec![0us, 1, 2, 3];
    let b = vec!["a", "b", "c"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
        (Some(3), None),
    ]);

    let a = vec![0us, 1, 2];
    let b = vec!["a", "b", "c"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
    ]);

    let a = vec![0us, 1, 2];
    let b = vec!["a", "b", "c", "d"];
    let r: Vec<_> = a.into_iter().zip_longest(b.into_iter()).collect();
    assert_eq!(r, vec![
        (Some(0), Some("a")),
        (Some(1), Some("b")),
        (Some(2), Some("c")),
        (None, Some("d")),
    ]);
}
