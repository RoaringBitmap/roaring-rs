mod container;
mod fmt;
mod store;
mod util;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod cmp;
mod inherent;
mod iter;
mod ops;
mod serialization;

pub use self::iter::IntoIter;
pub use self::iter::Iter;

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
