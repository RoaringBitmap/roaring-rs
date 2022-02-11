mod scalar;
mod vector;
mod visitor;

use crate::bitmap::store::array_store::visitor::{CardinalityCounter, VecWriter};
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitXor, RangeInclusive, Sub, SubAssign};

use super::bitmap_store::{bit, key, BitmapStore, BITMAP_LENGTH};

#[derive(Clone, Eq, PartialEq)]
pub struct ArrayStore {
    vec: Vec<u16>,
}

impl ArrayStore {
    pub fn new() -> ArrayStore {
        ArrayStore { vec: vec![] }
    }

    ///
    /// Create a new SortedU16Vec from a given vec
    /// It is up to the caller to ensure the vec is sorted and deduplicated
    /// Favor `try_from` / `try_into` for cases in which these invariants should be checked
    ///
    /// # Panics
    ///
    /// When debug_assertions are enabled and the above invariants are not met
    #[inline]
    pub fn from_vec_unchecked(vec: Vec<u16>) -> ArrayStore {
        if cfg!(debug_assertions) {
            vec.try_into().unwrap()
        } else {
            ArrayStore { vec }
        }
    }

    pub fn insert(&mut self, index: u16) -> bool {
        self.vec.binary_search(&index).map_err(|loc| self.vec.insert(loc, index)).is_err()
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let start = *range.start();
        let end = *range.end();

        // Figure out the starting/ending position in the vec.
        let pos_start = self.vec.binary_search(&start).unwrap_or_else(|x| x);
        let pos_end = self
            .vec
            .binary_search_by(|p| {
                // binary search the right most position when equals
                match p.cmp(&end) {
                    Greater => Greater,
                    _ => Less,
                }
            })
            .unwrap_or_else(|x| x);

        // Overwrite the range in the middle - there's no need to take
        // into account any existing elements between start and end, as
        // they're all being added to the set.
        let dropped = self.vec.splice(pos_start..pos_end, start..=end);

        end as u64 - start as u64 + 1 - dropped.len() as u64
    }

    pub fn push(&mut self, index: u16) -> bool {
        if self.max().map_or(true, |max| max < index) {
            self.vec.push(index);
            true
        } else {
            false
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
        if cfg!(debug_assertions) {
            if let Some(max) = self.max() {
                assert!(index > max, "store max >= index")
            }
        }
        self.vec.push(index);
    }

    pub fn remove(&mut self, index: u16) -> bool {
        self.vec.binary_search(&index).map(|loc| self.vec.remove(loc)).is_ok()
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let start = *range.start();
        let end = *range.end();

        // Figure out the starting/ending position in the vec.
        let pos_start = self.vec.binary_search(&start).unwrap_or_else(|x| x);
        let pos_end = self
            .vec
            .binary_search_by(|p| {
                // binary search the right most position when equals
                match p.cmp(&end) {
                    Greater => Greater,
                    _ => Less,
                }
            })
            .unwrap_or_else(|x| x);
        self.vec.drain(pos_start..pos_end);
        (pos_end - pos_start) as u64
    }

    pub fn contains(&self, index: u16) -> bool {
        self.vec.binary_search(&index).is_ok()
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        let (mut i1, mut i2) = (self.vec.iter(), other.vec.iter());
        let (mut value1, mut value2) = (i1.next(), i2.next());
        loop {
            match value1.and_then(|v1| value2.map(|v2| v1.cmp(v2))) {
                None => return true,
                Some(Equal) => return false,
                Some(Less) => value1 = i1.next(),
                Some(Greater) => value2 = i2.next(),
            }
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        let (mut i1, mut i2) = (self.iter(), other.iter());
        let (mut value1, mut value2) = (i1.next(), i2.next());
        loop {
            match (value1, value2) {
                (None, _) => return true,
                (Some(..), None) => return false,
                (Some(v1), Some(v2)) => match v1.cmp(v2) {
                    Equal => {
                        value1 = i1.next();
                        value2 = i2.next();
                    }
                    Less => return false,
                    Greater => value2 = i2.next(),
                },
            }
        }
    }

    pub fn intersection_len(&self, other: &Self) -> u64 {
        let mut visitor = CardinalityCounter::new();
        #[cfg(feature = "simd")]
        vector::and(self.as_slice(), other.as_slice(), &mut visitor);
        #[cfg(not(feature = "simd"))]
        scalar::and(self.as_slice(), other.as_slice(), &mut visitor);
        visitor.into_inner()
    }

    pub fn to_bitmap_store(&self) -> BitmapStore {
        let mut bits = Box::new([0; BITMAP_LENGTH]);
        let len = self.len() as u64;

        for &index in self.iter() {
            bits[key(index)] |= 1 << bit(index);
        }
        BitmapStore::from_unchecked(len, bits)
    }

    pub fn len(&self) -> u64 {
        self.vec.len() as u64
    }

    pub fn min(&self) -> Option<u16> {
        self.vec.first().copied()
    }

    pub fn max(&self) -> Option<u16> {
        self.vec.last().copied()
    }

    pub fn rank(&self, index: u16) -> u64 {
        match self.vec.binary_search(&index) {
            Ok(i) => i as u64 + 1,
            Err(i) => i as u64,
        }
    }

    pub fn select(&self, n: u16) -> Option<u16> {
        self.vec.get(n as usize).cloned()
    }

    pub fn iter(&self) -> std::slice::Iter<u16> {
        self.vec.iter()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<u16> {
        self.vec.into_iter()
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.vec
    }
}

impl Default for ArrayStore {
    fn default() -> Self {
        ArrayStore::new()
    }
}

#[derive(Debug)]
pub struct Error {
    index: usize,
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Duplicate,
    OutOfOrder,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Duplicate => {
                write!(f, "Duplicate element found at index: {}", self.index)
            }
            ErrorKind::OutOfOrder => {
                write!(f, "An element was out of order at index: {}", self.index)
            }
        }
    }
}

impl std::error::Error for Error {}

impl TryFrom<Vec<u16>> for ArrayStore {
    type Error = Error;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        let mut iter = value.iter().enumerate();
        if let Some((_, mut prev)) = iter.next() {
            for (i, cur) in iter {
                match cur.cmp(prev) {
                    Ordering::Less => return Err(Error { index: i, kind: ErrorKind::OutOfOrder }),
                    Ordering::Equal => return Err(Error { index: i, kind: ErrorKind::Duplicate }),
                    Ordering::Greater => (),
                }
                prev = cur;
            }
        }

