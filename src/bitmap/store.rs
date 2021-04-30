use std::cmp::Ordering::{Equal, Greater, Less};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXorAssign, SubAssign};
use std::{borrow::Borrow, ops::Range};
use std::{mem, slice, vec};

const BITMAP_LENGTH: usize = 1024;

use self::Store::{Array, Bitmap};
pub enum Store {
    Array(Vec<u16>),
    Bitmap(Box<[u64; BITMAP_LENGTH]>),
}

pub enum Iter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<&'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<Box<[u64; BITMAP_LENGTH]>>),
}

pub struct BitmapIter<B: Borrow<[u64; BITMAP_LENGTH]>> {
    key: usize,
    bit: usize,
    bits: B,
}

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => vec
                .binary_search(&index)
                .map_err(|loc| vec.insert(loc, index))
                .is_err(),
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) == 0 {
                    bits[key] |= 1 << bit;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn insert_range(&mut self, range: Range<u16>) -> u64 {
        // A Range is defined as being of size 0 if start >= end.
        if range.is_empty() {
            return 0;
        }

        match *self {
            Array(ref mut vec) => {
                // Figure out the starting/ending position in the vec
                let pos_start = vec.binary_search(&range.start).unwrap_or_else(|x| x);
                let pos_end = vec.binary_search(&range.end).unwrap_or_else(|x| x);

                // Overwrite the range in the middle - there's no need to take
                // into account any existing elements between start and end, as
                // they're all being added to the set.
                let dropped = vec.splice(pos_start..pos_end, range.clone());

                u64::from(range.end - range.start) - dropped.len() as u64
            }
            Bitmap(ref mut bits) => {
                let (start_key, start_bit) = (key(range.start), bit(range.start));
                let (end_key, end_bit) = (key(range.end), bit(range.end));

                if start_key == end_key {
                    // Set the end_bit -> LSB to 1
                    let mut mask = (1 << end_bit) - 1;
                    // Set start_bit -> LSB to 0
                    mask &= !((1 << start_bit) - 1);
                    // Leaving end_bit -> start_bit set to 1

                    let existed = (bits[start_key] & mask).count_ones();
                    bits[start_key] |= mask;

                    return u64::from(range.end - range.start) - u64::from(existed);
                }

                // Mask off the left-most bits (MSB -> start_bit)
                let mask = !((1 << start_bit) - 1);

                // Keep track of the number of bits that were already set to
                // return how many new bits were set later
                let mut existed = (bits[start_key] & mask).count_ones();

                bits[start_key] |= mask;

                // Set the full blocks, tracking the number of set bits
                for i in (start_key + 1)..end_key {
                    existed += bits[i].count_ones();
                    bits[i] = u64::MAX;
                }

                // Set the end bits in the last chunk (MSB -> end_bit)
                let mask = (1 << end_bit) - 1;
                existed += (bits[end_key] & mask).count_ones();
                bits[end_key] |= mask;

                u64::from(range.end - range.start) - u64::from(existed)
            }
        }
    }

    /// Push `index` at the end of the store only if `index` is the new max.
    ///
    /// Returns whether `index` was effectively pushed.
    pub fn push(&mut self, index: u16) -> bool {
        if self.max().map_or(true, |max| max < index) {
            match self {
                Array(vec) => vec.push(index),
                Bitmap(bits) => {
                    let (key, bit) = (key(index), bit(index));
                    bits[key] |= 1 << bit;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => vec.binary_search(&index).map(|loc| vec.remove(loc)).is_ok(),
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) != 0 {
                    bits[key] &= !(1 << bit);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn remove_range(&mut self, start: u32, end: u32) -> u64 {
        debug_assert!(start < end, "caller must ensure start < end");
        match *self {
            Array(ref mut vec) => {
                let a = vec.binary_search(&(start as u16)).unwrap_or_else(|e| e);
                let b = if end > u32::from(u16::max_value()) {
                    vec.len()
                } else {
                    vec.binary_search(&(end as u16)).unwrap_or_else(|e| e)
                };
                vec.drain(a..b);
                (b - a) as u64
            }
            Bitmap(ref mut bits) => {
                let start_key = key(start as u16) as usize;
                let start_bit = bit(start as u16) as u32;
                // end_key is inclusive
                let end_key = key((end - 1) as u16) as usize;
                let end_bit = bit(end as u16) as u32;

                if start_key == end_key {
                    let mask = (!0u64 << start_bit) & (!0u64).wrapping_shr(64 - end_bit);
                    let removed = (bits[start_key] & mask).count_ones();
                    bits[start_key] &= !mask;
                    return u64::from(removed);
                }

                let mut removed = 0;
                // start key bits
                removed += (bits[start_key] & (!0u64 << start_bit)).count_ones();
                bits[start_key] &= !(!0u64 << start_bit);
                // counts bits in between
                for word in &bits[start_key + 1..end_key] {
                    removed += word.count_ones();
                    // When popcnt is available zeroing in this loop is faster,
                    // but we opt to perform reasonably on most cpus by zeroing after.
                    // By doing that the compiler uses simd to count ones.
                }
                // do zeroing outside the loop
                for word in &mut bits[start_key + 1..end_key] {
                    *word = 0;
                }
                // end key bits
                removed += (bits[end_key] & (!0u64).wrapping_shr(64 - end_bit)).count_ones();
                bits[end_key] &= !(!0u64).wrapping_shr(64 - end_bit);
                u64::from(removed)
            }
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => vec.binary_search(&index).is_ok(),
            Bitmap(ref bits) => bits[key(index)] & (1 << bit(index)) != 0,
        }
    }

    pub fn is_disjoint<'a>(&'a self, other: &'a Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let (mut i1, mut i2) = (vec1.iter(), vec2.iter());
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
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => bits1
                .iter()
                .zip(bits2.iter())
                .all(|(&i1, &i2)| (i1 & i2) == 0),
            (&Array(ref vec), store @ &Bitmap(..)) | (store @ &Bitmap(..), &Array(ref vec)) => {
                vec.iter().all(|&i| !store.contains(i))
            }
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let (mut i1, mut i2) = (vec1.iter(), vec2.iter());
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
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => bits1
                .iter()
                .zip(bits2.iter())
                .all(|(&i1, &i2)| (i1 & i2) == i1),
            (&Array(ref vec), store @ &Bitmap(..)) => vec.iter().all(|&i| store.contains(i)),
            (&Bitmap(..), &Array(..)) => false,
        }
    }

    pub fn to_array(&self) -> Self {
        match *self {
            Array(..) => panic!("Cannot convert array to array"),
            Bitmap(ref bits) => {
                let mut vec = Vec::new();
                for (index, mut bit) in bits.iter().cloned().enumerate() {
                    while bit != 0 {
                        vec.push((u64::trailing_zeros(bit) + (64 * index as u32)) as u16);
                        bit &= bit - 1;
                    }
                }
                Array(vec)
            }
        }
    }

    pub fn to_bitmap(&self) -> Self {
        match *self {
            Array(ref vec) => {
                let mut bits = Box::new([0; BITMAP_LENGTH]);
                for &index in vec {
                    bits[key(index)] |= 1 << bit(index);
                }
                Bitmap(bits)
            }
            Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
        }
    }

    pub fn len(&self) -> u64 {
        match *self {
            Array(ref vec) => vec.len() as u64,
            Bitmap(ref bits) => bits.iter().map(|bit| u64::from(bit.count_ones())).sum(),
        }
    }

    pub fn min(&self) -> Option<u16> {
        match *self {
            Array(ref vec) => vec.first().copied(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| (index * 64 + (bit.trailing_zeros() as usize)) as u16),
        }
    }

    pub fn max(&self) -> Option<u16> {
        match *self {
            Array(ref vec) => vec.last().copied(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .rev()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| (index * 64 + (63 - bit.leading_zeros() as usize)) as u16),
        }
    }
}

impl BitOr<&Store> for &Store {
    type Output = Store;

    fn bitor(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (&Array(ref vec1), &Array(ref vec2)) => Array(union_arrays(vec1, vec2)),
            (&Bitmap(_), &Array(_)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Bitmap(_), &Bitmap(_)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Array(_), &Bitmap(_)) => {
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
                *vec1 = union_arrays(vec1, vec2);
            }
            (this @ &mut Bitmap(..), &mut Array(ref vec)) => {
                vec.iter().for_each(|index| {
                    this.insert(*index);
                });
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitOrAssign::bitor_assign(index1, index2);
                }
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
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let this = mem::take(vec1);
                *vec1 = union_arrays(&this, &vec2);
            }
            (this @ &mut Bitmap(..), &Array(ref vec)) => {
                vec.iter().for_each(|index| {
                    this.insert(*index);
                });
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitOrAssign::bitor_assign(index1, index2);
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                BitOrAssign::bitor_assign(this, rhs);
            }
        }
    }
}

