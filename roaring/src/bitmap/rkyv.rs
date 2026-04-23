use super::{
    container::ArchivedContainer,
    store::{ArchivedArrayStore, ArchivedBitmapStore, ArchivedIntervalStore, ArchivedStore},
};

impl super::ArchivedRoaringBitmap {
    /// Returns the number of elements in the archived bitmap.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.containers.iter().map(ArchivedContainer::len).sum()
    }

    /// Checks if the archived bitmap is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }

    /// Returns the minimum value in the archived bitmap.
    #[must_use]
    pub fn min(&self) -> Option<u32> {
        self.containers
            .first()
            .and_then(|c| c.min().map(|min| ((c.key.to_native() as u32) << 16) | (min as u32)))
    }

    /// Returns the maximum value in the archived bitmap.
    #[must_use]
    pub fn max(&self) -> Option<u32> {
        self.containers
            .last()
            .and_then(|c| c.max().map(|max| ((c.key.to_native() as u32) << 16) | (max as u32)))
    }

    /// Checks if the archived bitmap contains the given value.
    #[must_use]
    pub fn contains(&self, value: u32) -> bool {
        let key = (value >> 16) as u16;
        let index = value as u16;

        self.containers
            .binary_search_by_key(&key, |c| c.key.to_native())
            .is_ok_and(|loc| self.containers[loc].contains(index))
    }

    /// Returns the number of integers that are <= value.
    #[must_use]
    pub fn rank(&self, value: u32) -> u64 {
        let key = (value >> 16) as u16;
        let index = value as u16;

        match self.containers.binary_search_by_key(&key, |c| c.key.to_native()) {
            Ok(loc) => {
                self.containers[loc].rank(index)
                    + self.containers[..loc].iter().map(ArchivedContainer::len).sum::<u64>()
            }
            Err(loc) => self.containers[..loc].iter().map(ArchivedContainer::len).sum::<u64>(),
        }
    }

    /// Returns the `n`th integer in the set or `None` if `n >= len()`
    #[must_use]
    pub fn select(&self, mut n: u32) -> Option<u32> {
        for c in self.containers.iter() {
            let len = c.len();
            if (len as u32) > n {
                return c
                    .select(n)
                    .map(|index| ((c.key.to_native() as u32) << 16) | (index as u32));
            }
            n -= len as u32;
        }
        None
    }
}

impl ArchivedContainer {
    pub fn contains(&self, index: u16) -> bool {
        self.store.contains(index)
    }
    pub fn len(&self) -> u64 {
        self.store.len()
    }
    pub fn min(&self) -> Option<u16> {
        self.store.min()
    }
    pub fn max(&self) -> Option<u16> {
        self.store.max()
    }
    pub fn rank(&self, index: u16) -> u64 {
        self.store.rank(index)
    }
    pub fn select(&self, n: u32) -> Option<u16> {
        self.store.select(n)
    }
}

impl ArchivedStore {
    pub fn contains(&self, index: u16) -> bool {
        match self {
            ArchivedStore::Array(arr) => arr.contains(index),
            ArchivedStore::Bitmap(bit) => bit.contains(index),
            ArchivedStore::Run(run) => run.contains(index),
        }
    }
    pub fn len(&self) -> u64 {
        match self {
            ArchivedStore::Array(arr) => arr.len(),
            ArchivedStore::Bitmap(bit) => bit.len(),
            ArchivedStore::Run(run) => run.len(),
        }
    }
    pub fn min(&self) -> Option<u16> {
        match self {
            ArchivedStore::Array(arr) => arr.min(),
            ArchivedStore::Bitmap(bit) => bit.min(),
            ArchivedStore::Run(run) => run.min(),
        }
    }
    pub fn max(&self) -> Option<u16> {
        match self {
            ArchivedStore::Array(arr) => arr.max(),
            ArchivedStore::Bitmap(bit) => bit.max(),
            ArchivedStore::Run(run) => run.max(),
        }
    }
    pub fn rank(&self, index: u16) -> u64 {
        match self {
            ArchivedStore::Array(arr) => arr.rank(index),
            ArchivedStore::Bitmap(bit) => bit.rank(index),
            ArchivedStore::Run(run) => run.rank(index),
        }
    }
    pub fn select(&self, n: u32) -> Option<u16> {
        match self {
            ArchivedStore::Array(arr) => arr.select(n),
            ArchivedStore::Bitmap(bit) => bit.select(n),
            ArchivedStore::Run(run) => run.select(n),
        }
    }
}

