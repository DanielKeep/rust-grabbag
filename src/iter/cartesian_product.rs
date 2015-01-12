pub trait CartesianProductIterator<LeftItem>: Iterator<Item=LeftItem> + Sized {
    /**
Creates an iterator that yields the cartesian product of two input iterators.

The element type of the first input iterator must implement Clone, as must the second iterator type.
    */
    fn cartesian_product<RightIt, RightItem>(self, right: RightIt) -> CartesianProduct<Self, RightIt, LeftItem, RightItem> where RightIt: Clone + Iterator<Item=RightItem> {
        CartesianProduct {
            left: self,
            right: right.clone(),
            right_checkpoint: right,
            left_value: None,
        }
    }
}

impl<LeftIt, LeftItem> CartesianProductIterator<LeftItem> for LeftIt where LeftIt: Iterator<Item=LeftItem>, LeftItem: Clone {}

#[derive(Clone, Show)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CartesianProduct<LeftIt, RightIt, LeftItem, RightItem> {
    left: LeftIt,
    right: RightIt,
    right_checkpoint: RightIt,
    left_value: Option<LeftItem>,
}

impl<LeftIt, RightIt, LeftItem, RightItem> Iterator for CartesianProduct<LeftIt, RightIt, LeftItem, RightItem> where LeftIt: Iterator<Item=LeftItem>, RightIt: Clone + Iterator<Item=RightItem>, LeftItem: Clone {
    type Item = (LeftItem, RightItem);

    fn next(&mut self) -> Option<(LeftItem, RightItem)> {
        loop {
            if let Some(e0) = self.left_value.clone() {
                match self.right.next() {
                    Some(e1) => return Some((e0, e1)),
                    None => {
                        self.left_value = None;
                        self.right = self.right_checkpoint.clone();
                    }
                }
            }

            match self.left.next() {
                Some(e0) => {
                    self.left_value = Some(e0);
                }
                None => return None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l0, mu0) = self.left.size_hint();
        let (l1, mu1) = self.right.size_hint();
        let (lc, muc) = self.right_checkpoint.size_hint();

        let combine_bounds: fn(usize, usize, usize) -> usize;

        if self.left_value.is_some() {
            combine_bounds = combine_bounds_with_partial;
        } else {
            combine_bounds = combine_bounds_without_partial;
        }

        let l = combine_bounds(l1, l0, lc);
        let mu = match (mu0, mu1, muc) {
            (None, _, _) | (_, None, _) | (_, _, None) => None,
            (Some(u0), Some(u1), Some(uc)) => Some(combine_bounds(u1, u0, uc))
        };

        (l, mu)
    }
}

fn combine_bounds_with_partial(b0: usize, b1: usize, bc: usize) -> usize {
    b1 + b0*bc
}

fn combine_bounds_without_partial(b0: usize, _: usize, bc: usize) -> usize {
    b0*bc
}

#[test]
fn test_cartesian_product() {
    use super::CloneEachIterator;

    let a = vec![0us, 1, 2];
    let b = vec![3us, 4];
    let r: Vec<_> = a.into_iter().cartesian_product(b.iter().clone_each()).collect();
    assert_eq!(r, vec![(0,3),(0,4),(1,3),(1,4),(2,3),(2,4)]);
}
