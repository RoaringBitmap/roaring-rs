use std::{ u16, u32 };
use std::ptr;
use std::num::Int;
use std::slice::BinarySearchResult::{ Found, NotFound };

use store::Store::{ Array, Bitmap };

pub enum Store {
    Array(Vec<u16>),
    Bitmap([u32; 2048]),
}

fn bitmap_location(index: u16) -> (uint, uint) {
    ((index / (u32::BITS as u16)) as uint, (index % (u32::BITS as u16)) as uint)
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

fn insert_bitmap(bits: &mut [u32; 2048], index: u16) -> bool {
    let (key, bit) = bitmap_location(index);
    if bits[key] & (1 << bit) == 0 {
        bits[key] |= 1 << bit;
        true
    } else {
        false
    }
}

fn remove_bitmap(bits: &mut [u32; 2048], index: u16) -> bool {
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

fn contains_bitmap(bits: &[u32; 2048], index: u16) -> bool {
    let (key, bit) = bitmap_location(index);
    bits[key] & (1 << bit) != 0
}

fn bitmap_to_array(bits: &[u32; 2048]) -> Vec<u16> {
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
    vec
}

fn array_to_bitmap(vec: &Vec<u16>) -> [u32; 2048] {
    let mut bits = [0; 2048];
    for index in vec.iter() {
        let (key, bit) = bitmap_location(*index);
        bits[key] |= 1 << bit;
    }
    bits
}

fn is_disjoint_array(vec1: &Vec<u16>, vec2: &Vec<u16>) -> bool {
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
}

fn is_disjoint_bitmap(bits1: &[u32; 2048], bits2: &[u32; 2048]) -> bool {
    for (index1, index2) in bits1.iter().zip(bits2.iter()) {
        if *index1 & *index2 != 0 {
            return false;
        }
    }
    return true;
}

fn is_disjoint_array_bitmap(vec: &Vec<u16>, bits: &[u32; 2048]) -> bool {
    for index in vec.iter() {
        if contains_bitmap(bits, *index) {
            return false;
        }
    }
    return true;
}

fn is_subset_array(vec1: &Vec<u16>, vec2: &Vec<u16>) -> bool {
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
}

fn is_subset_bitmap(bits1: &[u32; 2048], bits2: &[u32; 2048]) -> bool {
    for (index1, index2) in bits1.iter().zip(bits2.iter()) {
        if *index1 & *index2 != *index1 {
            return false;
        }
    }
    return true;
}

fn is_subset_array_bitmap(vec: &Vec<u16>, bits: &[u32; 2048]) -> bool {
    for index in vec.iter() {
        if !contains_bitmap(bits, *index) {
            return false;
        }
    }
    return true;
}

fn union_with_array(vec1: &mut Vec<u16>, vec2: &Vec<u16>) {
    for index in vec2.iter() {
        insert_array(vec1, *index);
    }
}

fn union_with_bitmap(bits1: &mut [u32; 2048], bits2: &[u32; 2048]) {
    for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
        *index1 |= *index2;
    }
}

fn union_with_bitmap_array(bits: &mut [u32; 2048], vec: &Vec<u16>) {
    for index in vec.iter() {
        insert_bitmap(bits, *index);
    }
}

fn union_with(mut this: &mut Store, other: &Store) {
    match (&mut this, other) {
        (& &Array(ref mut vec1), &Array(ref vec2)) => union_with_array(vec1, vec2),
        (& &Bitmap(ref mut bits1), &Bitmap(ref bits2)) => union_with_bitmap(bits1, bits2),
        (& &Bitmap(ref mut bits), &Array(ref vec)) => union_with_bitmap_array(bits, vec),
        (& &Array(_), &Bitmap(_)) => {
            *this = this.to_bitmap();
            this.union_with(other);
        },
    }
}

fn intersect_with_array(vec1: &mut Vec<u16>, vec2: &Vec<u16>) {
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
}

fn intersect_with_bitmap(bits1: &mut [u32; 2048], bits2: &[u32; 2048]) {
    for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
        *index1 &= *index2;
    }
}

fn intersect_with_array_bitmap(vec: &mut Vec<u16>, bits: &[u32; 2048]) {
    let mut i = 0;
    while i < vec.len() {
        if contains_bitmap(bits, vec[i]) {
            i += 1;
        } else {
            vec.remove(i);
        }
    }
}

fn intersect_with(mut this: &mut Store, other: &Store) {
    match (&mut this, other) {
        (& &Array(ref mut vec1), &Array(ref vec2)) => intersect_with_array(vec1, vec2),
        (& &Bitmap(ref mut bits1), &Bitmap(ref bits2)) => intersect_with_bitmap(bits1, bits2),
        (& &Array(ref mut vec), &Bitmap(ref bits)) => intersect_with_array_bitmap(vec, bits),
        (& &Bitmap(_), &Array(_)) => {
            *this = this.to_array();
            this.intersect_with(other);
        },
    }
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

    pub fn to_array(&self) -> Self {
        match self {
            &Array(_) => panic!("Cannot convert array to array"),
            &Bitmap(ref bits) => Array(bitmap_to_array(bits)),
        }
    }

    pub fn to_bitmap(&self) -> Self {
        match self {
            &Array(ref vec) => Bitmap(array_to_bitmap(vec)),
            &Bitmap(_) => panic!("Cannot convert bitmap to bitmap"),
        }
    }

    pub fn union_with(&mut self, other: &Self) {
        union_with(self, other);
    }

    pub fn intersect_with(&mut self, other: &Self) {
        intersect_with(self, other);
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
                unsafe {
                    ptr::copy_memory(&mut new_bits, bits, 2048);
                }
                Bitmap(new_bits)
            },
        }
    }
}
