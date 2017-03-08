use RoaringBitmap64;
use RoaringBitmap;

use super::util;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

impl RoaringBitmap64 {
    /// Creates an empty `RoaringBitmap64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    /// let mut rb = RoaringBitmap64::new();
    /// ```
    pub fn new() -> RoaringBitmap64 {
        RoaringBitmap64 { map: BTreeMap::new() }
    }

    /// Adds a value to the set. Returns `true` if the value was not already present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// assert_eq!(rb.insert(3), true);
    /// assert_eq!(rb.insert(3), false);
    /// assert_eq!(rb.contains(3), true);
    /// ```
    pub fn insert(&mut self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        self.map.entry(hi).or_insert_with(RoaringBitmap::new).insert(lo)
    }

    /// Removes a value from the set. Returns `true` if the value was present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// rb.insert(3);
    /// assert_eq!(rb.remove(3), true);
    /// assert_eq!(rb.remove(3), false);
    /// assert_eq!(rb.contains(3), false);
    /// ```
    pub fn remove(&mut self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        match self.map.entry(hi) {
            Entry::Vacant(_) => false,
            Entry::Occupied(mut ent) => ent.get_mut().remove(lo),
        }
    }

    /// Returns `true` if this set contains the specified integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(0), false);
    /// assert_eq!(rb.contains(1), true);
    /// assert_eq!(rb.contains(100), false);
    /// ```
    pub fn contains(&self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        match self.map.get(&hi) {
            None => false,
            Some(r) => r.contains(lo),
        }
    }

    /// Clears all integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// rb.insert(1);
    /// assert_eq!(rb.contains(1), true);
    /// rb.clear();
    /// assert_eq!(rb.contains(1), false);
    /// ```
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns `true` if there are no integers in this set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// assert_eq!(rb.is_empty(), true);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map
            .values()
            .all(|r| r.is_empty())
    }

    /// Returns the number of distinct integers added to the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// assert_eq!(rb.len(), 0);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.len(), 1);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.len(), 2);
    /// ```
    pub fn len(&self) -> u64 {
        self.map
            .values()
            .map(|r| r.len())
            .sum()
    }

    /// Returns the minimum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// assert_eq!(rb.min(), None);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.min(), Some(3));
    /// ```
    pub fn min(&self) -> Option<u64> {
        self.map
            .iter()
            .filter(|&(_, r)| r.min().is_some())
            .nth(0)
            .map(|(k, r)| util::join(*k, r.min().unwrap()))
    }


    /// Returns the maximum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap64;
    ///
    /// let mut rb = RoaringBitmap64::new();
    /// assert_eq!(rb.max(), None);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.max(), Some(4));
    /// ```
    pub fn max(&self) -> Option<u64> {
        self.map
            .iter()
            .rev()
            .filter(|&(_, r)| r.max().is_some())
            .last()
            .map(|(k, r)| util::join(*k, r.max().unwrap()))
        // self.containers
        //     .last()
        //     .map(|tail| util::join(tail.key, tail.max()))
    }
}

impl Default for RoaringBitmap64 {
    fn default() -> RoaringBitmap64 {
        RoaringBitmap64::new()
    }
}
