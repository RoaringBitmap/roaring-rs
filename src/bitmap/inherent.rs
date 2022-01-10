use std::ops::RangeBounds;

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
        RoaringBitmap { containers: Vec::new() }
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was absent from the set.
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

    /// Search for the specific container by the given key.
    /// Create a new container if not exist.
    ///
    /// Return the index of the target container.
    fn find_container_by_key(&mut self, key: u16) -> usize {
        match self.containers.binary_search_by_key(&key, |c| c.key) {
            Ok(loc) => loc,
            Err(loc) => {
                self.containers.insert(loc, Container::new(key));
                loc
            }
        }
    }

    /// Inserts a range of values.
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
    pub fn insert_range<R>(&mut self, range: R) -> u64
    where
        R: RangeBounds<u32>,
    {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return 0,
        };

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        // Find the container index for start_container_key
        let first_index = self.find_container_by_key(start_container_key);

        // If the end range value is in the same container, just call into
        // the one container.
        if start_container_key == end_container_key {
            return self.containers[first_index].insert_range(start_index..=end_index);
        }

        // For the first container, insert start_index..=u16::MAX, with
        // subsequent containers inserting 0..MAX.
        //
        // The last container (end_container_key) is handled explicitly outside
        // the loop.
        let mut low = start_index;
        let mut inserted = 0;

        for i in start_container_key..end_container_key {
            let index = self.find_container_by_key(i);

            // Insert the range subset for this container
            inserted += self.containers[index].insert_range(low..=u16::MAX);

            // After the first container, always fill the containers.
            low = 0;
        }

        // Handle the last container
        let last_index = self.find_container_by_key(end_container_key);

        inserted += self.containers[last_index].insert_range(0..=end_index);

        inserted
    }

    /// Pushes `value` in the bitmap only if it is greater than the current maximum value.
    ///
    /// Returns whether the value was inserted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert!(rb.push(1));
    /// assert!(rb.push(3));
    /// assert_eq!(rb.push(3), false);
    /// assert!(rb.push(5));
    ///
    /// assert_eq!(rb.iter().collect::<Vec<u32>>(), vec![1, 3, 5]);
    /// ```
    pub fn push(&mut self, value: u32) -> bool {
        let (key, index) = util::split(value);

        match self.containers.last_mut() {
            Some(container) if container.key == key => container.push(index),
            Some(container) if container.key > key => false,
            _otherwise => {
                let mut container = Container::new(key);
                container.push(index);
                self.containers.push(container);
                true
            }
        }
    }

    /// Pushes `value` in the bitmap only if it is greater than the current maximum value.
    /// It is up to the caller to have validated index > self.max()
    pub(crate) fn push_unchecked(&mut self, value: u32) {
        let (key, index) = util::split(value);

        match self.containers.last_mut() {
            Some(container) if container.key == key => container.push_unchecked(index),
            _otherwise => {
                let mut container = Container::new(key);
                container.push_unchecked(index);
                self.containers.push(container);
            }
        }
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

    /// Removes a range of values.
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
    pub fn remove_range<R>(&mut self, range: R) -> u64
    where
        R: RangeBounds<u32>,
    {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return 0,
        };

        let (start_container_key, start_index) = util::split(start);
        let (end_container_key, end_index) = util::split(end);

        let mut index = 0;
        let mut removed = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            if key >= start_container_key && key <= end_container_key {
                let a = if key == start_container_key { start_index } else { 0 };
                let b = if key == end_container_key { end_index } else { u16::MAX };
                removed += self.containers[index].remove_range(a..=b);
                if self.containers[index].len == 0 {
                    self.containers.remove(index);
                    continue;
                }
            }
            index += 1;
        }
        removed
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
        self.containers.first().and_then(|tail| tail.min().map(|min| util::join(tail.key, min)))
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
        self.containers.last().and_then(|tail| tail.max().map(|max| util::join(tail.key, max)))
    }
}

impl Default for RoaringBitmap {
    fn default() -> RoaringBitmap {
        RoaringBitmap::new()
    }
}

