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

/// A compressed bitmap using the [Roaring bitmap compression scheme](http://roaringbitmap.org).
pub mod bitmap;

/// A compressed bitmap with u64 values.  Implemented as a `BTreeMap` of `RoaringBitmap`s.
pub mod treemap;

pub use bitmap::RoaringBitmap;
pub use treemap::RoaringTreemap;
