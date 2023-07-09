use std::collections::btree_map;
use std::collections::BTreeMap;
use std::iter::{self, FromIterator};

use super::util;
use crate::bitmap::IntoIter as IntoIter32;
use crate::bitmap::Iter as Iter32;
use crate::{NonSortedIntegers, RoaringBitmap, RoaringTreemap};

struct To64Iter<'a> {
    hi: u32,
    inner: Iter32<'a>,
}

impl<'a> Iterator for To64Iter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|n| util::join(self.hi, n))
    }
}

impl DoubleEndedIterator for To64Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|n| util::join(self.hi, n))
    }
}

fn to64iter<'a>(t: (&'a u32, &'a RoaringBitmap)) -> To64Iter<'a> {
    To64Iter { hi: *t.0, inner: t.1.iter() }
}

struct To64IntoIter {
    hi: u32,
    inner: IntoIter32,
}

impl Iterator for To64IntoIter {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|n| util::join(self.hi, n))
    }
}

impl DoubleEndedIterator for To64IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|n| util::join(self.hi, n))
    }
}

fn to64intoiter(t: (u32, RoaringBitmap)) -> To64IntoIter {
    To64IntoIter { hi: t.0, inner: t.1.into_iter() }
}

type InnerIter<'a> = iter::FlatMap<
    btree_map::Iter<'a, u32, RoaringBitmap>,
    To64Iter<'a>,
    fn((&'a u32, &'a RoaringBitmap)) -> To64Iter<'a>,
>;
type InnerIntoIter = iter::FlatMap<
    btree_map::IntoIter<u32, RoaringBitmap>,
    To64IntoIter,
    fn((u32, RoaringBitmap)) -> To64IntoIter,
>;

/// An iterator for `RoaringTreemap`.
pub struct Iter<'a> {
    inner: InnerIter<'a>,
    size_hint: u64,
}

/// An iterator for `RoaringTreemap`.
pub struct IntoIter {
    inner: InnerIntoIter,
    size_hint: u64,
}

impl<'a> Iter<'a> {
    fn new(map: &BTreeMap<u32, RoaringBitmap>) -> Iter {
        let size_hint: u64 = map.iter().map(|(_, r)| r.len()).sum();
        let i = map.iter().flat_map(to64iter as _);
        Iter { inner: i, size_hint }
    }
}

impl IntoIter {
    fn new(map: BTreeMap<u32, RoaringBitmap>) -> IntoIter {
        let size_hint = map.values().map(|r| r.len()).sum();
        let i = map.into_iter().flat_map(to64intoiter as _);
        IntoIter { inner: i, size_hint }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next_back()
    }
}

#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}

impl Iterator for IntoIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next_back()
    }
}

#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}

impl RoaringTreemap {
    /// Iterator over each value stored in the RoaringTreemap, guarantees values are ordered by
    /// value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use std::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringTreemap>();
    /// let mut iter = bitmap.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter {
        Iter::new(&self.map)
    }

    /// Iterator over pairs of partition number and the corresponding RoaringBitmap.
    /// The partition number is defined by the 32 most significant bits of the bit index.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::{RoaringBitmap, RoaringTreemap};
    /// use std::iter::FromIterator;
    ///
    /// let original = (0..6000).collect::<RoaringTreemap>();
    /// let mut bitmaps = original.bitmaps();
    ///
    /// assert_eq!(bitmaps.next(), Some((0, &(0..6000).collect::<RoaringBitmap>())));
    /// assert_eq!(bitmaps.next(), None);
    /// ```
    pub fn bitmaps(&self) -> BitmapIter {
        BitmapIter(self.map.iter())
    }

    /// Construct a RoaringTreemap from an iterator of partition number and RoaringBitmap pairs.
    /// The partition number is defined by the 32 most significant bits of the bit index.
    /// Note that repeated partitions, if present, will replace previously set partitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use std::iter::FromIterator;
    ///
    /// let original = (0..6000).collect::<RoaringTreemap>();
    /// let clone = RoaringTreemap::from_bitmaps(original.bitmaps().map(|(p, b)| (p, b.clone())));
    ///
    /// assert_eq!(clone, original);
    /// ```
    pub fn from_bitmaps<I: IntoIterator<Item = (u32, RoaringBitmap)>>(iterator: I) -> Self {
        RoaringTreemap { map: iterator.into_iter().collect() }
    }
}

