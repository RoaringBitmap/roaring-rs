use std::iter;
use std::marker::PhantomData;
use std::cmp::Ordering::{ Equal, Less, Greater };

use num::traits::{ Zero, Bounded };

use util::{ self, ExtInt };
use store::Store::{ Array, Bitmap };

pub enum Store<Size: ExtInt> {
    Array(Vec<Size>),
    Bitmap(Box<[u64]>),
}

impl<Size: ExtInt> Store<Size> {
    pub fn insert(&mut self, index: Size) -> bool {
        match *self {
            Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map_err(|loc| vec.insert(loc, index))
                    .is_err()
            },
            Bitmap(ref mut bits) => {
                let (key, bit) = bitmap_location(index);
                if bits[key] & (1 << bit) == 0 {
                    bits[key] |= 1 << bit;
                    true
                } else {
                    false
                }
            },
        }
    }

    pub fn remove(&mut self, index: Size) -> bool {
        match *self {
            Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map(|loc| vec.remove(loc))
                    .is_ok()
            },
            Bitmap(ref mut bits) => {
                let (key, bit) = bitmap_location(index);
                if bits[key] & (1 << bit) != 0 {
                    bits[key] &= !(1 << bit);
                    true
                } else {
                    false
                }
            },
        }
    }

    #[inline]
    pub fn contains(&self, index: Size) -> bool {
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
                for (key, val) in bits.iter().map(|v| *v).enumerate().filter(|&(_, v)| v != 0) {
                    for bit in 0..64 {
                        if (val & (1 << bit)) != 0 {
                            vec.push(util::cast(key * 64 + (bit as usize)));
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
                let count = util::cast::<Size, usize>(Bounded::max_value()) / 64 + 1;
                let mut bits = iter::repeat(0).take(count).collect::<Vec<u64>>().into_boxed_slice();
                for &index in vec.iter() {
                    bits[key(index)] |= 1 << bit(index);
                }
                Bitmap(bits)
            },
            Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
        }
    }

    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (ref mut this, &Array(ref vec)) => {
                for &index in vec.iter() {
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

    pub fn symmetric_difference(&self, other: &Self) -> Self {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let mut result = Vec::new();
                let mut iter1 = vec1.iter();
                let mut current1 = iter1.next();
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while current1.is_some() && current2.is_some() {
                    match current1.unwrap().cmp(current2.unwrap()) {
                        Less => {
                            result.push(*current1.unwrap());
                            current1 = iter1.next();
                        },
                        Greater => {
                            result.push(*current2.unwrap());
                            current2 = iter2.next();
                        },
                        Equal => {
                            current1 = iter1.next();
                            current2 = iter2.next();
                        },
                    }
                }
                if current1.is_some() {
                    result.push(*current1.unwrap());
                    result.extend(iter1.map(|&x| x));
                }
                if current2.is_some() {
                    result.push(*current2.unwrap());
                    result.extend(iter2.map(|&x| x));
                }
                Array(result)
            },
            (ref this @ &Bitmap(..), &Array(ref vec2)) => {
                let mut result = (*this).clone();
                for index in vec2.iter() {
                    if result.contains(*index) {
                        result.remove(*index);
                    } else {
                        result.insert(*index);
                    }
                }
                result
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                Bitmap(bits1.iter().zip(bits2.iter()).map(|(index1, index2)| *index1 ^ *index2).collect::<Vec<u64>>().into_boxed_slice())
            },
            (this @ &Array(..), &Bitmap(..)) => {
                other.symmetric_difference(this)
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
                    vec1.extend(iter2.map(|&x| x));
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
            Array(ref vec) => util::cast(vec.len()),
            Bitmap(ref bits) => {
                let mut len = 0;
                for bit in bits.iter() {
                    len += bit.count_ones();
                }
                util::cast(len)
            },
        }
    }

    pub fn min(&self) -> Size {
        match *self {
            Array(ref vec) => *vec.first().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate()
                    .filter(|&(_, &bit)| bit != 0)
                    .next().map(|(index, bit)| util::cast(index * 64 + (bit.trailing_zeros() as usize)))
                    .unwrap()
            },
        }
    }

    pub fn max(&self) -> Size {
        match *self {
            Array(ref vec) => *vec.last().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate().rev()
                    .filter(|&(_, &bit)| bit != 0)
                    .next().map(|(index, bit)| util::cast(index * 64 + (63 - (bit.leading_zeros() as usize))))
                    .unwrap()
            },
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> Box<DoubleEndedIterator<Item = Size> + 'a> {
        match *self {
            Array(ref vec) => Box::new(vec.iter().map(|x| *x)),
            Bitmap(ref bits) => Box::new(BitmapIter::new(bits)),
        }
    }

}

