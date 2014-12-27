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

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => insert_array(vec, index),
            Bitmap(ref mut bits) => insert_bitmap(bits, index),
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => remove_array(vec, index),
            Bitmap(ref mut bits) => remove_bitmap(bits, index),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => contains_array(vec, index),
            Bitmap(ref bits) => contains_bitmap(bits, index),
        }
    }
}

impl Store {
    pub fn to_array(&self) -> Store {
        match *self {
            Array(_) => panic!("Cannot convert array to array"),
            Bitmap(ref bits) => Array(bitmap_to_array(bits)),
        }
    }

    pub fn to_bitmap(&self) -> Store {
        match *self {
            Array(ref vec) => Bitmap(array_to_bitmap(vec)),
            Bitmap(_) => panic!("Cannot convert bitmap to bitmap"),
        }
    }
}