impl BitAnd<&Store> for &Store {
    type Output = Store;

    fn bitand(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (&Array(ref vec1), &Array(ref vec2)) => Array(intersect_arrays(vec1, vec2)),
            (&Bitmap(_), &Array(_)) => {
                let mut rhs = rhs.clone();
                BitAndAssign::bitand_assign(&mut rhs, self);
                rhs
            }
            (&Bitmap(_), &Bitmap(_)) => {
                let mut lhs = self.clone();
                BitAndAssign::bitand_assign(&mut lhs, rhs);
                lhs
            }
            (&Array(_), &Bitmap(_)) => {
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
            (&mut Array(ref mut lhs), &mut Array(ref mut rhs)) => {
                if rhs.len() < lhs.len() {
                    mem::swap(lhs, rhs);
                }

                let mut i = 0;
                lhs.retain(|x| {
                    i += rhs.iter().skip(i).position(|y| y >= x).unwrap_or(rhs.len());
                    rhs.get(i).map_or(false, |y| x == y)
                });
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitAndAssign::bitand_assign(index1, index2);
                }
            }
            (&mut Array(ref mut vec), store @ &mut Bitmap(..)) => {
                vec.retain(|x| store.contains(*x));
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
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let (mut lhs, rhs) = if vec1.len() <= vec2.len() {
                    (mem::take(vec1), vec2.as_slice())
                } else {
                    (vec2.clone(), vec1.as_slice())
                };

                let mut i = 0;
                lhs.retain(|x| {
                    i += rhs.iter().skip(i).position(|y| y >= x).unwrap_or(rhs.len());
                    rhs.get(i).map_or(false, |y| x == y)
                });

                *vec1 = lhs;
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitAndAssign::bitand_assign(index1, index2);
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                vec.retain(|x| store.contains(*x));
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = rhs.clone();
                BitAndAssign::bitand_assign(&mut new, &*this);
                *this = new;
            }
        }
    }
}

