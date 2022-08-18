//! This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
//! defined as a [Java library][roaring-java] and described in [_Better bitmap
//! performance with Roaring bitmaps_][roaring-paper].
//!
//! [Rust]: https://www.rust-lang.org/
//! [Roaring bitmap]: https://roaringbitmap.org/
//! [roaring-java]: https://github.com/lemire/RoaringBitmap
//! [roaring-paper]: https://arxiv.org/pdf/1402.6407v4

#![cfg_attr(feature = "simd", feature(portable_simd))]
#![warn(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(variant_size_differences)]
#![allow(unknown_lints)] // For clippy

extern crate byteorder;

use std::error::Error;
use std::fmt;

/// A compressed bitmap using the [Roaring bitmap compression scheme](https://roaringbitmap.org/).
pub mod bitmap;

/// A compressed bitmap with u64 values.  Implemented as a `BTreeMap` of `RoaringBitmap`s.
pub mod treemap;

pub use bitmap::RoaringBitmap;
pub use treemap::RoaringTreemap;

/// An error type that is returned when an iterator isn't sorted.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonSortedIntegers {
    valid_until: u64,
}

impl NonSortedIntegers {
    /// Returns the number of elements that were
    pub fn valid_until(&self) -> u64 {
        self.valid_until
    }
}

impl fmt::Display for NonSortedIntegers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "integers are ordered up to the {}th element", self.valid_until())
    }
}

impl Error for NonSortedIntegers {}

/// An [`IntoIterator`] blanket implementation that provides extra methods for [`RoaringBitmap`]
/// and [`RoaringTreemap`]
pub trait IterExt<T>: IntoIterator<Item = T> {
    /// The type of output from operations.
    type Output;

    /// The `union` between all elements.
    fn or(self) -> Self::Output;

    /// The `intersection` between all elements.
    fn and(self) -> Self::Output;

    /// The `difference` between all elements.
    fn sub(self) -> Self::Output;

    /// The `symmetric difference` between all elements.
    fn xor(self) -> Self::Output;
}
