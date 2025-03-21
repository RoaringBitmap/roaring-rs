use alloc::collections::{btree_map, BTreeMap};
use core::iter;
use std::iter::Peekable;

use super::util;
use crate::bitmap::IntoIter as IntoIter32;
use crate::bitmap::Iter as Iter32;
use crate::{NonSortedIntegers, RoaringBitmap, RoaringTreemap};

struct To64Iter<'a> {
    hi: u32,
    inner: Iter32<'a>,
}

impl To64Iter<'_> {
    fn advance_to(&mut self, n: u32) {
        self.inner.advance_to(n)
    }

    fn advance_back_to(&mut self, n: u32) {
        self.inner.advance_back_to(n)
    }
}

impl Iterator for To64Iter<'_> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|n| util::join(self.hi, n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, move |b, lo| f(b, ((self.hi as u64) << 32) + (lo as u64)))
    }
}

impl DoubleEndedIterator for To64Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|n| util::join(self.hi, n))
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, move |b, lo| f(b, ((self.hi as u64) << 32) + (lo as u64)))
    }
}

fn to64iter(t: (u32, &RoaringBitmap)) -> To64Iter<'_> {
    To64Iter { hi: t.0, inner: t.1.iter() }
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

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, move |b, lo| f(b, ((self.hi as u64) << 32) + (lo as u64)))
    }
}

impl DoubleEndedIterator for To64IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|n| util::join(self.hi, n))
    }

    #[inline]
    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, move |b, lo| f(b, ((self.hi as u64) << 32) + (lo as u64)))
    }
}

fn to64intoiter(t: (u32, RoaringBitmap)) -> To64IntoIter {
    To64IntoIter { hi: t.0, inner: t.1.into_iter() }
}

type InnerIntoIter = iter::FlatMap<
    btree_map::IntoIter<u32, RoaringBitmap>,
    To64IntoIter,
    fn((u32, RoaringBitmap)) -> To64IntoIter,
>;

/// An iterator for `RoaringTreemap`.
pub struct Iter<'a> {
    outer: Peekable<BitmapIter<'a>>,
    front: Option<To64Iter<'a>>,
    back: Option<To64Iter<'a>>,
    size_hint: u64,
}

/// An iterator for `RoaringTreemap`.
pub struct IntoIter {
    inner: InnerIntoIter,
    size_hint: u64,
}

impl Iter<'_> {
    fn new(map: &BTreeMap<u32, RoaringBitmap>) -> Iter {
        let size_hint: u64 = map.iter().map(|(_, r)| r.len()).sum();
        let outer = BitmapIter(map.iter()).peekable();
        Iter { size_hint, outer, front: None, back: None }
    }

    /// Advance the iterator to the first position where the item has a value >= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringTreemap>();
    /// let mut iter = bitmap.iter();
    /// iter.advance_to(2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn advance_to(&mut self, n: u64) {
        let (key, index) = util::split(n);

        loop {
            match self.outer.peek() {
                None => {
                    break;
                }
                Some((next_hi, _)) => {
                    if *next_hi > key {
                        if let Some(ref front) = self.front {
                            self.size_hint =
                                self.size_hint.saturating_sub(front.size_hint().0 as u64);
                        }

                        let next = self.outer.next().unwrap();
                        self.front = Some(to64iter(next));
                    } else {
                        break;
                    }
                }
            }
        }

        if self.front.is_none() {
            let Some(next) = self.outer.next() else {
                // if the current front iterator is empty or not yet initialized,
                // but the outer bitmap iterator is empty, then consume the back
                // iterator from the front if it is not also exhausted
                if let Some(ref mut back) = self.back {
                    let size_hint_pre = back.size_hint().0;
                    back.advance_to(index);
                    let size_hint_post = back.size_hint().0;

                    self.size_hint =
                        self.size_hint.saturating_sub((size_hint_pre - size_hint_post) as u64);
                }
                return;
            };
            self.front = Some(to64iter(next));
        }

        if let Some(ref mut front) = self.front {
            let size_hint_pre = front.size_hint().0;
            front.advance_to(index);
            let size_hint_post = front.size_hint().0;

            self.size_hint = self.size_hint.saturating_sub((size_hint_pre - size_hint_post) as u64);
        }
    }

    /// Advance the back of the iterator to the first position where the item has a value <= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringTreemap>();
    /// let mut iter = bitmap.iter();
    /// iter.advance_back_to(1);
    ///
    /// assert_eq!(iter.next_back(), Some(1));
    /// assert_eq!(iter.next_back(), None);
    /// ```
    pub fn advance_back_to(&mut self, n: u64) {
        let (key, index) = util::split(n);

        // advance beyond the current bitmap if n < min val in current bitmap
        loop {
            match self.outer.peek() {
                None => {
                    break;
                }
                Some((prev_hi, _)) => {
                    if *prev_hi < key {
                        if let Some(ref back) = self.back {
                            self.size_hint =
                                self.size_hint.saturating_sub(back.size_hint().0 as u64);
                        }

                        let next_back = self.outer.next_back().unwrap();
                        self.back = Some(to64iter(next_back));
                    } else {
                        break;
                    }
                }
            }
        }

        if self.back.is_none() {
            let Some(next_back) = self.outer.next_back() else {
                // if the current back iterator is empty or not yet initialized,
                // but the outer bitmap iterator is empty, then consume the front
                // iterator from the back if it is not also exhausted
                if let Some(ref mut front) = self.front {
                    let size_hint_pre = front.size_hint().0;
                    front.advance_back_to(index);
                    let size_hint_post = front.size_hint().0;

                    self.size_hint =
                        self.size_hint.saturating_sub((size_hint_pre - size_hint_post) as u64);
                }
                return;
            };
            self.back = Some(to64iter(next_back));
        }

        if let Some(ref mut back) = self.back {
            let size_hint_pre = back.size_hint().0;
            back.advance_back_to(index);
            let size_hint_post = back.size_hint().0;

            self.size_hint = self.size_hint.saturating_sub((size_hint_pre - size_hint_post) as u64);
        }
    }
}

