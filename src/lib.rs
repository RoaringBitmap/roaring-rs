use std::{ u16 };
use std::slice::BinarySearchResult::{ Found, NotFound };

use iter::RoaringIterator;
use container::Container;

mod util;
mod iter;
mod store;
mod container;

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
pub struct RoaringBitmap {
    containers: Vec<Container>,
}

impl RoaringBitmap {
    /// Creates an empty `RoaringBitmap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// let mut rb = RoaringBitmap::new();
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
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.insert(3), true);
    /// assert_eq!(rb.insert(3), false);
    /// assert_eq!(rb.contains(3), true);
    /// ```
    pub fn insert(&mut self, value: u32) -> bool {
        let (key, index) = calc_loc(value);
        let container = match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => &mut self.containers[loc],
            NotFound(loc) => {
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
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(3);
    /// assert_eq!(rb.remove(3), true);
    /// assert_eq!(rb.remove(3), false);
    /// assert_eq!(rb.contains(3), false);
    /// ```
    pub fn remove(&mut self, value: u32) -> bool {
        let (key, index) = calc_loc(value);
        match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => {
                if self.containers[loc].remove(index) {
                    if self.containers[loc].len() == 0 {
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
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(0), false);
    /// assert_eq!(rb.contains(1), true);
    /// assert_eq!(rb.contains(100), false);
    /// ```
    pub fn contains(&self, value: u32) -> bool {
        let (key, index) = calc_loc(value);
        match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => self.containers[loc].contains(index),
            NotFound(_) => false,
        }
    }

    /// Clears all integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
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
    /// let mut rb = RoaringBitmap::new();
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
    /// # Examples
    ///
    /// ```rust
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
    /// ```
    pub fn len(&self) -> uint {
        self.containers
            .iter()
            .map(|container| container.len() as uint)
            .fold(0, |sum, len| sum + len)
    }

    /// Iterator over each u32 stored in the RoaringBitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    ///
    /// rb.insert(1);
    /// rb.insert(4);
    /// rb.insert(6);
    ///
    /// // Print 1, 4, 6 in arbitrary order
    /// for x in rb.iter() {
    ///     println!("{}", x);
    /// }
    /// ```
    pub fn iter<'a>(&'a self) -> RoaringIterator<'a> {
        RoaringIterator::new(box self.containers.iter())
    }

    /// Returns true if the set has no elements in common with other. This is equivalent to
    /// checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```rust
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
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool {
        let result: bool;
        let mut iter1 = self.containers.iter();
        let mut iter2 = other.containers.iter();
        let mut container1 = iter1.next();
        let mut container2 = iter2.next();
        loop {
            match (container1, container2) {
                (Some(c1), Some(c2)) => {
                    match (c1.key(), c2.key()) {
                    (key1, key2) if key1 == key2 => {
                        if !c1.is_disjoint(c2) {
                            result = false;
                            break;
                        }
                        container1 = iter1.next();
                    },
                    (key1, key2) if key1 < key2 => container1 = iter1.next(),
                    (key1, key2) if key1 > key2 => container2 = iter2.next(),
                    (_, _) => panic!(),
                    }
                },
                (_, _) => {
                    result = true;
                    break;
                },
            }
        }
        result
    }
}

impl FromIterator<u32> for RoaringBitmap {
    fn from_iter<I: Iterator<u32>>(iterator: I) -> RoaringBitmap {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl Extend<u32> for RoaringBitmap {
    fn extend<I: Iterator<u32>>(&mut self, mut iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

#[inline]
fn calc_loc(index: u32) -> (u16, u16) { ((index >> u16::BITS) as u16, index as u16) }

#[cfg(test)]
mod test {
    use std::{ u16, u32 };
    use super::{ calc_loc };

    #[test]
    fn test_calc_location() {
        assert_eq!((0, 0), calc_loc(0));
        assert_eq!((0, 1), calc_loc(1));
        assert_eq!((0, u16::MAX - 1), calc_loc(u16::MAX as u32 - 1));
        assert_eq!((0, u16::MAX), calc_loc(u16::MAX as u32));
        assert_eq!((1, 0), calc_loc(u16::MAX as u32 + 1));
        assert_eq!((1, 1), calc_loc(u16::MAX as u32 + 2));
        assert_eq!((u16::MAX, u16::MAX - 1), calc_loc(u32::MAX - 1));
        assert_eq!((u16::MAX, u16::MAX), calc_loc(u32::MAX));
    }
}
