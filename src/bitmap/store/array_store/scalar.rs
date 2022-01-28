//! Scalar arithmetic binary set operations on [ArrayStore]'s inner types

use std::cmp::Ordering::*;

#[inline]
pub fn or(lhs: &[u16], rhs: &[u16]) -> Vec<u16> {
    let mut vec = {
        let capacity = (lhs.len() + rhs.len()).min(4096);
        Vec::with_capacity(capacity)
    };

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                vec.push(*a);
                i += 1;
            }
            Greater => {
                vec.push(*b);
                j += 1;
            }
            Equal => {
                vec.push(*a);
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the arrays
    vec.extend_from_slice(&lhs[i..]);
    vec.extend_from_slice(&rhs[j..]);

    vec
}

#[inline]
pub fn and(lhs: &[u16], rhs: &[u16]) -> Vec<u16> {
    let mut vec = Vec::new();

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => i += 1,
            Greater => j += 1,
            Equal => {
                vec.push(*a);
                i += 1;
                j += 1;
            }
        }
    }

    vec
}

#[inline]
pub fn sub(lhs: &[u16], rhs: &[u16]) -> Vec<u16> {
    let mut vec = Vec::new();

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                vec.push(*a);
                i += 1;
            }
            Greater => j += 1,
            Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the left array
    vec.extend_from_slice(&lhs[i..]);
    vec
}

#[inline]
pub fn xor(lhs: &[u16], rhs: &[u16]) -> Vec<u16> {
    let mut vec = Vec::new();

    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                vec.push(*a);
                i += 1;
            }
            Greater => {
                vec.push(*b);
                j += 1;
            }
            Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the arrays
    vec.extend_from_slice(&lhs[i..]);
    vec.extend_from_slice(&rhs[j..]);

    vec
}
