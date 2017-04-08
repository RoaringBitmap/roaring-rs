use std::slice;
use std::vec;
use std::borrow::Borrow;
use std::cmp::Ordering::{ Equal, Less, Greater };
use std::marker::PhantomData;

const BITMAP_LENGTH: usize = 1024;

use self::Store::{ Array, Bitmap };
pub enum Store {
    Array(Vec<u16>),
    Bitmap(Box<[u64; BITMAP_LENGTH]>),
}

pub enum Iter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<'a, &'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<'a, Box<[u64; BITMAP_LENGTH]>>),
}

pub struct BitmapIter<'a, B: Borrow<[u64; BITMAP_LENGTH]> + 'a> {
    key: usize,
    bit: usize,
    bits: B,
    marker: PhantomData<&'a B>,
}

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map_err(|loc| vec.insert(loc, index))
                    .is_err()
            },
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) == 0 {
                    bits[key] |= 1 << bit;
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map(|loc| vec.remove(loc))
                    .is_ok()
            },
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) != 0 {
                    bits[key] &= !(1 << bit);
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn remove_range(&mut self, start: u32, end: u32) -> u64 {
        debug_assert!(start < end, "caller must ensure start < end");
        match *self {
            Array(ref mut vec) => {
                let a = vec.binary_search(&(start as u16)).unwrap_or_else(|e| e);
                let b = if end > u16::max_value() as u32 {
                    vec.len()
                } else {
                    vec.binary_search(&(end as u16)).unwrap_or_else(|e| e)
                };
                vec.drain(a..b);
                (b - a) as u64
            },
            Bitmap(ref mut bits) => {
                let start_key = key(start as u16) as usize;
                let start_bit =  bit(start as u16) as u32;
                // end_key is inclusive
                let end_key = key((end - 1) as u16) as usize;
                let end_bit = bit(end as u16) as u32;

                if start_key == end_key {
                    let mask = (!0u64 << start_bit) & (!0u64).wrapping_shr(64 - end_bit);
                    let removed = (bits[start_key] & mask).count_ones();
                    bits[start_key] &= !mask;
                    return removed as u64;
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
                removed as u64
            },
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => vec.binary_search(&index).is_ok(),
            Bitmap(ref bits) => bits[key(index)] & (1 << bit(index)) != 0
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
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(&i1, &i2)| (i1 & i2) == 0)
            },
            (&Array(ref vec), store @ &Bitmap(..)) | (store @ &Bitmap(..), &Array(ref vec)) => {
                vec.iter().all(|&i| !store.contains(i))
            },
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
                            },
                            Less => return false,
                            Greater => value2 = i2.next(),
                        },
                    }
                }
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(&i1, &i2)| (i1 & i2) == i1)
            },
            (&Array(ref vec), store @ &Bitmap(..)) => {
                vec.iter().all(|&i| store.contains(i))
            },
            (&Bitmap(..), &Array(..)) => false,
        }
    }

    pub fn to_array(&self) -> Self {
        match *self {
            Array(..) => panic!("Cannot convert array to array"),
            Bitmap(ref bits) => {
                let mut vec = Vec::new();
                for (key, val) in bits.iter().cloned().enumerate().filter(|&(_, v)| v != 0) {
                    for bit in 0..64 {
                        if (val & (1 << bit)) != 0 {
                            vec.push(key as u16 * 64 + bit as u16);
                        }
                    }
                }
                Array(vec)
            },
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
            },
            Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
        }
    }

    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0;
                let mut iter2 = vec2.iter();
                'outer: for &index2 in &mut iter2 {
                    while i1 < vec1.len() {
                        match vec1[i1].cmp(&index2) {
                            Less => i1 += 1,
                            Greater => vec1.insert(i1, index2),
                            Equal => continue 'outer,
                        }
                    }
                    vec1.push(index2);
                    break
                }
                vec1.extend(iter2);
            },
            (ref mut this @ &mut Bitmap(..), &Array(ref vec)) => {
                for &index in vec {
                    this.insert(index);
                }
            },
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 |= index2;
                }
            },
            (this @ &mut Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            },
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None | Some(Less) => { vec1.remove(i1); },
                        Some(Greater) => { current2 = iter2.next(); },
                        Some(Equal) => {
                            i1 += 1;
                            current2 = iter2.next();
                        },
                    }
                }
            },
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= index2;
                }
            },
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                for i in (0..(vec.len())).rev() {
                    if !store.contains(vec[i]) {
                        vec.remove(i);
                    }
                }
            },
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = other.clone();
                new.intersect_with(this);
                *this = new;
            },
        }
    }

    pub fn difference_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None => break,
                        Some(Less) => { i1 += 1; },
                        Some(Greater) => { current2 = iter2.next(); },
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        },
                    }
                }
            },
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    this.remove(*index);
                }
            },
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= !*index2;
                }
            },
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                for i in (0 .. vec.len()).rev() {
                    if store.contains(vec[i]) {
                        vec.remove(i);
                    }
                }
            },
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
                        Some(Less) => { i1 += 1; },
                        Some(Greater) => {
                            vec1.insert(i1, *current2.unwrap());
                            i1 += 1;
                            current2 = iter2.next();
                        },
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        },
                    }
                }
                if current2.is_some() {
                    vec1.push(*current2.unwrap());
                    vec1.extend(iter2.cloned());
                }
            },
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    if this.contains(*index) {
                        this.remove(*index);
                    } else {
                        this.insert(*index);
                    }
                }
            },
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 ^= index2;
                }
            },
            (this @ &mut Array(..), &Bitmap(..)) => {
                let mut new = other.clone();
                new.symmetric_difference_with(this);
                *this = new;
            },
        }
    }

    pub fn len(&self) -> u64 {
        match *self {
            Array(ref vec) => vec.len() as u64,
            Bitmap(ref bits) => {
                bits.iter().map(|bit| bit.count_ones() as u64).sum()
            },
        }
    }

    pub fn min(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.first().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate()
                    .find(|&(_, &bit)| bit != 0)
                    .map(|(index, bit)| index * 64 + (bit.trailing_zeros() as usize))
                    .unwrap() as u16
            },
        }
    }

    pub fn max(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.last().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate().rev()
                    .find(|&(_, &bit)| bit != 0)
                    .map(|(index, bit)| index * 64 + (63 - bit.leading_zeros() as usize))
                    .unwrap() as u16
            },
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
            (&Array(ref vec1), &Array(ref vec2)) => {
                vec1 == vec2
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            },
            _ => false,
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        match *self {
            Array(ref vec) => Array(vec.clone()),
            Bitmap(ref bits) => {
                Bitmap(Box::new(**bits))
            },
        }
    }
}

impl<'a, B: Borrow<[u64; BITMAP_LENGTH]> + 'a> BitmapIter<'a, B> {
    fn new(bits: B) -> BitmapIter<'a, B> {
        BitmapIter {
            key: 0,
            bit: 0,
            bits: bits,
            marker: PhantomData,
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

impl<'a, B: Borrow<[u64; BITMAP_LENGTH]> + 'a> Iterator for BitmapIter<'a, B> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if self.key == BITMAP_LENGTH {
                return None;
            } else if (unsafe { self.bits.borrow().get_unchecked(self.key) } & (1u64 << self.bit)) != 0 {
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
fn key(index: u16) -> usize { index as usize / 64 }

#[inline]
fn bit(index: u16) -> usize { index as usize % 64 }
