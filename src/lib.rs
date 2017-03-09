//! This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
//! defined as a [Java library][roaring-java] and described in [_Better bitmap
//! performance with Roaring bitmaps_][roaring-paper].
//!
//! [Rust]: https://rust-lang.org
//! [Roaring bitmap]: http://roaringbitmap.org
//! [roaring-java]: https://github.com/lemire/RoaringBitmap
//! [roaring-paper]: http://arxiv.org/pdf/1402.6407v4

#![warn(missing_docs)]
#![warn(variant_size_differences)]

#![allow(unknown_lints)] // For clippy

extern crate byteorder;

mod store;
mod container;
mod util;
mod fmt;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod inherent;
mod iter;
mod ops;
mod cmp;
mod serialization;
mod roaring_treemap;

pub use iter::Iter;

/// A compressed bitmap using the [Roaring bitmap compression scheme](http://roaringbitmap.org).
///
/// # Examples
///
/// ```rust
/// use roaring::RoaringBitmap;
///
/// let mut rb = RoaringBitmap::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
#[derive(PartialEq, Clone)]
pub struct RoaringBitmap {
    containers: Vec<container::Container>,
}

pub use roaring_treemap::RoaringTreemap;
