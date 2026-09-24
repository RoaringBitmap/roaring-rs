use alloc::collections::btree_map;

use crate::{RoaringBitmap, RoaringTreemap};

/// A view into a single partition of a [`RoaringTreemap`], obtained from
/// [`RoaringTreemap::bitmap_entry`].
///
/// This is the entry-point style access for per-partition in-place
/// mutation: it lets callers locate, mutate, insert, or take out the
/// [`RoaringBitmap`] of one partition without rebuilding the treemap
/// around it.
pub enum BitmapEntry<'a> {
    /// The partition is absent.
    Vacant(VacantBitmapEntry<'a>),
    /// The partition exists.
    Occupied(OccupiedBitmapEntry<'a>),
}

/// A vacant partition view, obtained from [`RoaringTreemap::bitmap_entry`].
///
/// Use [`VacantBitmapEntry::insert`] to create that partition.
pub struct VacantBitmapEntry<'a> {
    inner: btree_map::VacantEntry<'a, u32, RoaringBitmap>,
}

/// An occupied partition view, obtained from [`RoaringTreemap::bitmap_entry`].
///
/// Use [`OccupiedBitmapEntry::get_mut`] to borrow its bitmap mutably and
/// [`OccupiedBitmapEntry::remove`] to take it out.
pub struct OccupiedBitmapEntry<'a> {
    inner: btree_map::OccupiedEntry<'a, u32, RoaringBitmap>,
}

impl<'a> VacantBitmapEntry<'a> {
    /// Inserts a bitmap into the vacant partition and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::{RoaringBitmap, RoaringTreemap};
    /// use roaring::treemap::BitmapEntry;
    ///
    /// let mut treemap = RoaringTreemap::new();
    /// match treemap.bitmap_entry(3) {
    ///     BitmapEntry::Vacant(entry) => {
    ///         let bitmap = entry.insert(RoaringBitmap::from([1]));
    ///         bitmap.insert(2);
    ///     }
    ///     BitmapEntry::Occupied(_) => unreachable!(),
    /// }
    /// assert_eq!(treemap.len(), 2);
    /// ```
    pub fn insert(self, bitmap: RoaringBitmap) -> &'a mut RoaringBitmap {
        self.inner.insert(bitmap)
    }
}

impl OccupiedBitmapEntry<'_> {
    /// Returns a mutable reference to the partition's bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use roaring::treemap::BitmapEntry;
    ///
    /// let mut treemap = RoaringTreemap::new();
    /// treemap.insert(1);
    /// if let BitmapEntry::Occupied(mut entry) = treemap.bitmap_entry(0) {
    ///     entry.get_mut().insert(2);
    /// }
    /// assert_eq!(treemap.len(), 2);
    /// ```
    pub fn get_mut(&mut self) -> &mut RoaringBitmap {
        self.inner.get_mut()
    }

    /// Takes the partition's bitmap out of the treemap, removing the partition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use roaring::treemap::BitmapEntry;
    ///
    /// let mut treemap = RoaringTreemap::new();
    /// treemap.insert(1);
    /// if let BitmapEntry::Occupied(entry) = treemap.bitmap_entry(0) {
    ///     assert_eq!(entry.remove().len(), 1);
    /// }
    /// assert!(treemap.is_empty());
    /// ```
    pub fn remove(self) -> RoaringBitmap {
        self.inner.remove()
    }
}

impl RoaringTreemap {
    /// Returns a view into the partition with the given prefix (the 32 most significant bits).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::{RoaringBitmap, RoaringTreemap};
    /// use roaring::treemap::BitmapEntry;
    ///
    /// let mut treemap = RoaringTreemap::new();
    ///
    /// // Vacant: insert a new partition.
    /// match treemap.bitmap_entry(0) {
    ///     BitmapEntry::Vacant(entry) => { entry.insert(RoaringBitmap::from([1])); }
    ///     BitmapEntry::Occupied(_) => unreachable!(),
    /// }
    ///
    /// // Occupied: mutate the partition in place.
    /// match treemap.bitmap_entry(0) {
    ///     BitmapEntry::Occupied(mut entry) => { entry.get_mut().insert(2); }
    ///     BitmapEntry::Vacant(_) => unreachable!(),
    /// }
    /// assert_eq!(treemap.len(), 2);
    /// ```
    pub fn bitmap_entry(&mut self, prefix: u32) -> BitmapEntry<'_> {
        match self.map.entry(prefix) {
            btree_map::Entry::Vacant(inner) => BitmapEntry::Vacant(VacantBitmapEntry { inner }),
            btree_map::Entry::Occupied(inner) => {
                BitmapEntry::Occupied(OccupiedBitmapEntry { inner })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{RoaringBitmap, RoaringTreemap};
    use proptest::prelude::*;

    use super::BitmapEntry;

    proptest! {
        #[test]
        fn occupied_entry_remove_and_insert(
            treemap in RoaringTreemap::arbitrary(),
        ) {
            let mut treemap = treemap;
            let Some(prefix) = treemap.iter().next().map(|v| (v >> 32) as u32) else {
                // empty treemap: a lookup is vacant
                return Ok(());
            };

            // Occupied: take the bitmap out, then put it back.
            let expected = treemap.clone();
            let bitmap = match treemap.bitmap_entry(prefix) {
                BitmapEntry::Occupied(entry) => entry.remove(),
                BitmapEntry::Vacant(_) => unreachable!("prefix exists"),
            };
            prop_assert!(matches!(treemap.bitmap_entry(prefix), BitmapEntry::Vacant(_)));

            match treemap.bitmap_entry(prefix) {
                BitmapEntry::Vacant(entry) => {
                    entry.insert(bitmap);
                }
                BitmapEntry::Occupied(_) => unreachable!("prefix was removed"),
            }
            prop_assert_eq!(treemap, expected);
        }

        #[test]
        fn vacant_entry_insert(
            treemap in RoaringTreemap::arbitrary(),
            prefix in any::<u32>(),
            bitmap in RoaringBitmap::arbitrary(),
        ) {
            let mut treemap = treemap;
            // Start from a guaranteed-vacant prefix (drop it if present).
            if let BitmapEntry::Occupied(entry) = treemap.bitmap_entry(prefix) {
                entry.remove();
            }

            // A vacant lookup inserts nothing.
            let before = treemap.clone();
            match treemap.bitmap_entry(prefix) {
                BitmapEntry::Vacant(_) => {}
                BitmapEntry::Occupied(_) => unreachable!("prefix was removed"),
            }
            prop_assert_eq!(&treemap, &before);

            // Inserting through the vacant entry makes it occupied with that bitmap.
            match treemap.bitmap_entry(prefix) {
                BitmapEntry::Vacant(entry) => {
                    entry.insert(bitmap.clone());
                }
                BitmapEntry::Occupied(_) => unreachable!("prefix was removed"),
            }
            match treemap.bitmap_entry(prefix) {
                BitmapEntry::Occupied(mut entry) => {
                    prop_assert_eq!(&*entry.get_mut(), &bitmap);
                }
                BitmapEntry::Vacant(_) => unreachable!("prefix was inserted"),
            }
        }
    }
}