impl<Size: ExtInt> PartialEq for Store<Size> {
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

impl<Size: ExtInt> Clone for Store<Size> {
    fn clone(&self) -> Self {
        match *self {
            Array(ref vec) => Array(vec.clone()),
            Bitmap(ref bits) => {
                Bitmap(bits.iter().map(|&i| i).collect::<Vec<u64>>().into_boxed_slice())
            },
        }
    }
}

struct BitmapIter<'a, Size: ExtInt> {
    fwd_key: usize,
    bwd_key: usize,
    fwd_bit: u8,
    bwd_bit: u8,
    overflow: bool,
    bits: &'a Box<[u64]>,
    marker: PhantomData<Size>,
}

impl<'a, Size: ExtInt> BitmapIter<'a, Size> {
    fn new(bits: &'a Box<[u64]>) -> BitmapIter<'a, Size> {
        BitmapIter {
            fwd_key: 0,
            bwd_key: bits.len() - 1,
            fwd_bit: Zero::zero(),
            bwd_bit: 63,
            bits: bits,
            overflow: false,
            marker: PhantomData,
        }
    }
}


impl<'a, Size: ExtInt> BitmapIter<'a, Size> {
    fn move_next(&mut self) {
        if self.fwd_bit == 63 {
            if self.fwd_key == self.bits.len() - 1 {
                self.overflow = true;
            } else {
                self.fwd_bit = 0;
                self.fwd_key += 1;
            }
        } else {
            self.fwd_bit += 1;
        }
    }

    fn move_next_back(&mut self) {
        if self.bwd_bit == 0 {
            if self.bwd_key == 0 {
                self.overflow = true;
            } else {
                self.bwd_bit = 63;
                self.bwd_key -= 1;
            }
        } else {
            self.bwd_bit -= 1;
        }
    }
}

impl<'a, Size: ExtInt> Iterator for BitmapIter<'a, Size> {
    type Item = Size;

    fn size_hint(&self) -> (usize, Option<usize>) {
      let min = self.bits.iter().skip(self.fwd_key + 1).take(self.bwd_key.checked_sub(self.fwd_key).and_then(|i| i.checked_sub(2)).unwrap_or(0)).map(|bits| bits.count_ones()).fold(0, |acc, ones| acc + ones) as usize;
      (min, Some(min + self.bits[self.fwd_key].count_ones() as usize + self.bits[self.bwd_key].count_ones() as usize))
    }

    fn next(&mut self) -> Option<Size> {
        loop {
            if self.overflow || self.fwd_key > self.bwd_key || (self.fwd_key == self.bwd_key && self.fwd_bit > self.bwd_bit) {
                return None;
            } else {
                if (self.bits[self.fwd_key] & (1u64 << util::cast::<u8, usize>(self.fwd_bit))) != 0 {
                    let result = Some(util::cast::<usize, Size>(self.fwd_key * 64 + util::cast::<u8, usize>(self.fwd_bit)));
                    self.move_next();
                    return result;
                } else {
                    self.move_next();
                }
            }
        }
    }
}

impl<'a, Size: ExtInt> DoubleEndedIterator for BitmapIter<'a, Size> {
    fn next_back(&mut self) -> Option<Size> {
        loop {
            if self.overflow || self.fwd_key > self.bwd_key || (self.fwd_key == self.bwd_key && self.fwd_bit > self.bwd_bit) {
                return None;
            } else {
                if (self.bits[self.bwd_key] & (1u64 << util::cast::<u8, usize>(self.bwd_bit))) != 0 {
                    let result = Some(util::cast::<usize, Size>(self.bwd_key * 64 + util::cast::<u8, usize>(self.bwd_bit)));
                    self.move_next_back();
                    return result;
                } else {
                    self.move_next_back();
                }
            }
        }
    }
}

#[inline]
fn bitmap_location<Size: ExtInt>(index: Size) -> (usize, usize) { (key(index), bit(index)) }

#[inline]
fn key<Size: ExtInt>(index: Size) -> usize { util::cast(index / util::cast(64u8)) }

#[inline]
fn bit<Size: ExtInt>(index: Size) -> usize { util::cast(index % util::cast(64u8)) }
