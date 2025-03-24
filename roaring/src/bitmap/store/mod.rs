mod array_store;
mod bitmap_store;

use alloc::vec;
use core::mem;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, Sub, SubAssign,
};
use core::slice;

pub use self::bitmap_store::BITMAP_LENGTH;
use self::Store::{Array, Bitmap};

pub(crate) use self::array_store::ArrayStore;
pub use self::bitmap_store::{BitmapIter, BitmapStore};

use crate::bitmap::container::ARRAY_LIMIT;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[derive(Clone)]
pub(crate) enum Store {
    Array(ArrayStore),
    Bitmap(BitmapStore),
}

#[derive(Clone)]
pub(crate) enum Iter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<&'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<Box<[u64; BITMAP_LENGTH]>>),
}

impl Store {
    pub fn new() -> Store {
        Store::Array(ArrayStore::new())
    }

    #[cfg(feature = "std")]
    pub fn with_capacity(capacity: usize) -> Store {
        if capacity <= ARRAY_LIMIT as usize {
            Store::Array(ArrayStore::with_capacity(capacity))
        } else {
            Store::Bitmap(BitmapStore::new())
        }
    }

    pub fn full() -> Store {
        Store::Bitmap(BitmapStore::full())
    }

    pub fn from_lsb0_bytes(bytes: &[u8], byte_offset: usize) -> Option<Self> {
        assert!(byte_offset + bytes.len() <= BITMAP_LENGTH * mem::size_of::<u64>());

        // It seems to be pretty considerably faster to count the bits
        // using u64s than for each byte
        let bits_set = {
            let mut bits_set = 0;
            let chunks = bytes.chunks_exact(mem::size_of::<u64>());
            let remainder = chunks.remainder();
            for chunk in chunks {
                let chunk = u64::from_ne_bytes(chunk.try_into().unwrap());
                bits_set += u64::from(chunk.count_ones());
            }
            for byte in remainder {
                bits_set += u64::from(byte.count_ones());
            }
            bits_set
        };
        if bits_set == 0 {
            return None;
        }

        Some(if bits_set < ARRAY_LIMIT {
            Array(ArrayStore::from_lsb0_bytes(bytes, byte_offset, bits_set))
        } else {
            Bitmap(BitmapStore::from_lsb0_bytes_unchecked(bytes, byte_offset, bits_set))
        })
    }

    #[inline]
    pub fn insert(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.insert(index),
            Bitmap(bits) => bits.insert(index),
        }
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        // A Range is defined as being of size 0 if start >= end.
        if range.is_empty() {
            return 0;
        }