impl SubAssign<&Store> for Store {
    fn sub_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut lhs), &Array(ref rhs)) => {
                let mut i = 0;
                lhs.retain(|x| {
                    i += rhs.iter().skip(i).position(|y| y >= x).unwrap_or(rhs.len());
                    rhs.get(i).map_or(true, |y| x != y)
                });
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                vec2.iter().for_each(|index| {
                    this.remove(*index);
                });
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= !*index2;
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                vec.retain(|x| !store.contains(*x));
            }
        }
    }
}

impl BitXorAssign<Store> for Store {
    fn bitxor_assign(&mut self, mut rhs: Store) {
        // TODO improve this function
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref mut vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None => break,
                        Some(Less) => {
                            i1 += 1;
                        }
                        Some(Greater) => {
                            vec1.insert(i1, *current2.unwrap());
                            i1 += 1;
                            current2 = iter2.next();
                        }
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        }
                    }
                }
                if let Some(current) = current2 {
                    vec1.push(*current);
                    vec1.extend(iter2.cloned());
                }
            }
            (ref mut this @ &mut Bitmap(..), &mut Array(ref mut vec2)) => {
                for index in vec2 {
                    if this.contains(*index) {
                        this.remove(*index);
                    } else {
                        this.insert(*index);
                    }
                }
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref mut bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitXorAssign::bitxor_assign(index1, index2);
                }
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
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None => break,
                        Some(Less) => {
                            i1 += 1;
                        }
                        Some(Greater) => {
                            vec1.insert(i1, *current2.unwrap());
                            i1 += 1;
                            current2 = iter2.next();
                        }
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        }
                    }
                }
                if let Some(current) = current2 {
                    vec1.push(*current);
                    vec1.extend(iter2.cloned());
                }
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    if this.contains(*index) {
                        this.remove(*index);
                    } else {
                        this.insert(*index);
                    }
                }
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    BitXorAssign::bitxor_assign(index1, index2);
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                let mut new = rhs.clone();
                BitXorAssign::bitxor_assign(&mut new, &*this);
                *this = new;
            }
        }
    }
}

impl<'a> IntoIterator for &'a Store {
    type Item = u16;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        match *self {
            Array(ref vec) => Iter::Array(vec.iter()),
            Bitmap(ref bits) => Iter::BitmapBorrowed(BitmapIter::new(&**bits)),
        }
    }
}

