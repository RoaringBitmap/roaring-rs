use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::ops::{Bound, RangeBounds};

use crate::RoaringBitmap;
use crate::RoaringTreemap;

use super::util;

impl RoaringTreemap {
    /// Creates an empty `RoaringTreemap`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// let mut rb = RoaringTreemap::new();
    /// ```
    pub fn new() -> RoaringTreemap {
        RoaringTreemap { map: BTreeMap::new() }
    }

    /// Adds a value to the set. Returns `true` if the value was not already present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// assert_eq!(rb.insert(3), true);
    /// assert_eq!(rb.insert(3), false);
    /// assert_eq!(rb.contains(3), true);
    /// ```
    pub fn insert(&mut self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        self.map.entry(hi).or_insert_with(RoaringBitmap::new).insert(lo)
    }

    /// Pushes `value` in the treemap only if it is greater than the current maximum value.
    ///
    /// Returns whether the value was inserted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// assert!(rb.push(1));
    /// assert!(rb.push(3));
    /// assert_eq!(rb.push(3), false);
    /// assert!(rb.push(5));
    ///
    /// assert_eq!(rb.iter().collect::<Vec<u64>>(), vec![1, 3, 5]);
    /// ```
    pub fn push(&mut self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        self.map.entry(hi).or_insert_with(RoaringBitmap::new).push(lo)
    }

    /// Removes a value from the set. Returns `true` if the value was present in the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// rb.insert(3);
    /// assert_eq!(rb.remove(3), true);
    /// assert_eq!(rb.remove(3), false);
    /// assert_eq!(rb.contains(3), false);
    /// ```
    pub fn remove(&mut self, value: u64) -> bool {
        let (hi, lo) = util::split(value);
        match self.map.entry(hi) {
            Entry::Vacant(_) => false,
            Entry::Occupied(mut ent) => {
                if ent.get_mut().remove(lo) {
                    if ent.get().is_empty() {
                        ent.remove();
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Removes a range of values.
    /// Returns the number of removed values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// rb.insert(2);
    /// rb.insert(3);
    /// assert_eq!(rb.remove_range(2..4), 2);
    /// ```
    pub fn remove_range<R>(&mut self, range: R) -> u64
    where
        R: RangeBounds<u64>,
    {
        // Get Range's inclusive start and end point.
        let mut start: u64 = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Unbounded => 0,
            _ => panic!("Should never be called (remove_range start with Excluded)"),
        };
        let end: u64 = match range.end_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(0) => {
                // Make this range empty with start > end
                start = u64::MAX;
                0
            }
            Bound::Excluded(&i) => i - 1,
            Bound::Unbounded => u64::MAX,
        };
        let start = start;

        if end < start {
            return 0;
        }

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        let mut keys_to_remove = Vec::new();
        let mut removed = 0;

        for (&key, rb) in &mut self.map {
            if key >= start_container_key && key <= end_container_key {
                let a = if key == start_container_key { start_index } else { 0 };
                let b = if key == end_container_key { end_index } else { u32::max_value() };
                removed += rb.remove_range(a..=b);
                if rb.is_empty() {
                    keys_to_remove.push(key);
                }
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }

        removed
    }

    /// Returns `true` if this set contains the specified integer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
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
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
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
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// assert_eq!(rb.is_empty(), true);
    ///
    /// rb.insert(3);
    /// assert_eq!(rb.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.values().all(RoaringBitmap::is_empty)
    }

    /// Returns the number of distinct integers added to the set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
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
        self.map.values().map(RoaringBitmap::len).sum()
    }

    /// Returns the minimum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// assert_eq!(rb.min(), None);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.min(), Some(3));
    /// ```
    pub fn min(&self) -> Option<u64> {
        self.map
            .iter()
            .find(|&(_, rb)| rb.min().is_some())
            .map(|(k, rb)| util::join(*k, rb.min().unwrap()))
    }

    /// Returns the maximum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
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
            .find(|&(_, rb)| rb.max().is_some())
            .map(|(k, rb)| util::join(*k, rb.max().unwrap()))
    }
}

impl Default for RoaringTreemap {
    fn default() -> RoaringTreemap {
        RoaringTreemap::new()
    }
}

impl Clone for RoaringTreemap {
    fn clone(&self) -> Self {
        RoaringTreemap { map: self.map.clone() }
    }

    fn clone_from(&mut self, other: &Self) {
        self.map.clone_from(&other.map);
    }
}