impl IntoIter {
    fn new(map: BTreeMap<u32, RoaringBitmap>) -> IntoIter {
        let size_hint = map.values().map(|r| r.len()).sum();
        let i = map.into_iter().flat_map(to64intoiter as _);
        IntoIter { inner: i, size_hint }
    }
}

impl Iterator for Iter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if let Some(ref mut front) = &mut self.front {
            if let Some(inner) = front.next() {
                self.size_hint = self.size_hint.saturating_sub(1);
                return Some(inner);
            }
        }

        let Some(outer_next) = self.outer.next() else {
            // if the current front iterator is empty or not yet initialized,
            // but the outer bitmap iterator is empty, then consume the back
            // iterator from the front if it is not also exhausted
            if let Some(ref mut back) = &mut self.back {
                if let Some(next) = back.next() {
                    self.size_hint = self.size_hint.saturating_sub(1);
                    return Some(next);
                }
            }
            return None;
        };

        self.front = Some(to64iter(outer_next));
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }

    #[inline]
    fn fold<B, F>(self, _init: B, _f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        todo!();
        // self.inner.fold(init, f)
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(ref mut back) = &mut self.back {
            if let Some(inner) = back.next_back() {
                self.size_hint = self.size_hint.saturating_sub(1);
                return Some(inner);
            }
        }

        let Some(outer_next_back) = self.outer.next_back() else {
            // if the current back iterator is empty or not yet initialized,
            // but the outer bitmap iterator is empty, then consume the front
            // iterator from the back if it is not also exhausted
            if let Some(ref mut front) = &mut self.front {
                if let Some(next_back) = front.next_back() {
                    self.size_hint = self.size_hint.saturating_sub(1);
                    return Some(next_back);
                }
            }
            return None;
        };

        self.back = Some(to64iter(outer_next_back));
        self.next_back()
    }

    #[inline]
    fn rfold<Acc, Fold>(self, _init: Acc, _fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        todo!();
        // self.inner.rfold(init, fold)
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

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, f)
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next_back()
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        self.inner.rfold(init, fold)
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
    /// use core::iter::FromIterator;
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
    /// use core::iter::FromIterator;
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
    /// use core::iter::FromIterator;
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

/// An iterator of `RoaringBitmap`s for `RoaringTreemap`.
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

impl DoubleEndedIterator for BitmapIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(&p, b)| (p, b))
    }
}