        Ok(ArrayStore { vec: value })
    }
}

impl BitOr<Self> for &ArrayStore {
    type Output = ArrayStore;

    fn bitor(self, rhs: Self) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        let capacity = self.vec.len() + rhs.vec.len();
        let mut visitor = VecWriter::new(capacity);
        #[cfg(feature = "simd")]
        vector::or(self.as_slice(), rhs.as_slice(), &mut visitor);
        #[cfg(not(feature = "simd"))]
        scalar::or(self.as_slice(), rhs.as_slice(), &mut visitor);
        ArrayStore::from_vec_unchecked(visitor.into_inner())
    }
}

impl BitAnd<Self> for &ArrayStore {
    type Output = ArrayStore;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut visitor = VecWriter::new(self.vec.len().min(rhs.vec.len()));
        #[cfg(feature = "simd")]
        vector::and(self.as_slice(), rhs.as_slice(), &mut visitor);
        #[cfg(not(feature = "simd"))]
        scalar::and(self.as_slice(), rhs.as_slice(), &mut visitor);
        ArrayStore::from_vec_unchecked(visitor.into_inner())
    }
}

impl BitAndAssign<&Self> for ArrayStore {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn bitand_assign(&mut self, rhs: &Self) {
        #[cfg(feature = "simd")]
        {
            let mut visitor = VecWriter::new(self.vec.len().min(rhs.vec.len()));
            vector::and(self.as_slice(), rhs.as_slice(), &mut visitor);
            self.vec = visitor.into_inner()
        }
        #[cfg(not(feature = "simd"))]
        {
            let mut i = 0;
            self.vec.retain(|x| {
                i += rhs.iter().skip(i).position(|y| y >= x).unwrap_or(rhs.vec.len());
                rhs.vec.get(i).map_or(false, |y| x == y)
            });
        }
    }
}

impl BitAndAssign<&BitmapStore> for ArrayStore {
    fn bitand_assign(&mut self, rhs: &BitmapStore) {
        self.vec.retain(|x| rhs.contains(*x));
    }
}

impl Sub<Self> for &ArrayStore {
    type Output = ArrayStore;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut visitor = VecWriter::new(self.vec.len());
        #[cfg(feature = "simd")]
        vector::sub(self.as_slice(), rhs.as_slice(), &mut visitor);
        #[cfg(not(feature = "simd"))]
        scalar::sub(self.as_slice(), rhs.as_slice(), &mut visitor);
        ArrayStore::from_vec_unchecked(visitor.into_inner())
    }
}

