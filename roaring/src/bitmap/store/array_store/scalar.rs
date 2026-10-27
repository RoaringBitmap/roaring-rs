//! Scalar arithmetic binary set operations on `ArrayStore`'s inner types

use crate::bitmap::store::array_store::visitor::BinaryOperationVisitor;
use core::cmp::Ordering::*;

#[inline]
pub fn or(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                visitor.visit_scalar(*a);
                i += 1;
            }
            Greater => {
                visitor.visit_scalar(*b);
                j += 1;
            }
            Equal => {
                visitor.visit_scalar(*a);
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the arrays
    visitor.visit_slice(&lhs[i..]);
    visitor.visit_slice(&rhs[j..]);
}

#[inline]
pub fn and(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
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
                visitor.visit_scalar(*a);
                i += 1;
                j += 1;
            }
        }
    }
}

/// Intersects a small array with a much larger one, binary searching `large` for four
/// elements of `small` at a time. Based on CRoaring's `intersect_skewed_uint16`.
// Out of line so the merge loop in the same callers compiles as it does without it.
#[inline(never)]
pub fn and_skewed(small: &[u16], large: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    let mut large = large;
    let (chunks, remainder) = small.as_chunks::<4>();
    for &targets in chunks {
        if large.is_empty() {
            return;
        }
        let indexes = lower_bounds(large, targets);
        for (target, index) in targets.into_iter().zip(indexes) {
            if large.get(index) == Some(&target) {
                visitor.visit_scalar(target);
            }
        }
        large = &large[indexes[3]..];
    }
    for &a in remainder {
        match large.binary_search(&a) {
            Ok(i) => {
                visitor.visit_scalar(a);
                large = &large[i + 1..];
            }
            Err(i) => large = &large[i..],
        }
    }
}

/// Binary searches for four targets at once, interleaved so they can overlap.
/// `array` must not be empty.
#[inline]
fn lower_bounds(array: &[u16], targets: [u16; 4]) -> [usize; 4] {
    let mut bases = [0; 4];
    let mut n = array.len();
    while n > 1 {
        let half = n / 2;
        for (base, target) in bases.iter_mut().zip(targets) {
            if array[*base + half] < target {
                *base += half;
            }
        }
        n -= half;
    }
    core::array::from_fn(|i| bases[i] + (array[bases[i]] < targets[i]) as usize)
}

#[inline]
pub fn sub(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                visitor.visit_scalar(*a);
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
    visitor.visit_slice(&lhs[i..]);
}

#[inline]
pub fn xor(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // Traverse both arrays
    let mut i = 0;
    let mut j = 0;
    while i < lhs.len() && j < rhs.len() {
        let a = unsafe { lhs.get_unchecked(i) };
        let b = unsafe { rhs.get_unchecked(j) };
        match a.cmp(b) {
            Less => {
                visitor.visit_scalar(*a);
                i += 1;
            }
            Greater => {
                visitor.visit_scalar(*b);
                j += 1;
            }
            Equal => {
                i += 1;
                j += 1;
            }
        }
    }

    // Store remaining elements of the arrays
    visitor.visit_slice(&lhs[i..]);
    visitor.visit_slice(&rhs[j..]);
}
