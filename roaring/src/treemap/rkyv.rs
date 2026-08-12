impl super::ArchivedRoaringTreemap {
    /// Returns the number of elements in the archived treemap.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.map.iter().map(|(_, rb)| rb.len()).sum()
    }

    /// Checks if the archived treemap is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.iter().all(|(_, rb)| rb.is_empty())
    }

    /// Returns the minimum value in the archived treemap.
    #[must_use]
    pub fn min(&self) -> Option<u64> {
        self.map
            .iter()
            .find_map(|(k, rb)| rb.min().map(|min| ((k.to_native() as u64) << 32) | (min as u64)))
    }

    /// Returns the maximum value in the archived treemap.
    #[must_use]
    pub fn max(&self) -> Option<u64> {
        self.map
            .iter()
            .filter(|(_, rb)| !rb.is_empty())
            .last()
            .and_then(|(k, rb)| rb.max().map(|max| ((k.to_native() as u64) << 32) | (max as u64)))
    }

    /// Checks if the archived treemap contains the given value.
    #[must_use]
    pub fn contains(&self, value: u64) -> bool {
        let hi = (value >> 32) as u32;
        let lo = value as u32;

        let hi_archived = rkyv::Archived::<u32>::from_native(hi);
        self.map.get(&hi_archived).is_some_and(|rb| rb.contains(lo))
    }

    /// Returns the number of integers that are <= value.
    #[must_use]
    pub fn rank(&self, value: u64) -> u64 {
        let hi = (value >> 32) as u32;
        let lo = value as u32;

        let mut rank = 0;
        for (k, rb) in self.map.iter() {
            let k = k.to_native();
            match k.cmp(&hi) {
                std::cmp::Ordering::Less => rank += rb.len(),
                std::cmp::Ordering::Equal => {
                    rank += rb.rank(lo);
                    break;
                }
                std::cmp::Ordering::Greater => break,
            }
        }
        rank
    }

    /// Returns the `n`th integer in the set or `None` if `n >= len()`
    #[must_use]
    pub fn select(&self, mut n: u64) -> Option<u64> {
        for (k, rb) in self.map.iter() {
            let len = rb.len();
            if n < len {
                return rb.select(n as u32).map(|lo| ((k.to_native() as u64) << 32) | (lo as u64));
            }
            n -= len;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rkyv::rancor::Error;

    use crate::{ArchivedRoaringTreemap, RoaringTreemap};

    /// Helper to serialize a `RoaringTreemap` and check that all methods on the
    /// `ArchivedRoaringTreemap` yield the exact same results as the native counterpart.
    fn check_archived_methods(rt: &RoaringTreemap) {
        let bytes = rkyv::to_bytes::<Error>(rt).unwrap();
        let archived = rkyv::access::<ArchivedRoaringTreemap, Error>(&bytes[..]).unwrap();

        assert_eq!(archived.len(), rt.len(), "Length mismatch");
        assert_eq!(archived.is_empty(), rt.is_empty(), "is_empty mismatch");
        assert_eq!(archived.min(), rt.min(), "Min mismatch");
        assert_eq!(archived.max(), rt.max(), "Max mismatch");

        // Dynamically check boundaries
        if let Some(min) = rt.min() {
            assert!(archived.contains(min));
            if min > 0 {
                assert_eq!(archived.contains(min - 1), rt.contains(min - 1));
            }
        }

        if let Some(max) = rt.max() {
            assert!(archived.contains(max));
            if max < u64::MAX {
                assert_eq!(archived.contains(max + 1), rt.contains(max + 1));
            }
        }

        // Check a varied assortment of presence/absence across potential container boundaries
        let check_vals = [0, 1, 100, 4095, 4096, 5000, 10000, 65535, 65536, 100_000, u64::MAX];
        for &v in &check_vals {
            assert_eq!(archived.contains(v), rt.contains(v), "Mismatch at {v}");
            assert_eq!(archived.rank(v), rt.rank(v), "Rank mismatch at {v}");
        }

        // Check select
        if !rt.is_empty() {
            let select_vals = [0, rt.len() / 2, rt.len() - 1];
            for &n in &select_vals {
                assert_eq!(archived.select(n), rt.select(n), "Select mismatch at {n}");
            }
        }
        assert_eq!(archived.select(rt.len()), None);
    }

    #[test]
    fn test_empty() {
        let rt = RoaringTreemap::new();
        check_archived_methods(&rt);
    }

    #[test]
    fn test_basic() {
        let mut rt = RoaringTreemap::new();
        rt.insert(1);
        rt.insert(2);
        rt.insert(100);
        rt.insert(1000);
        rt.insert(u32::MAX as u64 + 10);

        check_archived_methods(&rt);
    }
}