impl SubAssign<&Self> for ArrayStore {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: &Self) {
        #[cfg(feature = "simd")]
        {
            let mut visitor = VecWriter::new(self.vec.len().min(rhs.vec.len()));
            vector::sub(self.as_slice(), rhs.as_slice(), &mut visitor);
            self.vec = visitor.into_inner()
        }
        #[cfg(not(feature = "simd"))]
        {
            let mut i = 0;
            self.vec.retain(|x| {
                i += rhs.iter().skip(i).position(|y| y >= x).unwrap_or(rhs.vec.len());
                rhs.vec.get(i).map_or(true, |y| x != y)
            });
        }
    }
}

impl SubAssign<&BitmapStore> for ArrayStore {
    fn sub_assign(&mut self, rhs: &BitmapStore) {
        self.vec.retain(|x| !rhs.contains(*x));
    }
}

impl BitXor<Self> for &ArrayStore {
    type Output = ArrayStore;

    fn bitxor(self, rhs: Self) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        let capacity = self.vec.len() + rhs.vec.len();
        let mut visitor = VecWriter::new(capacity);
        #[cfg(feature = "simd")]
        vector::xor(self.as_slice(), rhs.as_slice(), &mut visitor);
        #[cfg(not(feature = "simd"))]
        scalar::xor(self.as_slice(), rhs.as_slice(), &mut visitor);
        ArrayStore::from_vec_unchecked(visitor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap::store::Store;

    fn into_vec(s: Store) -> Vec<u16> {
        match s {
            Store::Array(vec) => vec.vec,
            Store::Bitmap(bits) => bits.to_array_store().vec,
        }
    }

    fn into_bitmap_store(s: Store) -> Store {
        match s {
            Store::Array(vec) => Store::Bitmap(vec.to_bitmap_store()),
            Store::Bitmap(..) => s,
        }
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_array_insert_invalid_range() {
        let mut store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));

        // Insert a range with start > end.
        let new = store.insert_range(6..=1);
        assert_eq!(new, 0);

        assert_eq!(into_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_array_insert_range() {
        let mut store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));

        let new = store.insert_range(4..=5);
        assert_eq!(new, 2);

        assert_eq!(into_vec(store), vec![1, 2, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_left_overlap() {
        let mut store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));

        let new = store.insert_range(2..=5);
        assert_eq!(new, 3);

        assert_eq!(into_vec(store), vec![1, 2, 3, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_right_overlap() {
        let mut store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));

        let new = store.insert_range(4..=8);
        assert_eq!(new, 4);

        assert_eq!(into_vec(store), vec![1, 2, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_full_overlap() {
        let mut store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));

        let new = store.insert_range(1..=9);
        assert_eq!(new, 5);

        assert_eq!(into_vec(store), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_bitmap_insert_invalid_range() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 8, 9]));
        let mut store = into_bitmap_store(store);

        // Insert a range with start > end.
        let new = store.insert_range(6..=1);
        assert_eq!(new, 0);

        assert_eq!(into_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_bitmap_insert_same_key_overlap() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 3, 62, 63]));
        let mut store = into_bitmap_store(store);

        let new = store.insert_range(1..=62);
        assert_eq!(new, 58);

        assert_eq!(into_vec(store), (1..64).collect::<Vec<_>>());
    }

    #[test]
    fn test_bitmap_insert_range() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 130]));
        let mut store = into_bitmap_store(store);

        let new = store.insert_range(4..=128);
        assert_eq!(new, 125);

        let mut want = vec![1, 2];
        want.extend(4..129);
        want.extend(&[130]);

        assert_eq!(into_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_left_overlap() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 130]));
        let mut store = into_bitmap_store(store);

        let new = store.insert_range(1..=128);
        assert_eq!(new, 126);

        let mut want = Vec::new();
        want.extend(1..129);
        want.extend(&[130]);

        assert_eq!(into_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_right_overlap() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 130]));
        let mut store = into_bitmap_store(store);

        let new = store.insert_range(4..=132);
        assert_eq!(new, 128);

        let mut want = vec![1, 2];
        want.extend(4..133);

        assert_eq!(into_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_full_overlap() {
        let store = Store::Array(ArrayStore::from_vec_unchecked(vec![1, 2, 130]));
        let mut store = into_bitmap_store(store);

        let new = store.insert_range(1..=134);
        assert_eq!(new, 131);

        let mut want = Vec::new();
        want.extend(1..135);

        assert_eq!(into_vec(store), want);
    }
}
