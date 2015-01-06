use std::{ u16, u32 };
use std::num::Int;

use store::Store::{ Array, Bitmap };

pub enum Store {
    Array(Vec<u16>),
    Bitmap([u32; 2048]),
}

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match self {
            &Array(ref mut vec) => {
                match vec.binary_search_by(|elem| elem.cmp(&index)) {
                    Err(loc) => { vec.insert(loc, index); true },
                    Ok(..) => false,
                }
            },
            &Bitmap(ref mut bits) => {
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

    pub fn remove(&mut self, index: u16) -> bool {
        match self {
            &Array(ref mut vec) => {
                match vec.binary_search_by(|elem| elem.cmp(&index)) {
                    Ok(loc) => { vec.remove(loc); true },
                    Err(..) => false,
                }
            },
            &Bitmap(ref mut bits) => {
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

    pub fn contains(&self, index: u16) -> bool {
        match self {
            &Array(ref vec) => {
                match vec.binary_search_by(|elem| elem.cmp(&index)) {
                    Ok(..) => true,
                    Err(..) => false,
                }
            },
            &Bitmap(ref bits) => {
                let (key, bit) = bitmap_location(index);
                bits[key] & (1 << bit) != 0
            },
        }
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let (mut i1, mut i2) = (vec1.iter(), vec2.iter());
                let (mut value1, mut value2) = (i1.next(), i2.next());
                loop {
                    match (value1, value2) {
                        (None, _) | (_, None) => return true,
                        (v1, v2) if v1 == v2 => return false,
                        (v1, v2) if v1 < v2 => value1 = i1.next(),
                        (v1, v2) if v1 > v2 => value2 = i2.next(),
                        (_, _) => panic!(),
                    }
                }
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter().zip(bits2.iter()) {
                    if *index1 & *index2 != 0 {
                        return false;
                    }
                }
                return true;
            },
            (&Array(ref vec), store @ &Bitmap(..)) | (store @ &Bitmap(..), &Array(ref vec)) => {
                for &index in vec.iter() {
                    if store.contains(index) {
                        return false;
                    }
                }
                return true;
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
                        (_, None) => return false,
                        (v1, v2) if v1 == v2 => {
                            value1 = i1.next();
                            value2 = i2.next();
                        },
                        (v1, v2) if v1 < v2 => return false,
                        (v1, v2) if v1 > v2 => value2 = i2.next(),
                        (_, _) => panic!(),
                    }
                }
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter().zip(bits2.iter()) {
                    if *index1 & *index2 != *index1 {
                        return false;
                    }
                }
                return true;
            },
            (&Array(ref vec), store @ &Bitmap(..)) => {
                for &index in vec.iter() {
                    if !store.contains(index) {
                        return false;
                    }
                }
                return true;
            },
            (&Bitmap(..), &Array(..)) => false,
        }
    }

    pub fn to_array(&self) -> Self {
        match self {
            &Array(..) => panic!("Cannot convert array to array"),
            &Bitmap(ref bits) => {
                let mut vec = Vec::new();
                for (key, val) in bits.iter().map(|v| *v).enumerate() {
                    if val == 0 {
                        continue
                    }
                    for bit in 0..(u32::BITS) {
                        if (val & (1 << bit)) != 0 {
                            vec.push((key * u32::BITS + bit) as u16);
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
                let mut bits = [0; 2048];
                for index in vec.iter() {
                    let (key, bit) = bitmap_location(*index);
                    bits[key] |= 1 << bit;
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
            (&Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 |= index2;
                }
            },
            (this @ &Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            },
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        match (self, other) {
            (&Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0u;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match (vec1[i1], current2) {
                        (_, None) => { vec1.remove(i1); },
                        (ref val1, Some(val2)) if val1 < val2 => { vec1.remove(i1); },
                        (ref val1, Some(val2)) if val1 > val2 => { current2 = iter2.next(); },
                        (ref val1, Some(val2)) if val1 == val2 => {
                            i1 += 1;
                            current2 = iter2.next();
                        },
                        _ => panic!("Should not be possible to get here"),
                    }
                }
            },
            (&Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= *index2;
                }
            },
            (&Array(ref mut vec), store @ &Bitmap(..)) => {
                let mut i = 0;
                while i < vec.len() {
                    if store.contains(vec[i]) {
                        i += 1;
                    } else {
                        vec.remove(i);
                    }
                }
            },
            (this @ &Bitmap(..), &Array(..)) => {
                *this = this.to_array();
                this.intersect_with(other);
            },
        }
    }

    pub fn difference_with(&mut self, other: &Self) {
        match (self, other) {
            (ref mut this @ _, &Array(ref vec2)) => {
                for index in vec2.iter() {
                    this.remove(*index);
                }
            },
            (&Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= !*index2;
                }
            },
            (&Array(ref mut vec), store @ &Bitmap(..)) => {
                for i in range(0, vec.len()).rev() {
                    if store.contains(vec[i]) {
                        vec.remove(i);
                    }
                }
            },
        }
    }

    pub fn symmetric_difference_with(&mut self, other: &Self) {
        match (self, other) {
            (ref mut this, &Array(ref vec2)) => {
                for index in vec2.iter() {
                    if this.contains(*index) {
                        this.remove(*index);
                    } else {
                        this.insert(*index);
                    }
                }
            },
            (&Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 ^= *index2;
                }
            },
            (this @ &Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.symmetric_difference_with(other);
            },
        }
    }

    pub fn len(&self) -> u16 {
        match self {
            &Array(ref vec) => vec.len() as u16,
            &Bitmap(ref bits) => {
                let mut len = 0;
                for bit in bits.iter() {
                    len += bit.count_ones()
                }
                len as u16
            },
        }
    }

    pub fn min(&self) -> u16 {
        match self {
            &Array(ref vec) => vec[0],
            &Bitmap(ref bits) => {
                for (index, bit) in bits.iter().enumerate() {
                    if *bit != 0 {
                        return (index * u32::BITS + bit.leading_zeros()) as u16;
                    }
                }
                return u16::MIN;
            },
        }
    }

    pub fn max(&self) -> u16 {
        match self {
            &Array(ref vec) => vec[vec.len() - 1],
            &Bitmap(ref bits) => {
                for (index, bit) in bits.iter().enumerate().rev() {
                    if *bit != 0 {
                        return ((index + 1) * u32::BITS - bit.trailing_zeros()) as u16;
                    }
                }
                return u16::MAX;
            },
        }
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Store) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                vec1 == vec2
            },
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).map(|(i1, i2)| i1 == i2).fold(true, |acc, n| acc & n)
            },
            _ => false,
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        match self {
            &Array(ref vec) => Array(vec.clone()),
            &Bitmap(ref bits) => {
                let mut new_bits = [0u32; 2048];
                for (i1, &i2) in new_bits.iter_mut().zip(bits.iter()) {
                    *i1 = i2;
                }
                Bitmap(new_bits)
            },
        }
    }
}

#[inline]
fn bitmap_location(index: u16) -> (uint, uint) {
    ((index / (u32::BITS as u16)) as uint, (index % (u32::BITS as u16)) as uint)
}
