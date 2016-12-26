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

extern crate num;
extern crate byteorder;

mod ops;
mod iter;
mod store;
mod container;
mod util;
mod serialization;
mod fmt;
mod cmp;

use num::Zero;

use util::{ Halveable, ExtInt };
use container::Container;

pub use iter::Iter;

/// A compressed bitmap using the [Roaring bitmap compression scheme](http://roaringbitmap.org).
///
/// # Examples
///
/// ```rust
/// use roaring::RoaringBitmap;
///
/// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
#[derive(PartialEq, Clone)]
pub struct RoaringBitmap<Size: ExtInt + Halveable> where <Size as Halveable>::HalfSize: ExtInt {
    containers: Vec<Container<<Size as Halveable>::HalfSize>>,
}

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    /// Creates an empty `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// ```
    pub fn new() -> Self {
        RoaringBitmap { containers: Vec::new() }
    }

    /// Adds a value to the set. Returns `true` if the value was not already present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// assert_eq!(rb.insert(3), true);
    /// assert_eq!(rb.insert(3), false);
    /// assert_eq!(rb.contains(3), true);
    /// ```
    pub fn insert(&mut self, value: Size) -> bool {
        let (key, index) = value.split();
        let container = match self.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => &mut self.containers[loc],
            Err(loc) => {
                self.containers.insert(loc, Container::new(key));
                &mut self.containers[loc]
            },
        };
        container.insert(index)
    }

    /// Removes a value from the set. Returns `true` if the value was present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// rb.insert(3);
    /// assert_eq!(rb.remove(3), true);
    /// assert_eq!(rb.remove(3), false);
    /// assert_eq!(rb.contains(3), false);
    /// ```
    pub fn remove(&mut self, value: Size) -> bool {
        let (key, index) = value.split();
        match self.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => {
                if self.containers[loc].remove(index) {
                    if self.containers[loc].len() == Zero::zero() {
                        self.containers.remove(loc);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Returns `true` if this set contains the specified integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(0), false);
    /// assert_eq!(rb.contains(1), true);
    /// assert_eq!(rb.contains(100), false);
    /// ```
    pub fn contains(&self, value: Size) -> bool {
        let (key, index) = value.split();
        match self.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => self.containers[loc].contains(index),
            Err(_) => false,
        }
    }

    /// Clears all integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(1), true);
    /// rb.clear();
    /// assert_eq!(rb.contains(1), false);
    /// ```
    pub fn clear(&mut self) {
        self.containers.clear();
    }

    /// Returns `true` if there are no integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// assert_eq!(rb.is_empty(), true);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }

    /// Returns the number of distinct integers added to the set.
    ///
    /// ## Warning
    ///
    /// Will overflow (panic in debug mode) if the set is 100% full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
    /// assert_eq!(rb.len(), 0);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.len(), 1);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.len(), 2);
    /// ```
    pub fn len(&self) -> Size {
        self.containers
            .iter()
            .map(|container| util::cast(container.len()))
            .sum()
    }

    fn min(&self) -> Option<Size> {
        self.containers.first()
            .map(|head| Halveable::join(head.key(), head.min()))
    }

    fn max(&self) -> Option<Size> {
        self.containers.last()
            .map(|tail| Halveable::join(tail.key(), tail.max()))
    }
}