impl ArchivedArrayStore {
    pub fn contains(&self, index: u16) -> bool {
        self.vec.as_slice().binary_search_by_key(&index, |x| x.to_native()).is_ok()
    }
    pub fn len(&self) -> u64 {
        self.vec.len() as u64
    }
    pub fn min(&self) -> Option<u16> {
        self.vec.first().map(|x| x.to_native())
    }
    pub fn max(&self) -> Option<u16> {
        self.vec.last().map(|x| x.to_native())
    }
    pub fn rank(&self, index: u16) -> u64 {
        match self.vec.as_slice().binary_search_by_key(&index, |x| x.to_native()) {
            Ok(loc) => loc as u64 + 1,
            Err(loc) => loc as u64,
        }
    }
    pub fn select(&self, n: u32) -> Option<u16> {
        self.vec.get(n as usize).map(|x| x.to_native())
    }
}

impl ArchivedBitmapStore {
    pub fn contains(&self, index: u16) -> bool {
        let key = (index / 64) as usize;
        let bit = index % 64;
        (self.bits[key].to_native() & (1 << bit)) != 0
    }
    pub const fn len(&self) -> u64 {
        self.len.to_native()
    }
    pub fn min(&self) -> Option<u16> {
        self.bits
            .iter()
            .enumerate()
            .find(|&(_, bit)| bit.to_native() != 0)
            .map(|(index, bit)| (index * 64 + bit.to_native().trailing_zeros() as usize) as u16)
    }
    pub fn max(&self) -> Option<u16> {
        self.bits.iter().enumerate().rev().find(|&(_, bit)| bit.to_native() != 0).map(
            |(index, bit)| (index * 64 + (63 - bit.to_native().leading_zeros() as usize)) as u16,
        )
    }
    pub fn rank(&self, index: u16) -> u64 {
        let key = (index / 64) as usize;
        let bit = index % 64;

        self.bits[..key].iter().map(|v| v.to_native().count_ones() as u64).sum::<u64>()
            + (self.bits[key].to_native() << (63 - bit)).count_ones() as u64
    }
    pub fn select(&self, mut n: u32) -> Option<u16> {
        for (key, word) in self.bits.iter().enumerate() {
            let word = word.to_native();
            let count = word.count_ones();
            if n < count {
                let mut w = word;
                for _ in 0..n {
                    w &= w - 1;
                }
                return Some((key * 64 + w.trailing_zeros() as usize) as u16);
            }
            n -= count;
        }
        None
    }
}

