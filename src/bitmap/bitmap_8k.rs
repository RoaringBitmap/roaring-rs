use crate::bitmap::sorted_u16_vec::SortedU16Vec;
use crate::bitmap::store::Store;
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign, RangeInclusive, SubAssign};

pub const BITMAP_LENGTH: usize = 1024;

#[derive(Clone)]
pub struct Bitmap8K {
    len: u64,
    bits: Box<[u64; BITMAP_LENGTH]>,
}

impl Bitmap8K {
    pub fn new() -> Bitmap8K {
        Bitmap8K { len: 0, bits: Box::new([0; BITMAP_LENGTH]) }
    }

    pub fn try_from(len: u64, bits: Box<[u64; BITMAP_LENGTH]>) -> Result<Bitmap8K, Error> {
        let actual_len = bits.iter().map(|v| v.count_ones() as u64).sum();
        if len != actual_len {
            Err(Error { kind: ErrorKind::Cardinality { expected: len, actual: actual_len } })
        } else {
            Ok(Bitmap8K { len, bits })
        }
    }

    ///
    /// Create a new Bitmap8K from a given len and bits array
    /// It is up to the caller to ensure len == cardinality of bits
    /// Favor `try_from` for cases in which this invariants should be checked
    ///
    /// # Panics
    ///
    /// When debug_assertions are enabled and the above invariant is not met
    pub fn from_unchecked(len: u64, bits: Box<[u64; BITMAP_LENGTH]>) -> Bitmap8K {
        if cfg!(debug_assertions) {
            Bitmap8K::try_from(len, bits).unwrap()
        } else {
            Bitmap8K { len, bits }
        }
    }

    pub fn insert(&mut self, index: u16) -> bool {
        let (key, bit) = (key(index), bit(index));
        let old_w = self.bits[key];
        let new_w = old_w | 1 << bit;
        let inserted = (old_w ^ new_w) >> bit; // 1 or 0
        self.bits[key] = new_w;
        self.len += inserted;
        inserted != 0
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let start = *range.start();
        let end = *range.end();

        let (start_key, start_bit) = (key(start), bit(start));
        let (end_key, end_bit) = (key(end), bit(end));

        // MSB > start_bit > end_bit > LSB
        if start_key == end_key {
            // Set the end_bit -> LSB to 1
            let mut mask = if end_bit == 63 { u64::MAX } else { (1 << (end_bit + 1)) - 1 };
            // Set MSB -> start_bit to 1
            mask &= !((1 << start_bit) - 1);

            let existed = (self.bits[start_key] & mask).count_ones();
            self.bits[start_key] |= mask;

            let inserted = u64::from(end - start + 1) - u64::from(existed);
            self.len += inserted;
            return inserted;
        }

        // Mask off the left-most bits (MSB -> start_bit)
        let mask = !((1 << start_bit) - 1);

        // Keep track of the number of bits that were already set to
        // return how many new bits were set later
        let mut existed = (self.bits[start_key] & mask).count_ones();

        self.bits[start_key] |= mask;

        // Set the full blocks, tracking the number of set bits
        for i in (start_key + 1)..end_key {
            existed += self.bits[i].count_ones();
            self.bits[i] = u64::MAX;
        }

        // Set the end bits in the last chunk (MSB -> end_bit)
        let mask = if end_bit == 63 { u64::MAX } else { (1 << (end_bit + 1)) - 1 };
        existed += (self.bits[end_key] & mask).count_ones();
        self.bits[end_key] |= mask;

        let inserted = end as u64 - start as u64 + 1 - existed as u64;
        self.len += inserted;
        inserted
    }

