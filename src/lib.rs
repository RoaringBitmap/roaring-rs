//! This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
//! defined as a [Java library][roaring-java] and described in [_Better bitmap
//! performance with Roaring bitmaps_][roaring-paper].
//!
//! [Rust]: https://rust-lang.org
//! [Roaring bitmap]: http://roaringbitmap.org
//! [roaring-java]: https://github.com/lemire/RoaringBitmap
//! [roaring-paper]: http://arxiv.org/pdf/1402.6407v4

#![feature(slicing_syntax)]
#![feature(advanced_slice_patterns)]
#![feature(associated_types)]
#![feature(default_type_params)]

#![warn(missing_docs)]

use std::fmt::{ Show, Formatter, Result };
use std::ops::{ BitXor, BitAnd, BitOr, Sub };
use std::iter::{ FromIterator };

pub use iter::{ Iter, UnionIter, IntersectionIter, DifferenceIter, SymmetricDifferenceIter };

mod imp;
mod util;
mod iter;
mod store;
mod container;

/// A compressed bitmap using the [Roaring bitmap compression scheme](http://roaringbitmap.org).
///
/// # Examples
///
/// ```rust
/// # #![feature(slicing_syntax)]
/// # extern crate roaring;
/// # fn main() {
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
/// # }
/// ```
#[derive(PartialEq, Clone)]
pub struct RoaringBitmap {
    containers: Vec<container::Container>,
}

impl RoaringBitmap {
    /// Creates an empty `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    /// let mut rb = RoaringBitmap::new();
    /// # }
    /// ```
    #[inline]
    pub fn new() -> Self {
        imp::new()
    }

    /// Adds a value to the set. Returns `true` if the value was not already present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.insert(3), true);
    /// assert_eq!(rb.insert(3), false);
    /// assert_eq!(rb.contains(3), true);
    /// # }
    /// ```
    #[inline]
    pub fn insert(&mut self, value: u32) -> bool {
        imp::insert(self, value)
    }

    /// Removes a value from the set. Returns `true` if the value was present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(3);
    /// assert_eq!(rb.remove(3), true);
    /// assert_eq!(rb.remove(3), false);
    /// assert_eq!(rb.contains(3), false);
    /// # }
    /// ```
    #[inline]
    pub fn remove(&mut self, value: u32) -> bool {
        imp::remove(self, value)
    }

