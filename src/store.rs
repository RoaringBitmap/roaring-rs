use std::iter;
use std::slice;
use std::marker::PhantomData;
use std::cmp::Ordering::{ Equal, Less, Greater };

use num::traits::{ Zero, Bounded };

use util::{ self, ExtInt };
use store::Store::{ Array, Bitmap };

pub enum Store<Size: ExtInt> {
    Array(Vec<Size>),
    Bitmap(Box<[u64]>),
}

pub enum Iter<'a, Size: ExtInt + 'a> {
    Array(slice::Iter<'a, Size>),
    Bitmap(BitmapIter<'a, Size>),
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
                for (key, val) in bits.iter().cloned().enumerate().filter(|&(_, v)| v != 0) {
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
            Array(ref vec) => util::cast(vec.len()),
            Bitmap(ref bits) => {
                bits.iter().map(|bit| bit.count_ones() as u64).sum()
            },
        }
    }

    pub fn min(&self) -> Size {
        match *self {
            Array(ref vec) => *vec.first().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate()
                    .find(|&(_, &bit)| bit != 0)
                    .map(|(index, bit)| util::cast(index * 64 + (bit.trailing_zeros() as usize)))
                    .unwrap()
            },
        }
    }

    pub fn max(&self) -> Size {
        match *self {
            Array(ref vec) => *vec.last().unwrap(),
            Bitmap(ref bits) => {
                bits.iter().enumerate().rev()
                    .find(|&(_, &bit)| bit != 0)
                    .map(|(index, bit)| util::cast(index * 64 + (63 - (bit.leading_zeros() as usize))))
                    .unwrap()
            },
        }
    }

    #[allow(needless_lifetimes)] // TODO: https://github.com/Manishearth/rust-clippy/issues/740
    #[inline]
    pub fn iter(&self) -> Iter<Size> {
        match *self {
            Array(ref vec) => Iter::Array(vec.iter()),
            Bitmap(ref bits) => Iter::Bitmap(BitmapIter::new(bits)),
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
                Bitmap(bits.iter().cloned().collect::<Vec<u64>>().into_boxed_slice())
            },
        }
    }
}

pub struct BitmapIter<'a, Size: ExtInt> {
    key: usize,
    bit: u8,
    bits: &'a Box<[u64]>,
    marker: PhantomData<Size>,
}

impl<'a, Size: ExtInt> BitmapIter<'a, Size> {
    fn new(bits: &'a Box<[u64]>) -> BitmapIter<'a, Size> {
        BitmapIter {
            key: 0,
            bit: Zero::zero(),
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

impl<'a, Size: ExtInt> Iterator for BitmapIter<'a, Size> {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        loop {
            if self.key == self.bits.len() {
                return None;
            } else if (unsafe { self.bits.get_unchecked(self.key) } & (1u64 << util::cast::<u8, usize>(self.bit))) != 0 {
                let result = Some(util::cast::<usize, Size>(self.key * 64 + util::cast::<u8, usize>(self.bit)));
                self.move_next();
                return result;
            } else {
                self.move_next();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
      let min = self.bits.iter().skip(self.key + 1).map(|bits| bits.count_ones()).fold(0, |acc, ones| acc + ones) as usize;
      (min, Some(min + self.bits[self.key].count_ones() as usize))
    }
}

impl<'a, Size: ExtInt> Iterator for Iter<'a, Size> {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match *self {
            Iter::Array(ref mut inner) => inner.next().cloned(),
            Iter::Bitmap(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Iter::Array(ref inner) => inner.size_hint(),
            Iter::Bitmap(ref inner) => inner.size_hint(),
        }
    }
}

#[inline]
fn bitmap_location<Size: ExtInt>(index: Size) -> (usize, usize) { (key(index), bit(index)) }

#[inline]
fn key<Size: ExtInt>(index: Size) -> usize { util::cast(index / util::cast(64u8)) }

#[inline]
fn bit<Size: ExtInt>(index: Size) -> usize { util::cast(index % util::cast(64u8)) }