impl ArchivedIntervalStore {
    pub fn contains(&self, index: u16) -> bool {
        use core::cmp::Ordering;

        self.0
            .as_slice()
            .binary_search_by(|iv| {
                let start = iv.start.to_native();
                let end = iv.end.to_native();
                if index < start {
                    Ordering::Greater
                } else if index > end {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .is_ok()
    }
    pub fn len(&self) -> u64 {
        self.0.iter().map(|iv| (iv.end.to_native() - iv.start.to_native()) as u64 + 1).sum()
    }
    pub fn min(&self) -> Option<u16> {
        self.0.first().map(|iv| iv.start.to_native())
    }
    pub fn max(&self) -> Option<u16> {
        self.0.last().map(|iv| iv.end.to_native())
    }
    pub fn rank(&self, index: u16) -> u64 {
        let mut rank = 0;
        for iv in self.0.iter() {
            let start = iv.start.to_native();
            let end = iv.end.to_native();
            if end <= index {
                rank += (end - start) as u64 + 1;
            } else if start <= index {
                rank += (index - start) as u64 + 1;
                break;
            } else {
                break;
            }
        }
        rank
    }
    pub fn select(&self, mut n: u32) -> Option<u16> {
        for iv in self.0.iter() {
            let start = iv.start.to_native();
            let end = iv.end.to_native();
            let len = (end - start) as u32 + 1;
            if n < len {
                return Some(start + n as u16);
            }
            n -= len;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rkyv::rancor::Error;

    use crate::{ArchivedRoaringBitmap, RoaringBitmap};

    /// Helper to serialize a `RoaringBitmap` and check that all methods on the
    /// `ArchivedRoaringBitmap` yield the exact same results as the native counterpart.
    fn check_archived_methods(rb: &RoaringBitmap) {
        let bytes = rkyv::to_bytes::<Error>(rb).unwrap();
        let archived = rkyv::access::<ArchivedRoaringBitmap, Error>(&bytes[..]).unwrap();

        assert_eq!(archived.len(), rb.len(), "Length mismatch");
        assert_eq!(archived.is_empty(), rb.is_empty(), "is_empty mismatch");
        assert_eq!(archived.min(), rb.min(), "Min mismatch");
        assert_eq!(archived.max(), rb.max(), "Max mismatch");

        // Dynamically check boundaries
        if let Some(min) = rb.min() {
            assert!(archived.contains(min));
            if min > 0 {
                assert_eq!(archived.contains(min - 1), rb.contains(min - 1));
            }
        }

        if let Some(max) = rb.max() {
            assert!(archived.contains(max));
            if max < u32::MAX {
                assert_eq!(archived.contains(max + 1), rb.contains(max + 1));
            }
        }

        // Check a varied assortment of presence/absence across potential container boundaries
        let check_vals = [0, 1, 100, 4095, 4096, 5000, 10000, 65535, 65536, 100_000, u32::MAX];
        for &v in &check_vals {
            assert_eq!(archived.contains(v), rb.contains(v), "Mismatch at {v}");
            assert_eq!(archived.rank(v), rb.rank(v), "Rank mismatch at {v}");
        }

        // Check select
        if !rb.is_empty() {
            let select_vals = [0, (rb.len() / 2) as u32, (rb.len() - 1) as u32];
            for &n in &select_vals {
                assert_eq!(archived.select(n), rb.select(n), "Select mismatch at {n}");
            }
        }
        assert_eq!(archived.select(rb.len() as u32), None);
    }

    #[test]
    fn test_empty() {
        let rb = RoaringBitmap::new();
        check_archived_methods(&rb);
    }

    #[test]
    fn test_array_store() {
        // Less than 4096 elements creates an ArrayStore
        let mut rb = RoaringBitmap::new();
        rb.insert(1);
        rb.insert(2);
        rb.insert(100);
        rb.insert(1000);

        check_archived_methods(&rb);
    }

    #[test]
    fn test_bitmap_store() {
        let mut rb = RoaringBitmap::new();
        // Inserting > 4096 scattered elements forces a BitmapStore
        for i in (0..10_000).step_by(2) {
            rb.insert(i);
        }

        check_archived_methods(&rb);
    }

    #[test]
    fn test_interval_store() {
        let mut rb = RoaringBitmap::new();
        rb.insert_range(100..=5000);
        // Calling optimize on a contiguous block converts it to an IntervalStore (Run container)
        rb.optimize();

        check_archived_methods(&rb);
    }

    #[test]
    fn test_mixed_stores() {
        let mut rb = RoaringBitmap::new();

        // Container 0: ArrayStore
        rb.insert(1);
        rb.insert(10);

        // Container 1: BitmapStore
        let offset = 1 << 16;
        for i in (0..10_000).step_by(2) {
            rb.insert(offset + i);
        }

        // Container 2: IntervalStore
        let offset2 = 2 << 16;
        rb.insert_range(offset2 + 100..=offset2 + 5000);
        rb.optimize();

        check_archived_methods(&rb);
    }
}