    pub fn push(&mut self, index: u16) -> bool {
        if self.max().map_or(true, |max| max < index) {
            self.insert(index);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        let (key, bit) = (key(index), bit(index));
        let old_w = self.bits[key];
        let new_w = old_w & !(1 << bit);
        let removed = (old_w ^ new_w) >> bit; // 0 or 1
        self.bits[key] = new_w;
        self.len -= removed;
        removed != 0
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let start = *range.start();
        let end = *range.end();

        let (start_key, start_bit) = (key(start), bit(start));
        let (end_key, end_bit) = (key(end), bit(end));

        if start_key == end_key {
            let mask = (u64::MAX << start_bit) & (u64::MAX >> (63 - end_bit));
            let removed = (self.bits[start_key] & mask).count_ones();
            self.bits[start_key] &= !mask;
            let removed = u64::from(removed);
            self.len -= removed;
            return removed;
        }

        let mut removed = 0;
        // start key bits
        removed += (self.bits[start_key] & (u64::MAX << start_bit)).count_ones();
        self.bits[start_key] &= !(u64::MAX << start_bit);
        // counts bits in between
        for word in &self.bits[start_key + 1..end_key] {
            removed += word.count_ones();
            // When popcnt is available zeroing in this loop is faster,
            // but we opt to perform reasonably on most cpus by zeroing after.
            // By doing that the compiler uses simd to count ones.
        }
        // do zeroing outside the loop
        for word in &mut self.bits[start_key + 1..end_key] {
            *word = 0;
        }
        // end key bits
        removed += (self.bits[end_key] & (u64::MAX >> (63 - end_bit))).count_ones();
        self.bits[end_key] &= !(u64::MAX >> (63 - end_bit));
        let removed = u64::from(removed);
        self.len -= removed;
        removed
    }

    pub fn contains(&self, index: u16) -> bool {
        self.bits[key(index)] & (1 << bit(index)) != 0
    }

    pub fn is_disjoint(&self, other: &Bitmap8K) -> bool {
        self.bits.iter().zip(other.bits.iter()).all(|(&i1, &i2)| (i1 & i2) == 0)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.bits.iter().zip(other.bits.iter()).all(|(&i1, &i2)| (i1 & i2) == i1)
    }

    pub fn to_array_store(&self) -> Store {
        let mut vec = Vec::with_capacity(self.len as usize);
        for (index, mut bit) in self.bits.iter().cloned().enumerate() {
            while bit != 0 {
                vec.push((u64::trailing_zeros(bit) + (64 * index as u32)) as u16);
                bit &= bit - 1;
            }
        }
        Store::Array(SortedU16Vec::from_vec_unchecked(vec))
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn min(&self) -> Option<u16> {
        self.bits
            .iter()
            .enumerate()
            .find(|&(_, &bit)| bit != 0)
            .map(|(index, bit)| (index * 64 + (bit.trailing_zeros() as usize)) as u16)
    }

    pub fn max(&self) -> Option<u16> {
        self.bits
            .iter()
            .enumerate()
            .rev()
            .find(|&(_, &bit)| bit != 0)
            .map(|(index, bit)| (index * 64 + (63 - bit.leading_zeros() as usize)) as u16)
    }

    pub fn iter(&self) -> BitmapIter<&[u64; BITMAP_LENGTH]> {
        BitmapIter::new(&self.bits)
    }

    pub fn into_iter(self) -> BitmapIter<Box<[u64; BITMAP_LENGTH]>> {
        BitmapIter::new(self.bits)
    }

    pub fn as_array(&self) -> &[u64; BITMAP_LENGTH] {
        &self.bits
    }
}

impl Default for Bitmap8K {
    fn default() -> Self {
        Bitmap8K::new()
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Cardinality { expected: u64, actual: u64 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Cardinality { expected, actual } => {
                write!(f, "Expected cardinality was {} but was {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for Error {}

pub struct BitmapIter<B: Borrow<[u64; BITMAP_LENGTH]>> {
    key: usize,
    value: u64,
    bits: B,
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> BitmapIter<B> {
    fn new(bits: B) -> BitmapIter<B> {
        BitmapIter { key: 0, value: bits.borrow()[0], bits }
    }
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> Iterator for BitmapIter<B> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if self.value == 0 {
                self.key += 1;
                if self.key >= BITMAP_LENGTH {
                    return None;
                }
                self.value = unsafe { *self.bits.borrow().get_unchecked(self.key) };
                continue;
            }
            let index = self.value.trailing_zeros() as usize;
            self.value &= self.value - 1;
            return Some((64 * self.key + index) as u16);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

#[inline]
pub fn key(index: u16) -> usize {
    index as usize / 64
}

#[inline]
pub fn bit(index: u16) -> usize {
    index as usize % 64
}

#[inline]
fn op_bitmaps(bits1: &mut Bitmap8K, bits2: &Bitmap8K, op: impl Fn(&mut u64, u64)) {
    bits1.len = 0;
    for (index1, &index2) in bits1.bits.iter_mut().zip(bits2.bits.iter()) {
        op(index1, index2);
        bits1.len += index1.count_ones() as u64;
    }
}

impl BitOrAssign<&Self> for Bitmap8K {
    fn bitor_assign(&mut self, rhs: &Self) {
        op_bitmaps(self, rhs, BitOrAssign::bitor_assign);
    }
}

impl BitOrAssign<&SortedU16Vec> for Bitmap8K {
    fn bitor_assign(&mut self, rhs: &SortedU16Vec) {
        for &index in rhs.iter() {
            let (key, bit) = (key(index), bit(index));
            let old_w = self.bits[key];
            let new_w = old_w | 1 << bit;
            self.len += (old_w ^ new_w) >> bit;
            self.bits[key] = new_w;
        }
    }
}

impl BitAndAssign<&Self> for Bitmap8K {
    fn bitand_assign(&mut self, rhs: &Self) {
        op_bitmaps(self, rhs, BitAndAssign::bitand_assign);
    }
}

impl SubAssign<&Self> for Bitmap8K {
    fn sub_assign(&mut self, rhs: &Self) {
        op_bitmaps(self, rhs, |l, r| *l &= !r);
    }
}

impl SubAssign<&SortedU16Vec> for Bitmap8K {
    fn sub_assign(&mut self, rhs: &SortedU16Vec) {
        for &index in rhs.iter() {
            let (key, bit) = (key(index), bit(index));
            let old_w = self.bits[key];
            let new_w = old_w & !(1 << bit);
            self.len -= (old_w ^ new_w) >> bit;
            self.bits[key] = new_w;
        }
    }
}

impl BitXorAssign<&Self> for Bitmap8K {
    fn bitxor_assign(&mut self, rhs: &Self) {
        op_bitmaps(self, rhs, BitXorAssign::bitxor_assign);
    }
}

impl BitXorAssign<&SortedU16Vec> for Bitmap8K {
    fn bitxor_assign(&mut self, rhs: &SortedU16Vec) {
        let mut len = self.len as i64;
        for &index in rhs.iter() {
            let (key, bit) = (key(index), bit(index));
            let old_w = self.bits[key];
            let new_w = old_w ^ 1 << bit;
            len += 1 - 2 * (((1 << bit) & old_w) >> bit) as i64; // +1 or -1
            self.bits[key] = new_w;
        }
        self.len = len as u64;
    }
}
