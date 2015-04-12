use std::iter;
use std::marker::PhantomData;
use std::mem::size_of;
use std::cmp::Ordering::{ Equal, Less, Greater };

use num::traits::{ One, Bounded };

use util::{ self, ExtInt };
use store::Store::{ Array, Bitmap };

pub enum Store<Size: ExtInt> {
    Array(Vec<Size>),
    Bitmap(Box<[u64]>),
}

impl<Size: ExtInt> Store<Size> {
    pub fn insert(&mut self, index: Size) -> bool {
        match self {
            &mut Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map_err(|loc| vec.insert(loc, index))
                    .is_err()
            },
            &mut Bitmap(ref mut bits) => {
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
        match self {
            &mut Array(ref mut vec) => {
                vec.binary_search(&index)
                    .map(|loc| vec.remove(loc))
                    .is_ok()
            },
            &mut Bitmap(ref mut bits) => {
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
        match self {
            &Array(ref vec) => vec.binary_search(&index).is_ok(),
            &Bitmap(ref bits) => bits[key(index)] & (1 << bit(index)) != 0
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
        match self {
            &Array(..) => panic!("Cannot convert array to array"),
            &Bitmap(ref bits) => {
                let mut vec = Vec::new();
                for (key, val) in bits.iter().map(|v| *v).enumerate().filter(|&(_, v)| v != 0) {
                    for bit in 0..(size_of::<u64>()) {
                        if (val & (1 << bit)) != 0 {
                            vec.push(util::cast(key * (size_of::<u64>() as usize) + (bit as usize)));
                        }
                    }
                }
                Array(vec)
            },
        }
    }

    pub fn to_bitmap(&self) -> Self {
        match self {
            &Array(ref vec) => {
                let one: Size = One::one();
                let count = one.rotate_right(6);
                let mut bits = iter::repeat(0).take(util::cast(count)).collect::<Vec<u64>>().into_boxed_slice();
                for &index in vec.iter() {
                    bits[key(index)] |= 1 << bit(index);
                }
                Bitmap(bits)
            },
            &Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
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

    pub fn len(&self) -> Size {
        match self {
            &Array(ref vec) => util::cast(vec.len()),
            &Bitmap(ref bits) => {
                let mut len = 0;
                for bit in bits.iter() {
                    len += bit.count_ones();
                }
                util::cast(len)
            },
        }
    }

    pub fn min(&self) -> Size {
        match self {
            &Array(ref vec) => vec[0],
            &Bitmap(ref bits) => {
                bits.iter().enumerate()
                    .filter(|&(_, &bit)| bit != 0)
                    .next().map(|(index, bit)| util::cast(index * (size_of::<u64>() as usize) + (bit.leading_zeros() as usize)))
                    .unwrap()
            },
        }
    }

    pub fn max(&self) -> Size {
        match self {
            &Array(ref vec) => vec[vec.len() - 1],
            &Bitmap(ref bits) => {
                bits.iter().enumerate().rev()
                    .filter(|&(_, &bit)| bit != 0)
                    .next().map(|(index, bit)| util::cast(index * (size_of::<u64>() as usize) + (bit.leading_zeros() as usize)))
                    .unwrap()
            },
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = Size> + 'a> {
        match self {
            &Array(ref vec) => Box::new(vec.iter().map(|x| *x)),
            &Bitmap(ref bits) => Box::new(BitmapIter::new(bits)),
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
        match self {
            &Array(ref vec) => Array(vec.clone()),
            &Bitmap(ref bits) => {
                Bitmap(bits.iter().map(|&i| i).collect::<Vec<u64>>().into_boxed_slice())
            },
        }
    }
}

struct BitmapIter<'a, Size: ExtInt> {
    key: usize,
    bit: u8,
    bits: &'a Box<[u64]>,
    marker: PhantomData<Size>,
}

impl<'a, Size: ExtInt> BitmapIter<'a, Size> {
    fn new(bits: &'a Box<[u64]>) -> BitmapIter<'a, Size> {
        BitmapIter {
            key: 0,
            bit: Bounded::max_value(),
            bits: bits,
            marker: PhantomData,
        }
    }
}

impl<'a, Size: ExtInt> Iterator for BitmapIter<'a, Size> {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        loop {
            self.bit += 1;
            if self.bit == util::cast(size_of::<u64>()) {
                self.bit = 0;
                self.key += 1;
            }
            if self.key == self.bits.len() {
                return None;
            }
            if (self.bits[self.key] & (1u64 << util::cast::<u8, usize>(self.bit))) != 0 {
                return Some(util::cast::<usize, Size>(self.key * (size_of::<u64>() as usize) + util::cast::<u8, usize>(self.bit)));
            }
        }
    }
}

#[inline]
fn bitmap_location<Size: ExtInt>(index: Size) -> (usize, usize) { (key(index), bit(index)) }

#[inline]
fn key<Size: ExtInt>(index: Size) -> usize { util::cast(index / util::cast(size_of::<u64>())) }

#[inline]
fn bit<Size: ExtInt>(index: Size) -> usize { util::cast(index % util::cast(size_of::<u64>())) }