impl Clone for RoaringBitmap {
    fn clone(&self) -> Self {
        RoaringBitmap { containers: self.containers.clone() }
    }

    fn clone_from(&mut self, other: &Self) {
        self.containers.clone_from(&other.containers);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn qc_insert_range(r: Range<u32>, checks: Vec<u32>) {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(r.clone());
        if r.end > r.start {
            assert_eq!(inserted, r.end as u64 - r.start as u64);
        } else {
            assert_eq!(inserted, 0);
        }

        // Assert all values in the range are present
        for i in r.clone() {
            assert!(b.contains(i as u32), "does not contain {}", i);
        }

        // Run the check values looking for any false positives
        for i in checks {
            let bitmap_has = b.contains(i);
            let range_has = r.contains(&i);
            assert_eq!(
                bitmap_has, range_has,
                "value {} in bitmap={} and range={}",
                i, bitmap_has, range_has
            );
        }
    }

    #[test]
    fn test_insert_remove_range_same_container() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(1..5);
        assert_eq!(inserted, 4);

        for i in 1..5 {
            assert!(b.contains(i));
        }

        let removed = b.remove_range(2..10);
        assert_eq!(removed, 3);
        assert!(b.contains(1));
        for i in 2..5 {
            assert!(!b.contains(i));
        }
    }

    #[test]
    fn test_insert_remove_range_pre_populated() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(1..20_000);
        assert_eq!(inserted, 19_999);

        let removed = b.remove_range(10_000..21_000);
        assert_eq!(removed, 10_000);

        let inserted = b.insert_range(1..20_000);
        assert_eq!(inserted, 10_000);
    }

    #[test]
    fn test_insert_max_u32() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert(u32::MAX);
        // We are allowed to add u32::MAX
        assert!(inserted);
    }

    #[test]
    fn test_insert_remove_across_container() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(u16::MAX as u32..=u16::MAX as u32 + 1);
        assert_eq!(inserted, 2);

        assert_eq!(b.containers.len(), 2);

        let removed = b.remove_range(u16::MAX as u32 + 1..=u16::MAX as u32 + 1);
        assert_eq!(removed, 1);

        assert_eq!(b.containers.len(), 1);
    }

    #[test]
    fn test_insert_remove_single_element() {
        let mut b = RoaringBitmap::new();
        let inserted = b.insert_range(u16::MAX as u32 + 1..=u16::MAX as u32 + 1);
        assert_eq!(inserted, 1);

        assert_eq!(b.containers[0].len, 1);
        assert_eq!(b.containers.len(), 1);

        let removed = b.remove_range(u16::MAX as u32 + 1..=u16::MAX as u32 + 1);
        assert_eq!(removed, 1);

        assert_eq!(b.containers.len(), 0);
    }

    #[test]
    fn test_insert_remove_range_multi_container() {
        let mut bitmap = RoaringBitmap::new();
        assert_eq!(bitmap.insert_range(0..((1_u32 << 16) + 1)), (1_u64 << 16) + 1);
        assert_eq!(bitmap.containers.len(), 2);
        assert_eq!(bitmap.containers[0].key, 0);
        assert_eq!(bitmap.containers[1].key, 1);
        assert_eq!(bitmap.insert_range(0..((1_u32 << 16) + 1)), 0);

        assert!(bitmap.insert((1_u32 << 16) * 4));
        assert_eq!(bitmap.containers.len(), 3);
        assert_eq!(bitmap.containers[2].key, 4);

        assert_eq!(bitmap.remove_range(((1_u32 << 16) * 3)..=((1_u32 << 16) * 4)), 1);
        assert_eq!(bitmap.containers.len(), 2);
    }

    #[test]
    fn insert_range_single() {
        let mut bitmap = RoaringBitmap::new();
        assert_eq!(bitmap.insert_range((1_u32 << 16)..(2_u32 << 16)), 1_u64 << 16);
        assert_eq!(bitmap.containers.len(), 1);
        assert_eq!(bitmap.containers[0].key, 1);
    }
}