impl<'a> IntoIterator for &'a RoaringTreemap {
    type Item = u64;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl IntoIterator for RoaringTreemap {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(self.map)
    }
}

impl<const N: usize> From<[u64; N]> for RoaringTreemap {
    fn from(arr: [u64; N]) -> Self {
        RoaringTreemap::from_iter(arr)
    }
}

impl FromIterator<u64> for RoaringTreemap {
    fn from_iter<I: IntoIterator<Item = u64>>(iterator: I) -> RoaringTreemap {
        let mut rb = RoaringTreemap::new();
        rb.extend(iterator);
        rb
    }
}

impl<'a> FromIterator<&'a u64> for RoaringTreemap {
    fn from_iter<I: IntoIterator<Item = &'a u64>>(iterator: I) -> RoaringTreemap {
        let mut rb = RoaringTreemap::new();
        rb.extend(iterator);
        rb
    }
}

impl Extend<u64> for RoaringTreemap {
    fn extend<I: IntoIterator<Item = u64>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl<'a> Extend<&'a u64> for RoaringTreemap {
    fn extend<I: IntoIterator<Item = &'a u64>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(*value);
        }
    }
}

impl RoaringTreemap {
    /// Create the set from a sorted iterator. Values must be sorted and deduplicated.
    ///
    /// The values of the iterator must be ordered and strictly greater than the greatest value
    /// in the set. If a value in the iterator doesn't satisfy this requirement, it is not added
    /// and the append operation is stopped.
    ///
    /// Returns `Ok` with the requested `RoaringTreemap`, `Err` with the number of elements
    /// we tried to append before an error occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::from_sorted_iter(0..10).unwrap();
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    pub fn from_sorted_iter<I: IntoIterator<Item = u64>>(
        iterator: I,
    ) -> Result<RoaringTreemap, NonSortedIntegers> {
        let mut rt = RoaringTreemap::new();
        rt.append(iterator).map(|_| rt)
    }

    /// Extend the set with a sorted iterator.
    ///
    /// The values of the iterator must be ordered and strictly greater than the greatest value
    /// in the set. If a value in the iterator doesn't satisfy this requirement, it is not added
    /// and the append operation is stopped.
    ///
    /// Returns `Ok` with the number of elements appended to the set, `Err` with
    /// the number of elements we effectively appended before an error occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb = RoaringTreemap::new();
    /// rb.append(0..10);
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    pub fn append<I: IntoIterator<Item = u64>>(
        &mut self,
        iterator: I,
    ) -> Result<u64, NonSortedIntegers> {
        let mut iterator = iterator.into_iter();
        let mut prev = match (iterator.next(), self.max()) {
            (None, _) => return Ok(0),
            (Some(first), Some(max)) if first <= max => {
                return Err(NonSortedIntegers { valid_until: 0 })
            }
            (Some(first), _) => first,
        };

        // It is now guaranteed that so long as the values of the iterator are
        // monotonically increasing they must also be the greatest in the set.

        self.push_unchecked(prev);

        let mut count = 1;
        for value in iterator {
            if value <= prev {
                return Err(NonSortedIntegers { valid_until: count });
            } else {
                self.push_unchecked(value);
                prev = value;
                count += 1;
            }
        }

        Ok(count)
    }
}

pub struct BitmapIter<'a>(btree_map::Iter<'a, u32, RoaringBitmap>);

impl<'a> Iterator for BitmapIter<'a> {
    type Item = (u32, &'a RoaringBitmap);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(&p, b)| (p, b))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FromIterator<(u32, RoaringBitmap)> for RoaringTreemap {
    fn from_iter<I: IntoIterator<Item = (u32, RoaringBitmap)>>(iterator: I) -> RoaringTreemap {
        Self::from_bitmaps(iterator)
    }
}