        match self {
            Array(vec) => vec.insert_range(range),
            Bitmap(bits) => bits.insert_range(range),
        }
    }

    /// Push `index` at the end of the store only if `index` is the new max.
    ///
    /// Returns whether `index` was effectively pushed.
    pub fn push(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.push(index),
            Bitmap(bits) => bits.push(index),
        }
    }

    ///
    /// Pushes `index` at the end of the store.
    /// It is up to the caller to have validated index > self.max()
    ///
    /// # Panics
    ///
    /// If debug_assertions enabled and index is > self.max()
    pub(crate) fn push_unchecked(&mut self, index: u16) {
        match self {
            Array(vec) => vec.push_unchecked(index),
            Bitmap(bits) => bits.push_unchecked(index),
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.remove(index),
            Bitmap(bits) => bits.remove(index),
        }
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        if range.is_empty() {
            return 0;
        }

        match self {
            Array(vec) => vec.remove_range(range),
            Bitmap(bits) => bits.remove_range(range),
        }
    }

    pub fn remove_smallest(&mut self, index: u64) {
        match self {
            Array(vec) => vec.remove_smallest(index),
            Bitmap(bits) => bits.remove_smallest(index),
        }
    }

    pub fn remove_biggest(&mut self, index: u64) {
        match self {
            Array(vec) => vec.remove_biggest(index),
            Bitmap(bits) => bits.remove_biggest(index),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match self {
            Array(vec) => vec.contains(index),
            Bitmap(bits) => bits.contains(index),
        }
    }

    pub fn contains_range(&self, range: RangeInclusive<u16>) -> bool {
        match self {
            Array(vec) => vec.contains_range(range),
            Bitmap(bits) => bits.contains_range(range),
        }
    }

    pub fn is_full(&self) -> bool {
        self.len() == (1 << 16)
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.is_disjoint(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.is_disjoint(bits2),
            (Array(vec), Bitmap(bits)) | (Bitmap(bits), Array(vec)) => {
                vec.iter().all(|&i| !bits.contains(i))
            }
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.is_subset(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.is_subset(bits2),
            (Array(vec), Bitmap(bits)) => vec.iter().all(|&i| bits.contains(i)),
            (Bitmap(..), &Array(..)) => false,
        }
    }

    pub fn intersection_len(&self, other: &Self) -> u64 {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.intersection_len(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.intersection_len_bitmap(bits2),
            (Array(vec), Bitmap(bits)) => bits.intersection_len_array(vec),
            (Bitmap(bits), Array(vec)) => bits.intersection_len_array(vec),
        }
    }

    pub fn len(&self) -> u64 {
        match self {
            Array(vec) => vec.len(),
            Bitmap(bits) => bits.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Array(vec) => vec.is_empty(),
            Bitmap(bits) => bits.is_empty(),
        }
    }

    pub fn min(&self) -> Option<u16> {
        match self {
            Array(vec) => vec.min(),
            Bitmap(bits) => bits.min(),
        }
    }

    #[inline]
    pub fn max(&self) -> Option<u16> {
        match self {
            Array(vec) => vec.max(),
            Bitmap(bits) => bits.max(),
        }
    }

    pub fn rank(&self, index: u16) -> u64 {
        match self {
            Array(vec) => vec.rank(index),
            Bitmap(bits) => bits.rank(index),
        }
    }

    pub fn select(&self, n: u16) -> Option<u16> {
        match self {
            Array(vec) => vec.select(n),
            Bitmap(bits) => bits.select(n),
        }
    }

    pub(crate) fn to_bitmap(&self) -> Store {
        match self {
            Array(arr) => Bitmap(arr.to_bitmap_store()),
            Bitmap(_) => self.clone(),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}

impl BitOr<&Store> for &Store {
    type Output = Store;

    fn bitor(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitOr::bitor(vec1, vec2)),
            (&Bitmap(..), &Array(..)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Bitmap(..), &Bitmap(..)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Array(..), &Bitmap(..)) => {
                let mut rhs = rhs.clone();
                BitOrAssign::bitor_assign(&mut rhs, self);
                rhs
            }
        }
    }
}

impl BitOrAssign<Store> for Store {
    fn bitor_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref vec2)) => {
                *vec1 = BitOr::bitor(&*vec1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Array(ref vec2)) => {
                BitOrAssign::bitor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitOrAssign::bitor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), &mut Bitmap(..)) => {
                mem::swap(this, &mut rhs);
                BitOrAssign::bitor_assign(this, rhs);
            }
        }
    }
}

impl BitOrAssign<&Store> for Store {
    fn bitor_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let this = mem::take(vec1);
                *vec1 = BitOr::bitor(&this, vec2);
            }
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                BitOrAssign::bitor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitOrAssign::bitor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), Bitmap(bits2)) => {
                let mut lhs: Store = Bitmap(bits2.clone());
                BitOrAssign::bitor_assign(&mut lhs, &*this);
                *this = lhs;
            }
        }
    }
}

impl BitAnd<&Store> for &Store {
    type Output = Store;

    fn bitand(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitAnd::bitand(vec1, vec2)),
            (&Bitmap(..), &Array(..)) => {
                let mut rhs = rhs.clone();
                BitAndAssign::bitand_assign(&mut rhs, self);
                rhs
            }
            _ => {
                let mut lhs = self.clone();
                BitAndAssign::bitand_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl BitAndAssign<Store> for Store {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn bitand_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref mut vec2)) => {
                if vec2.len() < vec1.len() {
                    mem::swap(vec1, vec2);
                }
                BitAndAssign::bitand_assign(vec1, &*vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitAndAssign::bitand_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), &mut Bitmap(ref bits2)) => {
                BitAndAssign::bitand_assign(vec1, bits2);
            }
            (this @ &mut Bitmap(..), &mut Array(..)) => {
                mem::swap(this, &mut rhs);
                BitAndAssign::bitand_assign(this, rhs);
            }
        }
    }
}

impl BitAndAssign<&Store> for Store {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn bitand_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let (mut lhs, rhs) = if vec2.len() < vec1.len() {
                    (vec2.clone(), &*vec1)
                } else {
                    (mem::take(vec1), vec2)
                };

                BitAndAssign::bitand_assign(&mut lhs, rhs);
                *vec1 = lhs;
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitAndAssign::bitand_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), Bitmap(bits2)) => {
                BitAndAssign::bitand_assign(vec1, bits2);
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = rhs.clone();
                BitAndAssign::bitand_assign(&mut new, &*this);
                *this = new;
            }
        }
    }
}

impl Sub<&Store> for &Store {
    type Output = Store;

    fn sub(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(Sub::sub(vec1, vec2)),
            _ => {
                let mut lhs = self.clone();
                SubAssign::sub_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl SubAssign<&Store> for Store {
    fn sub_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                SubAssign::sub_assign(vec1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                SubAssign::sub_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                SubAssign::sub_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), Bitmap(bits2)) => {
                SubAssign::sub_assign(vec1, bits2);
            }
        }
    }
}