    /// Returns `true` if this set contains the specified integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(0), false);
    /// assert_eq!(rb.contains(1), true);
    /// assert_eq!(rb.contains(100), false);
    /// # }
    /// ```
    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        imp::contains(self, value)
    }

    /// Clears all integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(1), true);
    /// rb.clear();
    /// assert_eq!(rb.contains(1), false);
    /// # }
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        imp::clear(self)
    }

    /// Returns `true` if there are no integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.is_empty(), true);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.is_empty(), false);
    /// # }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        imp::is_empty(self)
    }

    /// Returns the number of distinct integers added to the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.len(), 0);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.len(), 1);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.len(), 2);
    /// # }
    /// ```
    #[inline]
    pub fn len(&self) -> uint {
        imp::len(self)
    }

    /// Iterator over each u32 stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    ///
    /// rb.insert(1);
    /// rb.insert(6);
    /// rb.insert(4);
    ///
    /// let mut iter = rb.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(4));
    /// assert_eq!(iter.next(), Some(6));
    /// assert_eq!(iter.next(), None);
    /// # }
    /// ```
    #[inline]
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        imp::iter(self)
    }

    /// Returns true if the set has no elements in common with other. This is equivalent to
    /// checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb1.is_disjoint(&rb2), true);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb1.is_disjoint(&rb2), false);
    ///
    /// # }
    /// ```
    #[inline]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        imp::is_disjoint(self, other)
    }

    /// Returns `true` if this set is a subset of `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), false);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), true);
    ///
    /// rb1.insert(2);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), false);
    /// # }
    /// ```
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        imp::is_subset(self, other)
    }

    /// Returns `true` if this set is a superset of `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), false);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), true);
    ///
    /// rb1.insert(2);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), false);
    /// # }
    /// ```
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        imp::is_superset(self, other)
    }

    /// Returns an iterator over the union of this bitmap with the `other` bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    /// rb1.insert(2);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    ///
    /// let mut iter = rb1.union(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// # }
    /// ```
    #[inline]
    pub fn union<'a>(&'a self, other: &'a Self) -> UnionIter<'a> {
        imp::union(self, other)
    }

    /// Returns an iterator over the intersection of this bitmap with the `other` bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter = rb1.intersection(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(4));
    /// assert_eq!(iter.next(), None);
    /// # }
    /// ```
    #[inline]
    pub fn intersection<'a>(&'a self, other: &'a Self) -> IntersectionIter<'a> {
        imp::intersection(self, other)
    }

    /// Returns an iterator over the set of `u32`s in `this` that are not in `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter1 = rb1.difference(&rb2);
    ///
    /// assert_eq!(iter1.next(), Some(2));
    /// assert_eq!(iter1.next(), None);
    ///
    /// let mut iter2 = rb2.difference(&rb1);
    ///
    /// assert_eq!(iter2.next(), Some(3));
    /// assert_eq!(iter2.next(), None);
    /// # }
    /// ```
    #[inline]
    pub fn difference<'a>(&'a self, other: &'a Self) -> DifferenceIter<'a> {
        imp::difference(self, other)
    }

    /// Returns an iterator over the set of `u32`s in `this` that are not in `other` + in `other`
    /// that are not in `this`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter = rb1.symmetric_difference(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// # }
    /// ```
    #[inline]
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifferenceIter<'a> {
        imp::symmetric_difference(self, other)
    }

    /// Unions in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// rb1.union_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// # }
    /// ```
    #[inline]
    pub fn union_with(&mut self, other: &Self) {
        imp::union_with(self, other)
    }

    /// Intersects in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (3..4).collect();
    ///
    /// rb1.intersect_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// # }
    /// ```
    #[inline]
    pub fn intersect_with(&mut self, other: &Self) {
        imp::intersect_with(self, other)
    }

    /// Removes all values in the specified other bitmap from self, in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
    ///
    /// rb1.difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// # }
    /// ```
    #[inline]
    pub fn difference_with(&mut self, other: &Self) {
        imp::difference_with(self, other)
    }

    /// Replaces this bitmap with one that is equivalent to `self XOR other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = ((1..3).chain(4..6)).collect();
    ///
    /// rb1.symmetric_difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// # }
    /// ```
    #[inline]
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        imp::symmetric_difference_with(self, other)
    }
}

impl FromIterator<u32> for RoaringBitmap {
    #[inline]
    fn from_iter<I: Iterator<Item = u32>>(iterator: I) -> RoaringBitmap {
        imp::from_iter(iterator)
    }
}

impl<'a> FromIterator<&'a u32> for RoaringBitmap {
    #[inline]
    fn from_iter<I: Iterator<Item = &'a u32>>(iterator: I) -> RoaringBitmap {
        imp::from_iter_ref(iterator)
    }
}

impl Extend<u32> for RoaringBitmap {
    #[inline]
    fn extend<I: Iterator<Item = u32>>(&mut self, iterator: I) {
        imp::extend(self, iterator)
    }
}

impl<'a> Extend<&'a u32> for RoaringBitmap {
    #[inline]
    fn extend<I: Iterator<Item = &'a u32>>(&mut self, iterator: I) {
        imp::extend_ref(self, iterator)
    }
}

impl BitOr<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Unions the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// let rb4 = rb1 | rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitor(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.union_with(&rhs);
        self
    }
}

impl<'a> BitOr<RoaringBitmap> for &'a RoaringBitmap {
    type Output = RoaringBitmap;

