use std::ops::{Bound, RangeBounds};

use crate::RoaringBitmap;

use super::container::Container;
use super::util;

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
        RoaringBitmap {
            containers: Vec::new(),
        }
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
        let (key, index) = util::split(value);
        let container = match self.containers.binary_search_by_key(&key, |c| c.key) {
            Ok(loc) => &mut self.containers[loc],
            Err(loc) => {
                self.containers.insert(loc, Container::new(key));
                &mut self.containers[loc]
            }
        };
        container.insert(index)
    }

    /// Inserts a range of values from the set.
    /// Returns the number of inserted values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert_range(2..4);
    /// assert!(rb.contains(2));
    /// assert!(rb.contains(3));
    /// assert!(!rb.contains(4));
    /// ```
    pub fn insert_range<R: RangeBounds<u32>>(&mut self, range: R) -> u64 {
        let start = match range.start_bound() {
            Bound::Included(value) => *value,
            Bound::Excluded(value) => match value.checked_add(1) {
                Some(value) => value,
                None => return 0,
            },
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(value) => match value.checked_add(1) {
                Some(value) => value,
                None => return 0,
            },
            Bound::Excluded(value) => *value,
            Bound::Unbounded => u32::max_value(),
        };

        if end.saturating_sub(start) == 0 {
            return 0;
        }

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        // Find the container index for start_container_key
        let start_i = match self
            .containers
            .binary_search_by_key(&start_container_key, |c| c.key)
        {
            Ok(loc) => loc,
            Err(loc) => {
                self.containers
                    .insert(loc, Container::new(start_container_key));
                loc
            }
        };

        // If the end range value is in the same container, just call into
        // the one container.
        if start_container_key == end_container_key {
            return self.containers[start_i].insert_range(start_index..end_index);
        }

        // For the first container, insert start_index..u16::MAX, with
        // subsequent containers inserting 0..MAX.
        //
        // The last container (end_container_key) is handled explicitly outside
        // the loop.
        let mut low = start_index;
        let mut inserted = 0;

        // Walk through the containers until the container for end_container_key
        let end_i = usize::from(end_container_key - start_container_key);
        for i in start_i..end_i {
            // Fetch (or upsert) the container for i
            let c = match self.containers.get_mut(i) {
                Some(c) => c,
                None => {
                    // For each i, the container key is start_container + i in
                    // the upper u8 of the u16.
                    let key = start_container_key + ((1 << 8) * i) as u16;
                    self.containers.insert(i, Container::new(key));
                    &mut self.containers[i]
                }
            };

            // Insert the range subset for this container
            inserted += c.insert_range(low..u16::MAX);

            // After the first container, always fill the containers.
            low = 0;
        }

        // Handle the last container
        let c = match self.containers.get_mut(end_i) {
            Some(c) => c,
            None => {
                let (key, _) = util::split(start);
                self.containers.insert(end_i, Container::new(key));
                &mut self.containers[end_i]
            }
        };
        c.insert_range(0..end_index);

        inserted
    }

    /// Adds a value to the set.
    /// The value **must** be greater or equal to the maximum value in the set.
    ///
    /// This method can be faster than `insert` because it skips the binary searches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.push(1);
    /// rb.push(3);
    /// rb.push(3);
    /// rb.push(5);
    ///
    /// assert_eq!(rb.iter().collect::<Vec<u32>>(), vec![1, 3, 5]);
    /// ```
    pub fn push(&mut self, value: u32) {
        let (key, index) = util::split(value);
        match self.containers.last() {
            Some(container) => {
                if container.key != key {
                    self.containers
                        .insert(self.containers.len(), Container::new(key));
                }
            }
            None => {
                self.containers
                    .insert(self.containers.len(), Container::new(key));
            }
        }
        let last = self.containers.last_mut().unwrap();
        assert!(last.key <= key);
        assert!(last.len == 0 || last.max() <= index);
        last.push(index)
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
        let (key, index) = util::split(value);
        match self.containers.binary_search_by_key(&key, |c| c.key) {
            Ok(loc) => {
                if self.containers[loc].remove(index) {
                    if self.containers[loc].len == 0 {
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

    /// Removes a range of values from the set.
    /// Returns the number of removed values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(2);
    /// rb.insert(3);
    /// assert_eq!(rb.remove_range(2..4), 2);
    /// ```
    pub fn remove_range<R: RangeBounds<u32>>(&mut self, range: R) -> u64 {
        let start = match range.start_bound() {
            Bound::Included(value) => *value,
            Bound::Excluded(value) => match value.checked_add(1) {
                Some(value) => value,
                None => return 0,
            },
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(value) => match value.checked_add(1) {
                Some(value) => value,
                None => return 0,
            },
            Bound::Excluded(value) => *value,
            Bound::Unbounded => u32::max_value(),
        };

        if end.saturating_sub(start) == 0 {
            return 0;
        }
        // inclusive bounds for start and end
        let (start_hi, start_lo) = util::split(start);
        let (end_hi, end_lo) = util::split(end - 1);
        let mut index = 0;
        let mut result = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            if key >= start_hi && key <= end_hi {
                let a = if key == start_hi {
                    u32::from(start_lo)
                } else {
                    0
                };
                let b = if key == end_hi {
                    u32::from(end_lo) + 1 // make it exclusive
                } else {
                    u32::from(u16::max_value()) + 1
                };
                // remove container?
                if a == 0 && b == u32::from(u16::max_value()) + 1 {
                    result += self.containers[index].len;
                    self.containers.remove(index);
                    continue;
                } else {
                    result += self.containers[index].remove_range(a, b);
                    if self.containers[index].len == 0 {
                        self.containers.remove(index);
                        continue;
                    }
                }
            }
            index += 1;
        }
        result
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
        let (key, index) = util::split(value);
        match self.containers.binary_search_by_key(&key, |c| c.key) {
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
    pub fn len(&self) -> u64 {
        self.containers.iter().map(|container| container.len).sum()
    }

    /// Returns the minimum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.min(), None);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.min(), Some(3));
    /// ```
    pub fn min(&self) -> Option<u32> {
        self.containers
            .first()
            .map(|head| util::join(head.key, head.min()))
    }

    /// Returns the maximum value in the set (if the set is non-empty).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.max(), None);
    ///
    /// rb.insert(3);
    /// rb.insert(4);
    /// assert_eq!(rb.max(), Some(4));
    /// ```
    pub fn max(&self) -> Option<u32> {
        self.containers
            .last()
            .map(|tail| util::join(tail.key, tail.max()))
    }
}

impl Default for RoaringBitmap {
    fn default() -> RoaringBitmap {
        RoaringBitmap::new()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn insert_range(r: Range<u32>, checks: Vec<u32>) {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(r.clone());
        if r.end > r.start {
            assert_eq!(inserted as u32, r.end - r.start);
        } else {
            assert_eq!(inserted as u32, 0);
        }

        // Assert all values in the range are present
        for i in r.clone() {
            assert!(b.contains(i as u32), "does not contain {}", i);
        }

        // Run the check values looking for any false positives
        for i in checks {
            let bitmap_has = b.contains(i);
            let range_has = r.contains(&i);
            assert!(
                bitmap_has == range_has,
                "value {} in bitmap={} and range={}",
                i, bitmap_has, range_has,
            );
        }
    }

    #[test]
    fn test_insert_range_same_container() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(1..5);
        assert_eq!(inserted, 4);

        for i in 1..5 {
            assert!(b.contains(i));
        }
    }

    #[test]
    fn test_insert_range_pre_populated() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(1..20_000);
        assert_eq!(inserted, 19_999);

        let inserted = b.insert_range(1..20_000);
        assert_eq!(inserted, 0);
    }

    #[test]
    fn test_insert_max_u32() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert(u32::MAX);
         // We are allowed to add u32::MAX
        assert!(inserted);
    }

    #[test]
    fn test_insert_range_zero_inclusive() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(0..=0);
        // `insert_range(value..=value)` appears equivalent to `insert(value)`
        assert_eq!(inserted, 1);
        assert!(b.contains(0), "does not contain {}", 0);
    }

    #[test]
    fn test_insert_range_max_u32_inclusive() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(u32::MAX..=u32::MAX);
        // But not equivalent for u32::MAX
        assert_eq!(inserted, 1); // Fails - left: 0, right: 1
        assert!(b.contains(u32::MAX), "does not contain {}", u32::MAX);
    }

    #[test]
    fn test_insert_all_u32() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(0..u32::MAX); // Largest possible range seemingly allowed
        assert_eq!(inserted, u32::MAX as u64); // Still not bigger than u32::MAX
    }
}
