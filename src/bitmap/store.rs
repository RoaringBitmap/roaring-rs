use std::{slice, vec};
use std::borrow::Borrow;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::ops::RangeInclusive;

use self::Store::{Array, Bitmap};

const BITMAP_LENGTH: usize = 1024;

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

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        // A Range is defined as being of size 0 if start >= end.
        if range.is_empty() {
            return 0;
        }

        let start = *range.start();
        let end = *range.end();

        match *self {
            Array(ref mut vec) => {
                // Figure out the starting/ending position in the vec.
                let pos_start = vec.binary_search(&start).unwrap_or_else(|x| x);
                let pos_end = vec
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
                let dropped = vec.splice(pos_start..pos_end, start..=end);

                end as u64 - start as u64 + 1 - dropped.len() as u64
            }
            Bitmap(ref mut bits) => {
                let (start_key, start_bit) = (key(start), bit(start));
                let (end_key, end_bit) = (key(end), bit(end));

                // MSB > start_bit > end_bit > LSB
                if start_key == end_key {
                    // Set the end_bit -> LSB to 1
                    let mut mask = if end_bit == 63 {
                        u64::MAX
                    } else {
                        (1 << (end_bit + 1)) - 1
                    };
                    // Set MSB -> start_bit to 1
                    mask &= !((1 << start_bit) - 1);

                    let existed = (bits[start_key] & mask).count_ones();
                    bits[start_key] |= mask;

                    return u64::from(end - start + 1) - u64::from(existed);
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
                let mask = if end_bit == 63 {
                    u64::MAX
                } else {
                    (1 << (end_bit + 1)) - 1
                };
                existed += (bits[end_key] & mask).count_ones();
                bits[end_key] |= mask;

                end as u64 - start as u64 + 1 - existed as u64
            }
        }
    }

    /// Push the value that must be the new max of the set.
    ///
    /// This function returns whether the value is equal to the
    /// last max. This information is needed to correctly update the
    /// length of the container.
    pub fn push(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => {
                if vec.last().map_or(true, |x| x < &index) {
                    vec.push(index);
                    true
                } else {
                    false
                }
            }
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

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        if range.is_empty() {
            return 0;
        }

        let start = *range.start();
        let end = *range.end();

        match *self {
            Array(ref mut vec) => {
                // Figure out the starting/ending position in the vec.
                let pos_start = vec.binary_search(&start).unwrap_or_else(|x| x);
                let pos_end = vec
                    .binary_search_by(|p| {
                        // binary search the right most position when equals
                        match p.cmp(&end) {
                            Greater => Greater,
                            _ => Less,
                        }
                    })
                    .unwrap_or_else(|x| x);
                vec.drain(pos_start..pos_end);
                (pos_end - pos_start) as u64
            }
            Bitmap(ref mut bits) => {
                let (start_key, start_bit) = (key(start), bit(start));
                let (end_key, end_bit) = (key(end), bit(end));

                if start_key == end_key {
                    let mask = (!0u64 << start_bit) & (!0u64 >> (63 - end_bit));
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
                removed += (bits[end_key] & (!0u64 >> (63 - end_bit))).count_ones();
                bits[end_key] &= !(!0u64 >> (63 - end_bit));
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

    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                fn merge_arrays(arr1: &[u16], arr2: &[u16]) -> Vec<u16> {
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

                let this = std::mem::take(vec1);
                *vec1 = merge_arrays(&this, &vec2);
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec)) => {
                for &index in vec {
                    this.insert(index);
                }
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 |= index2;
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            }
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i = 0;
                vec1.retain(|x| {
                    i += vec2
                        .iter()
                        .skip(i)
                        .position(|y| y >= x)
                        .unwrap_or(vec2.len());
                    vec2.get(i).map_or(false, |y| x == y)
                });
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= index2;
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                vec.retain(|x| store.contains(*x));
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = other.clone();
                new.intersect_with(this);
                *this = new;
            }
        }
    }

    pub fn difference_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i = 0;
                vec1.retain(|x| {
                    i += vec2
                        .iter()
                        .skip(i)
                        .position(|y| y >= x)
                        .unwrap_or(vec2.len());
                    vec2.get(i).map_or(true, |y| x != y)
                });
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    this.remove(*index);
                }
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

    pub fn symmetric_difference_with(&mut self, other: &Self) {
        match (self, other) {
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
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 ^= index2;
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                let mut new = other.clone();
                new.symmetric_difference_with(this);
                *this = new;
            }
        }
    }

    pub fn len(&self) -> u64 {
        match *self {
            Array(ref vec) => vec.len() as u64,
            Bitmap(ref bits) => bits.iter().map(|bit| u64::from(bit.count_ones())).sum(),
        }
    }

    pub fn min(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.first().unwrap(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| index * 64 + (bit.trailing_zeros() as usize))
                .unwrap() as u16,
        }
    }

    pub fn max(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.last().unwrap(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .rev()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| index * 64 + (63 - bit.leading_zeros() as usize))
                .unwrap() as u16,
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
        let new = store.insert_range(6..=1);
        assert_eq!(new, 0);

        assert_eq!(as_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_array_insert_range() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(4..=5);
        assert_eq!(new, 2);

        assert_eq!(as_vec(store), vec![1, 2, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_left_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(2..=5);
        assert_eq!(new, 3);

        assert_eq!(as_vec(store), vec![1, 2, 3, 4, 5, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_right_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(4..=8);
        assert_eq!(new, 4);

        assert_eq!(as_vec(store), vec![1, 2, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_array_insert_range_full_overlap() {
        let mut store = Store::Array(vec![1, 2, 8, 9]);

        let new = store.insert_range(1..=9);
        assert_eq!(new, 5);

        assert_eq!(as_vec(store), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_bitmap_insert_invalid_range() {
        let store = Store::Array(vec![1, 2, 8, 9]);
        let mut store = store.to_bitmap();

        // Insert a range with start > end.
        let new = store.insert_range(6..=1);
        assert_eq!(new, 0);

        assert_eq!(as_vec(store), vec![1, 2, 8, 9]);
    }

    #[test]
    fn test_bitmap_insert_same_key_overlap() {
        let store = Store::Array(vec![1, 2, 3, 62, 63]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(1..=62);
        assert_eq!(new, 58);

        assert_eq!(as_vec(store), (1..64).collect::<Vec<_>>());
    }

    #[test]
    fn test_bitmap_insert_range() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(4..=128);
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

        let new = store.insert_range(1..=128);
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

        let new = store.insert_range(4..=132);
        assert_eq!(new, 128);

        let mut want = vec![1, 2];
        want.extend(4..133);

        assert_eq!(as_vec(store), want);
    }

    #[test]
    fn test_bitmap_insert_range_full_overlap() {
        let store = Store::Array(vec![1, 2, 130]);
        let mut store = store.to_bitmap();

        let new = store.insert_range(1..=134);
        assert_eq!(new, 131);

        let mut want = Vec::new();
        want.extend(1..135);

        assert_eq!(as_vec(store), want);
    }
}