impl BitXor<&Store> for &Store {
    type Output = Store;

    fn bitxor(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitXor::bitxor(vec1, vec2)),
            (&Array(..), &Bitmap(..)) => {
                let mut lhs = rhs.clone();
                BitXorAssign::bitxor_assign(&mut lhs, self);
                lhs
            }
            _ => {
                let mut lhs = self.clone();
                BitXorAssign::bitxor_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl BitXorAssign<Store> for Store {
    fn bitxor_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref vec2)) => {
                *vec1 = BitXor::bitxor(&*vec1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Array(ref vec2)) => {
                BitXorAssign::bitxor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitXorAssign::bitxor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), &mut Bitmap(..)) => {
                mem::swap(this, &mut rhs);
                BitXorAssign::bitxor_assign(this, rhs);
            }
        }
    }
}

impl BitXorAssign<&Store> for Store {
    fn bitxor_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let this = mem::take(vec1);
                *vec1 = BitXor::bitxor(&this, vec2);
            }
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                BitXorAssign::bitxor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitXorAssign::bitxor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), Bitmap(bits2)) => {
                let mut lhs: Store = Bitmap(bits2.clone());
                BitXorAssign::bitxor_assign(&mut lhs, &*this);
                *this = lhs;
            }
        }
    }
}

impl<'a> IntoIterator for &'a Store {
    type Item = u16;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        match self {
            Array(vec) => Iter::Array(vec.iter()),
            Bitmap(bits) => Iter::BitmapBorrowed(bits.iter()),
        }
    }
}

impl IntoIterator for Store {
    type Item = u16;
    type IntoIter = Iter<'static>;
    fn into_iter(self) -> Iter<'static> {
        match self {
            Array(vec) => Iter::Vec(vec.into_iter()),
            Bitmap(bits) => Iter::BitmapOwned(bits.into_iter()),
        }
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1 == vec2,
            (Bitmap(bits1), Bitmap(bits2)) => {
                bits1.len() == bits2.len()
                    && bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            _ => false,
        }
    }
}

impl Iter<'_> {
    /// Advance the iterator to the first value greater than or equal to `n`.
    pub(crate) fn advance_to(&mut self, n: u16) {
        match self {
            Iter::Array(inner) => {
                let skip = inner.as_slice().partition_point(|&i| i < n);
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth(nth);
                }
            }
            Iter::Vec(inner) => {
                let skip = inner.as_slice().partition_point(|&i| i < n);
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth(nth);
                }
            }
            Iter::BitmapBorrowed(inner) => inner.advance_to(n),
            Iter::BitmapOwned(inner) => inner.advance_to(n),
        }
    }

    pub(crate) fn advance_back_to(&mut self, n: u16) {
        match self {
            Iter::Array(inner) => {
                let slice = inner.as_slice();
                let from_front = slice.partition_point(|&i| i <= n);
                let skip = slice.len() - from_front;
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth_back(nth);
                }
            }
            Iter::Vec(inner) => {
                let slice = inner.as_slice();
                let from_front = slice.partition_point(|&i| i <= n);
                let skip = slice.len() - from_front;
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth_back(nth);
                }
            }
            Iter::BitmapBorrowed(inner) => inner.advance_back_to(n),
            Iter::BitmapOwned(inner) => inner.advance_back_to(n),
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            Iter::Array(inner) => inner.next().cloned(),
            Iter::Vec(inner) => inner.next(),
            Iter::BitmapBorrowed(inner) => inner.next(),
            Iter::BitmapOwned(inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Array(inner) => inner.size_hint(),
            Iter::Vec(inner) => inner.size_hint(),
            Iter::BitmapBorrowed(inner) => inner.size_hint(),
            Iter::BitmapOwned(inner) => inner.size_hint(),
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Iter::Array(inner) => inner.count(),
            Iter::Vec(inner) => inner.count(),
            Iter::BitmapBorrowed(inner) => inner.count(),
            Iter::BitmapOwned(inner) => inner.count(),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Iter::Array(inner) => inner.nth(n).copied(),
            Iter::Vec(inner) => inner.nth(n),
            Iter::BitmapBorrowed(inner) => inner.nth(n),
            Iter::BitmapOwned(inner) => inner.nth(n),
        }
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Array(inner) => inner.next_back().cloned(),
            Iter::Vec(inner) => inner.next_back(),
            Iter::BitmapBorrowed(inner) => inner.next_back(),
            Iter::BitmapOwned(inner) => inner.next_back(),
        }
    }
}

impl ExactSizeIterator for Iter<'_> {}
