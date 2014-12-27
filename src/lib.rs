use std::{ u16 };
use std::slice::BinarySearchResult::{ Found, NotFound };

use container::Container;

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
/// rb.set(2, true);
/// rb.set(3, true);
/// rb.set(5, true);
/// rb.set(7, true);
/// println!("total bits set to true: {}", rb.cardinality());
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
    pub fn new() -> RoaringBitmap {
        RoaringBitmap { containers: Vec::new(), }
    }
}

impl RoaringBitmap {
    /// Sets the value of a bit at an index `i`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.set(3, true);
    /// assert_eq!(rb[3], true);
    /// ```
    pub fn set(&mut self, index: u32, value: bool) {
        let (key, index) = calc_loc(index);
        let container = match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => &mut self.containers[loc],
            NotFound(loc) => {
                self.containers.insert(loc, Container::new(key));
                &mut self.containers[loc]
            },
        };
        container.set(index, value);
    }

    /// Retrieves the value at index `i`, will never return `None`.
    ///
    /// > TODO: Should this just return a bool, or will it be important that the API is similar to
    /// `std::collections::Bitv`'s?
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.set(1, true);
    /// assert_eq!(rb.get(0), Some(false));
    /// assert_eq!(rb.get(1), Some(true));
    /// assert_eq!(rb.get(100), Some(false));
    ///
    /// // Can also use array indexing
    /// assert_eq!(rb[1], true);
    /// ```
    pub fn get(&self, index: u32) -> Option<bool> {
        let (key, index) = calc_loc(index);
        match self.containers.as_slice().binary_search(|container| key.cmp(&container.key())) {
            Found(loc) => Some(self.containers[loc].get(index)),
            NotFound(_) => Some(false),
        }
    }
}

impl RoaringBitmap {
    /// Returns true if all bits are 0.
    ///
    /// #Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.none(), true);
    ///
    /// rb.set(3, true);
    /// assert_eq!(rb.none(), false);
    /// ```
    pub fn none(&self) -> bool {
        self.cardinality() == 0u32
    }

    /// Returns true if any bit is 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.any(), false);
    ///
    /// rb.set(3, true);
    /// assert_eq!(rb.any(), true);
    /// ```
    pub fn any(&self) -> bool {
        self.cardinality() != 0u32
    }

    /// Returns the number of distinct integers added to the bitmap (e.g., number of bits set).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.cardinality(), 0);
    ///
    /// rb.set(3, true);
    /// assert_eq!(rb.cardinality(), 1);
    ///
    /// rb.set(3, true);
    /// rb.set(4, true);
    /// assert_eq!(rb.cardinality(), 2);
    /// ```
    pub fn cardinality(&self) -> u32 {
        self.containers
            .iter()
            .map(|container| container.cardinality() as u32)
            .fold(0, |sum, cardinality| sum + cardinality)
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl Index<u32, bool> for RoaringBitmap {
    fn index(&self, index: &u32) -> &bool {
        if self.get(*index).unwrap() { &TRUE } else { &FALSE }
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