impl IntoIterator for Store {
    type Item = u16;
    type IntoIter = Iter<'static>;
    fn into_iter(self) -> Iter<'static> {
        match self {
            Array(vec) => Iter::Vec(vec.into_iter()),
            Bitmap(bits) => Iter::BitmapOwned(BitmapIter::new(bits)),
        }
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => vec1 == vec2,
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            _ => false,
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        match *self {
            Array(ref vec) => Array(vec.clone()),
            Bitmap(ref bits) => Bitmap(Box::new(**bits)),
        }
    }
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> BitmapIter<B> {
    fn new(bits: B) -> BitmapIter<B> {
        BitmapIter {
            key: 0,
            bit: 0,
            bits,
        }
    }

    fn move_next(&mut self) {
        self.bit += 1;
        if self.bit == 64 {
            self.bit = 0;
            self.key += 1;
        }
    }
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> Iterator for BitmapIter<B> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if self.key == BITMAP_LENGTH {
                return None;
            } else if (unsafe { self.bits.borrow().get_unchecked(self.key) } & (1u64 << self.bit))
                != 0
            {
                let result = Some((self.key * 64 + self.bit) as u16);
                self.move_next();
                return result;
            } else {
                self.move_next();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match *self {
            Iter::Array(ref mut inner) => inner.next().cloned(),
            Iter::Vec(ref mut inner) => inner.next(),
            Iter::BitmapBorrowed(ref mut inner) => inner.next(),
            Iter::BitmapOwned(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

#[inline]
fn union_arrays(arr1: &[u16], arr2: &[u16]) -> Vec<u16> {
    let len = (arr1.len() + arr2.len()).min(4096);
    let mut out = Vec::with_capacity(len);

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < arr1.len() && j < arr2.len() {
        let a = unsafe { arr1.get_unchecked(i) };
        let b = unsafe { arr2.get_unchecked(j) };
        match a.cmp(&b) {
            Less => {
                out.push(*a);
                i += 1
            }
            Greater => {
                out.push(*b);
                j += 1
            }
            Equal => {
                out.push(*a);
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the arrays
    out.extend_from_slice(&arr1[i..]);
    out.extend_from_slice(&arr2[j..]);

    out
}

#[inline]
fn intersect_arrays(arr1: &[u16], arr2: &[u16]) -> Vec<u16> {
    let mut out = Vec::new();

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < arr1.len() && j < arr2.len() {
        let a = unsafe { arr1.get_unchecked(i) };
        let b = unsafe { arr2.get_unchecked(j) };
        match a.cmp(&b) {
            Less => i += 1,
            Greater => j += 1,
            Equal => {
                out.push(*a);
                i += 1;
                j += 1;
            }
        }
    }

    out
}

#[inline]
fn key(index: u16) -> usize {
    index as usize / 64
}

#[inline]
fn bit(index: u16) -> usize {
    index as usize % 64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_vec(s: Store) -> Vec<u16> {
        if let Store::Array(v) = s {
            return v;
        }
        as_vec(s.to_array())
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_array_insert_invalid_range() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        // Insert a range with start > end.
        let new = store.insert_range(6..1);
        assert_eq!(new, 0);

        assert_eq!(as_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_array_insert_range() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(4..6);
        assert_eq!(new, 2);

        assert_eq!(as_vec(store), vec![1, 2, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_left_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(2..6);
        assert_eq!(new, 3);

        assert_eq!(as_vec(store), vec![1, 2, 3, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_right_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(4..9);
        assert_eq!(new, 4);

        assert_eq!(as_vec(store), vec![1, 2, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_full_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(1..10);
        assert_eq!(new, 5);

        assert_eq!(as_vec(store), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_bitmap_insert_invalid_range() {
        let store = Store::Array(vec![1, 2, 8, 9]);
        let mut store = store.to_bitmap();

        // Insert a range with start > end.
        let new = store.insert_range(6..1);
        assert_eq!(new, 0);

        assert_eq!(as_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_bitmap_insert_same_key_overlap() {
        let store = Store::Array(vec![1, 2, 3, 62, 63]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(1..63);
        assert_eq!(new, 58);

        assert_eq!(as_vec(store), (1..64).collect::<Vec<_>>());
    }

    #[test]
    fn test_bitmap_insert_range() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(4..129);
        assert_eq!(new, 125);

        let mut want = vec![1, 2];
        want.extend(4..129);
        want.extend(&[130]);

        assert_eq!(as_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_left_overlap() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(1..129);
        assert_eq!(new, 126);

        let mut want = Vec::new();
        want.extend(1..129);
        want.extend(&[130]);

        assert_eq!(as_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_right_overlap() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(4..133);
        assert_eq!(new, 128);

        let mut want = vec![1, 2];
        want.extend(4..133);

        assert_eq!(as_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_full_overlap() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(1..135);
        assert_eq!(new, 131);

        let mut want = Vec::new();
        want.extend(1..135);

        assert_eq!(as_vec(store), want);
    }
}