    /// Unions`rhs` and `self`, writes result in place to `rhs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// let rb4 = &rb1 | rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitor(self, mut rhs: RoaringBitmap) -> RoaringBitmap {
        rhs.union_with(self);
        rhs
    }
}

impl<'a, 'b> BitOr<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = RoaringBitmap;

    /// Unions`rhs` and `self`, allocates new bitmap for result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// let rb4 = rb1 | &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitor(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        let mut result = self.clone();
        result.union_with(rhs);
        result
    }
}

impl<'a> BitOr<&'a RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Unions the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// let rb4 = rb1 | &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitor(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.union_with(rhs);
        self
    }
}

impl BitAnd<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Intersects the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (2..4).collect();
    ///
    /// let rb4 = rb1 & rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitand(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.intersect_with(&rhs);
        self
    }
}

impl<'a> BitAnd<&'a RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Intersects the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (2..4).collect();
    ///
    /// let rb4 = rb1 & &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitand(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.intersect_with(rhs);
        self
    }
}

impl<'a> BitAnd<RoaringBitmap> for &'a RoaringBitmap {
    type Output = RoaringBitmap;

    /// Intersects `self` into the `rhs` `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (2..4).collect();
    ///
    /// let rb4 = &rb1 & rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitand(self, mut rhs: RoaringBitmap) -> RoaringBitmap {
        rhs.intersect_with(self);
        rhs
    }
}

impl<'a, 'b> BitAnd<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = RoaringBitmap;

    /// Intersects `self` and `rhs` into a new `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (2..5).collect();
    /// let rb3: RoaringBitmap = (2..4).collect();
    ///
    /// let rb4 = &rb1 & &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitand(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        let mut result = self.clone();
        result.intersect_with(rhs);
        result
    }
}

impl Sub<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
    ///
    /// let rb4 = rb1 - rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn sub(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.difference_with(&rhs);
        self
    }
}

impl<'a> Sub<&'a RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
    ///
    /// let rb4 = rb1 - &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn sub(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.difference_with(rhs);
        self
    }
}

impl<'a, 'b> Sub<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = RoaringBitmap;

    /// Subtracts `rhs` from `self` and allocates a new `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
    ///
    /// let rb4 = &rb1 - &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn sub(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        let mut result = self.clone();
        result.difference_with(rhs);
        result
    }
}

impl BitXor<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Subtracts the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = ((1..3).chain(4..6)).collect();
    ///
    /// let rb4 = rb1 ^ rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitxor(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference_with(&rhs);
        self
    }
}

impl<'a> BitXor<&'a RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// Exclusive ors the `rhs` into this `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = ((1..3).chain(4..6)).collect();
    ///
    /// let rb4 = rb1 ^ &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitxor(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference_with(rhs);
        self
    }
}

impl<'a> BitXor<RoaringBitmap> for &'a RoaringBitmap {
    type Output = RoaringBitmap;

    /// Exclusive ors `rhs` and `self`, writes result in place to `rhs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = ((1..3).chain(4..6)).collect();
    ///
    /// let rb4 = &rb1 ^ rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitxor(self, mut rhs: RoaringBitmap) -> RoaringBitmap {
        rhs.symmetric_difference_with(self);
        rhs
    }
}

impl<'a, 'b> BitXor<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = RoaringBitmap;

    /// Exclusive ors `rhs` and `self`, allocates a new bitmap for the result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(slicing_syntax)]
    /// # extern crate roaring;
    /// # fn main() {
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = ((1..3).chain(4..6)).collect();
    ///
    /// let rb4 = &rb1 ^ &rb2;
    ///
    /// assert_eq!(rb3, rb4);
    /// # }
    /// ```
    #[inline]
    fn bitxor(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        let mut result = self.clone();
        result.symmetric_difference_with(rhs);
        result
    }
}

impl Show for RoaringBitmap {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        format!("RoaringBitmap<{} values between {} and {}>", self.len(), imp::min(self), imp::max(self)).fmt(formatter)
    }
}
