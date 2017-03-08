use RoaringBitmap;
use std::collections::BTreeMap;

//mod store;
//mod container;
mod util;
//mod fmt;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod inherent;
//mod iter;
//mod ops;
//mod cmp;
//mod serialization;

/// A compressed bitmap with u64 keys.
/// Implemented as a `BTreeMap` of `RoaringBitmap`s.
///
/// # Examples
///
/// ```rust
/// use roaring::RoaringBitmap64;
///
/// let mut rb = RoaringBitmap64::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
#[derive(PartialEq, Clone)]
pub struct RoaringBitmap64 {
    map: BTreeMap<u32, RoaringBitmap>,
}
