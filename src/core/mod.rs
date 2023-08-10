mod arbitrary;
mod container;
mod fmt;
mod multiops;
mod proptests;
mod store;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod cmp;
mod inherent;
mod iter;
mod ops;

#[cfg(feature = "serde")]
mod serde;

mod serialization;

use self::cmp::Pairs;
use crate::Value;

pub use self::iter::IntoIter;
pub use self::iter::Iter;

/// A generic compressed bitmap using the [Roaring bitmap compression scheme](https://roaringbitmap.org/).
#[derive(PartialEq)]
pub struct RoaringBitmap<V: Value> {
    containers: Vec<container::Container<V>>,
}
