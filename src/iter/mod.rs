/*!
This module contains a set of iterator extensions.  Rather than being named for the type of iterator they are implemented on, they are named for the method (or group of associated methods) being implemented.

If you can't be bothered to import them individually, you can just pull in `iter::prelude::*` instead.

# Standard Features

The iterator extensions, where possible, should implement the following features:

- `Clone`, which produces an *independent* version of the iterator.
- `DoubleEndedIterator`.
- `ExactSizeIterator`.
- `RandomAccessIterator`.
- `Show`.
- Accurate `size_hint` (depending on the transform being performed, and the accuracy of the underlying iterator).
- An `unwrap` method, which returns any owned values passed into the iterator; typically, this is the original iterator.
*/

#[doc(inline)]
pub use self::prelude::{
    AccumulateIterator,
    CartesianProductIterator,
    CloneEachIterator,
    GroupByIterator,
    IndexedIterator,
    FoldlIterator, FoldrIterator,
    IntersperseIterator,
    KeepSomeIterator,
    PadTailToIterator,
    PacingWalkIterator,
    RoundRobinIterator,
    SkipExactlyIterator,
    SortedIterator,
    StrideIterator,
    TakeExactlyIterator,
    TeeIterator,
    ZipLongestIterator,
};

pub mod accumulate;
pub mod cartesian_product;
pub mod clone_each;
pub mod group_by;
pub mod indexed;
pub mod fold;
pub mod intersperse;
pub mod keep_some;
pub mod pad_tail_to;
pub mod pacing_walk;
pub mod round_robin;
pub mod skip_exactly;
pub mod sorted;
pub mod stride;
pub mod take_exactly;
pub mod tee;
pub mod zip_longest;

/**
This module just centralises all the iterator extension traits in one place, suitable for glob importing.
*/
pub mod prelude {
    pub use super::accumulate::AccumulateIterator;
    pub use super::cartesian_product::CartesianProductIterator;
    pub use super::clone_each::CloneEachIterator;
    pub use super::group_by::GroupByIterator;
    pub use super::indexed::IndexedIterator;
    pub use super::fold::{FoldlIterator, FoldrIterator};
    pub use super::intersperse::IntersperseIterator;
    pub use super::keep_some::KeepSomeIterator;
    pub use super::pad_tail_to::PadTailToIterator;
    pub use super::pacing_walk::PacingWalkIterator;
    pub use super::round_robin::RoundRobinIterator;
    pub use super::skip_exactly::SkipExactlyIterator;
    pub use super::sorted::SortedIterator;
    pub use super::stride::StrideIterator;
    pub use super::take_exactly::TakeExactlyIterator;
    pub use super::tee::TeeIterator;
    pub use super::zip_longest::ZipLongestIterator;
}
