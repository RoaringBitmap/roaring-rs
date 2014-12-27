use std::{ u32 };
use std::slice::BinarySearchResult::{ Found, NotFound };

use store::Store::{ Array, Bitmap };

pub enum Store {
    Array(Vec<u16>),
    Bitmap([u32, ..2048]),
}

fn set_array(vec: &mut Vec<u16>, index: u16, value: bool) -> bool {
    match (value, vec.binary_search(|elem| elem.cmp(&index))) {
        (false, Found(loc)) => { vec.remove(loc); true },
        (true, NotFound(loc)) => { vec.insert(loc, index); true },
        _ => false,
    }
}

fn set_bitmap(bits: &mut [u32, ..2048], index: u16, value: bool) -> bool {
    let key = (index / (u32::BITS as u16)) as uint;
    let bit = (index % (u32::BITS as u16)) as uint;
    if value {
        if bits[key] & (1 << bit) == 0 {
            bits[key] |= 1 << bit;
            true
        } else {
            false
        }
    } else {
        if bits[key] & (1 << bit) != 0 {
            bits[key] &= !(1 << bit);
            true
        } else {
            false
        }
    }
}

fn get_array(vec: &Vec<u16>, index: u16) -> bool {
    match vec.binary_search(|elem| elem.cmp(&index)) {
        Found(_) => true,
        NotFound(_) => false,
    }
}

fn get_bitmap(bits: &[u32, ..2048], index: u16) -> bool {
    let key = (index / (u32::BITS as u16)) as uint;
    let bit = (index % (u32::BITS as u16)) as uint;
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
    pub fn set(&mut self, index: u16, value: bool) -> bool {
        match *self {
            Array(ref mut vec) => set_array(vec, index, value),
            Bitmap(ref mut bits) => set_bitmap(bits, index, value),
        }
    }

    pub fn get(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => get_array(vec, index),
            Bitmap(ref bits) => get_bitmap(bits, index),
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
