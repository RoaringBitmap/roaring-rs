use std::{ u32 };
use std::slice::BinarySearchResult::{ Found, NotFound };

use store::Store::{ Array, Bitmap };

pub enum Store {
    Array(Vec<u16>),
    Bitmap([u32, ..2048]),
}

fn insert_array(vec: &mut Vec<u16>, index: u16) -> bool {
    match vec.binary_search(|elem| elem.cmp(&index)) {
        NotFound(loc) => { vec.insert(loc, index); true },
        _ => false,
    }
}

fn remove_array(vec: &mut Vec<u16>, index: u16) -> bool {
    match vec.binary_search(|elem| elem.cmp(&index)) {
        Found(loc) => { vec.remove(loc); true },
        _ => false,
    }
}

fn bitmap_location(index: u16) -> (uint, uint) {
    ((index / (u32::BITS as u16)) as uint, (index % (u32::BITS as u16)) as uint)
}

fn insert_bitmap(bits: &mut [u32, ..2048], index: u16) -> bool {
    let (key, bit) = bitmap_location(index);
    if bits[key] & (1 << bit) == 0 {
        bits[key] |= 1 << bit;
        true
    } else {
        false
    }
}

fn remove_bitmap(bits: &mut [u32, ..2048], index: u16) -> bool {
    let (key, bit) = bitmap_location(index);
    if bits[key] & (1 << bit) != 0 {
        bits[key] &= !(1 << bit);
        true
    } else {
        false
    }
}

fn contains_array(vec: &Vec<u16>, index: u16) -> bool {
    match vec.binary_search(|elem| elem.cmp(&index)) {
        Found(_) => true,
        NotFound(_) => false,
    }
}

fn contains_bitmap(bits: &[u32, ..2048], index: u16) -> bool {
    let (key, bit) = bitmap_location(index);
    bits[key] & (1 << bit) != 0
}

fn bitmap_to_array(bits: &[u32, ..2048]) -> Vec<u16> {
    let mut vec = Vec::new();
    for key in 0..2048 {
        if bits[key] == 0 {
            continue
        }
        for bit in 0..32 {
            if (bits[key] & (1 << bit)) != 0 {
                vec.push((key * u32::BITS + bit) as u16);
            }
        }
    }
    vec
}

fn array_to_bitmap(vec: &Vec<u16>) -> [u32, ..2048] {
    let mut bits = [0, ..2048];
    for index in vec.iter() {
        let key = (*index / (u32::BITS as u16)) as uint;
        let bit = (*index % (u32::BITS as u16)) as uint;
        bits[key] |= 1 << bit;
    }
    bits
}

fn is_disjoint_array(vec1: &Vec<u16>, vec2: &Vec<u16>) -> bool {
    let result: bool;
    let mut iter1 = vec1.iter();
    let mut iter2 = vec2.iter();
    let mut index1 = iter1.next();
    let mut index2 = iter2.next();
    loop {
        match (index1, index2) {
            (Some(i1), Some(i2)) if i1 == i2 => {
                result = false;
                break;
            },
            (Some(i1), Some(i2)) if i1 < i2 => index1 = iter1.next(),
            (Some(i1), Some(i2)) if i1 > i2 => index2 = iter2.next(),
            (_, _) => {
                result = true;
                break;
            },
        }
    }
    result
}

fn is_disjoint_bitmap(bits1: &[u32, ..2048], bits2: &[u32, ..2048]) -> bool {
    let mut result = true;
    for (index1, index2) in bits1.iter().zip(bits2.iter()) {
        if *index1 & *index2 != 0 {
            result = false;
            break;
        }
    }
    result
}

fn is_disjoint_array_bitmap(vec: &Vec<u16>, bits: &[u32, ..2048]) -> bool {
    let mut result = true;
    for index in vec.iter() {
        if contains_bitmap(bits, *index) {
            result = false;
            break;
        }
    }
    result
}

fn is_subset_array(vec1: &Vec<u16>, vec2: &Vec<u16>) -> bool {
    let result: bool;
    let mut iter1 = vec1.iter();
    let mut iter2 = vec2.iter();
    let mut index1 = iter1.next();
    let mut index2 = iter2.next();
    loop {
        match (index1, index2) {
            (Some(i1), Some(i2)) if i1 == i2 => {
                index1 = iter1.next();
                index2 = iter2.next();
            },
            (Some(i1), Some(i2)) if i1 < i2 => {
                result = false;
                break;
            },
            (Some(i1), Some(i2)) if i1 > i2 => index2 = iter2.next(),
            (Some(_), Some(_)) => panic!(),
            (None, _) => {
                result = true;
                break;
            },
            (_, None) => {
                result = false;
                break;
            },
        }
    }
    result
}

fn is_subset_bitmap(bits1: &[u32, ..2048], bits2: &[u32, ..2048]) -> bool {
    let mut result = true;
    for (index1, index2) in bits1.iter().zip(bits2.iter()) {
        if *index1 & *index2 != *index1 {
            result = false;
            break;
        }
    }
    result
}

fn is_subset_array_bitmap(vec: &Vec<u16>, bits: &[u32, ..2048]) -> bool {
    let mut result = true;
    for index in vec.iter() {
        if !contains_bitmap(bits, *index) {
            result = false;
            break;
        }
    }
    result
}

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match self {
            &Array(ref mut vec) => insert_array(vec, index),
            &Bitmap(ref mut bits) => insert_bitmap(bits, index),
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match self {
            &Array(ref mut vec) => remove_array(vec, index),
            &Bitmap(ref mut bits) => remove_bitmap(bits, index),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match self {
            &Array(ref vec) => contains_array(vec, index),
            &Bitmap(ref bits) => contains_bitmap(bits, index),
        }
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => is_disjoint_array(vec1, vec2),
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => is_disjoint_bitmap(bits1, bits2),
            (&Array(ref vec), &Bitmap(ref bits)) => is_disjoint_array_bitmap(vec, bits),
            (&Bitmap(ref bits), &Array(ref vec)) => is_disjoint_array_bitmap(vec, bits),
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => is_subset_array(vec1, vec2),
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => is_subset_bitmap(bits1, bits2),
            (&Array(ref vec), &Bitmap(ref bits)) => is_subset_array_bitmap(vec, bits),
            (&Bitmap(..), &Array(..)) => false,
        }
    }

    pub fn to_array(&self) -> Store {
        match self {
            &Array(_) => panic!("Cannot convert array to array"),
            &Bitmap(ref bits) => Array(bitmap_to_array(bits)),
        }
    }

    pub fn to_bitmap(&self) -> Store {
        match self {
            &Array(ref vec) => Bitmap(array_to_bitmap(vec)),
            &Bitmap(_) => panic!("Cannot convert bitmap to bitmap"),
        }
    }
}
