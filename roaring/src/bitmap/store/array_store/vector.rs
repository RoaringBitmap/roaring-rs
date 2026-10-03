//! Ported from CRoaring and arXiv:1709.07821
//! Lemire et al, Roaring Bitmaps: Implementation of an Optimized Software Library
//!
//! Prior work: Schlegel et al., Fast Sorted-Set Intersection using SIMD Instructions
//!
//! Rust port notes:
//! The x86 PCMPESTRM instruction has been replaced with a portable all-pairs comparison
//! (see `matrix_cmp_u16`), which takes more instructions but is available on every SIMD level.
//!
//! The public functions select a SIMD level at runtime with `fearless_simd::dispatch!`, and call
//! implementations generic over the SIMD level. Those are annotated with `#[simd]`, so that they
//! are compiled with the matching target features enabled.
//!
//! The small helpers they call are `#[inline(always)]` rather than `#[simd]`: they get the target
//! features by being inlined, and `#[simd]` does not force inlining, which can leave them out of
//! line in the hot loops.

#![cfg(feature = "simd")]

use super::scalar;
use crate::bitmap::store::array_store::visitor::BinaryOperationVisitor;
use fearless_simd::prelude::*;
use fearless_simd::{dispatch, mask16x8, u16x8, Level};
use fearless_simd_macros::simd;

/// The number of lanes in a `u16x8`
const LANES: usize = 8;

/// The SIMD level to use.
///
/// With `std`, this is detected at runtime on x86/x86-64 (and cached by `fearless_simd`).
/// Otherwise, it is determined by the target features enabled at compile time.
#[inline]
fn level() -> Level {
    #[cfg(feature = "std")]
    return Level::new();
    #[cfg(not(feature = "std"))]
    return Level::baseline();
}

// a one-pass union algorithm
pub fn or(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    dispatch!(level(), simd => or_impl(simd, lhs, rhs, visitor))
}

#[simd]
fn or_impl<S: Simd>(simd: S, lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // De-duplicates `slice` in place
    // Returns the end index of the deduplicated slice.
    // elements after the return value are not guaranteed to be unique or in order
    #[inline]
    fn dedup(slice: &mut [u16]) -> usize {
        let mut pos: usize = 1;
        for i in 1..slice.len() {
            if slice[i] != slice[i - 1] {
                slice[pos] = slice[i];
                pos += 1;
            }
        }
        pos
    }

    // returns `new`, with a mask of its values which were not already written,
    // assuming that the previously written vector was `old`
    #[inline(always)]
    fn handle_vector<S: Simd>(old: u16x8<S>, new: u16x8<S>) -> (u16x8<S>, u8) {
        // `[old[7], new[0], ..., new[6]]`
        let tmp: u16x8<S> = old.slide::<7>(new);
        let mask = 255 - tmp.simd_eq(new).to_bitmask() as u8;
        (new, mask)
    }

    if (lhs.len() < 8) || (rhs.len() < 8) {
        scalar::or(lhs, rhs, visitor);
        return;
    }

    let len1: usize = lhs.len() / 8;
    let len2: usize = rhs.len() / 8;

    let v_a: u16x8<S> = load(simd, lhs);
    let v_b: u16x8<S> = load(simd, rhs);
    let [mut v_min, mut v_max] = simd_merge_u16(v_a, v_b);

    let mut i = 1;
    let mut j = 1;
    let (v, m) = handle_vector(u16x8::splat(simd, u16::MAX), v_min);
    visitor.visit_vector(v, m);
    let mut v_prev: u16x8<S> = v_min;
    if (i < len1) && (j < len2) {
        let mut v: u16x8<S>;
        let mut cur_a: u16 = lhs[8 * i];
        let mut cur_b: u16 = rhs[8 * j];
        loop {
            if cur_a <= cur_b {
                v = load(simd, &lhs[8 * i..]);
                i += 1;
                if i < len1 {
                    cur_a = lhs[8 * i];
                } else {
                    break;
                }
            } else {
                v = load(simd, &rhs[8 * j..]);
                j += 1;
                if j < len2 {
                    cur_b = rhs[8 * j];
                } else {
                    break;
                }
            }
            [v_min, v_max] = simd_merge_u16(v, v_max);
            let (v, m) = handle_vector(v_prev, v_min);
            visitor.visit_vector(v, m);
            v_prev = v_min;
        }
        [v_min, v_max] = simd_merge_u16(v, v_max);
        let (v, m) = handle_vector(v_prev, v_min);
        visitor.visit_vector(v, m);
        v_prev = v_min;
    }

    debug_assert!(i == len1 || j == len2);

    // we finish the rest off using a scalar algorithm
    // could be improved?
    //
    // copy the small end on a tmp buffer
    let mut buffer: [u16; 16] = [0; 16];
    let (v, m) = handle_vector(v_prev, v_max);
    store(swizzle_to_front(v, m), buffer.as_mut_slice());
    let mut rem = m.count_ones() as usize;

    let (tail_a, tail_b, tail_len) = if i == len1 {
        (&lhs[8 * i..], &rhs[8 * j..], lhs.len() - 8 * len1)
    } else {
        (&rhs[8 * j..], &lhs[8 * i..], rhs.len() - 8 * len2)
    };

    buffer[rem..rem + tail_len].copy_from_slice(tail_a);
    rem += tail_len;

    if rem == 0 {
        visitor.visit_slice(tail_b)
    } else {
        buffer[..rem].sort_unstable();
        rem = dedup(&mut buffer[..rem]);
        scalar::or(&buffer[..rem], tail_b, visitor);
    }
}

pub fn and(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    dispatch!(level(), simd => and_impl(simd, lhs, rhs, visitor))
}

#[simd]
fn and_impl<S: Simd>(simd: S, lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    let st_a = (lhs.len() / LANES) * LANES;
    let st_b = (rhs.len() / LANES) * LANES;

    let mut i: usize = 0;
    let mut j: usize = 0;
    if (i < st_a) && (j < st_b) {
        let mut v_a: u16x8<S> = load(simd, &lhs[i..]);
        let mut v_b: u16x8<S> = load(simd, &rhs[j..]);
        loop {
            let mask = matrix_cmp_u16(v_a, v_b).to_bitmask() as u8;
            visitor.visit_vector(v_a, mask);

            let a_max: u16 = lhs[i + LANES - 1];
            let b_max: u16 = rhs[j + LANES - 1];
            if a_max <= b_max {
                i += LANES;
                if i == st_a {
                    break;
                }
                v_a = load(simd, &lhs[i..]);
            }
            if b_max <= a_max {
                j += LANES;
                if j == st_b {
                    break;
                }
                v_b = load(simd, &rhs[j..]);
            }
        }
    }

    // intersect the tail using scalar intersection
    scalar::and(&lhs[i..], &rhs[j..], visitor);
}

// a one-pass xor algorithm
pub fn xor(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    dispatch!(level(), simd => xor_impl(simd, lhs, rhs, visitor))
}

#[simd]
fn xor_impl<S: Simd>(simd: S, lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    /// De-duplicates `slice` in place, removing _both_ duplicates
    /// Returns the end index of the xor-ed slice.
    /// elements after the return value are not guaranteed to be unique or in order
    #[inline]
    fn xor_slice(slice: &mut [u16]) -> usize {
        let mut pos: usize = 1;
        for i in 1..slice.len() {
            if slice[i] != slice[i - 1] {
                slice[pos] = slice[i];
                pos += 1;
            } else {
                pos -= 1; // it is identical to previous, delete it
            }
        }
        pos
    }

    // returns the vector to write, and a mask of the values to write from it,
    // omitting repeated values assuming that previously written vector was "old"
    #[inline(always)]
    fn handle_vector<S: Simd>(old: u16x8<S>, new: u16x8<S>) -> (u16x8<S>, u8) {
        // `[old[6], old[7], new[0], ..., new[5]]`
        let tmp1: u16x8<S> = old.slide::<6>(new);
        // `[old[7], new[0], ..., new[6]]`
        let tmp2: u16x8<S> = old.slide::<7>(new);
        let eq_l: mask16x8<S> = tmp2.simd_eq(tmp1);
        let eq_r: mask16x8<S> = tmp2.simd_eq(new);
        let eq_l_or_r: mask16x8<S> = eq_l | eq_r;
        let mask: u8 = eq_l_or_r.to_bitmask() as u8;
        (tmp2, 255 - mask)
    }

    if (lhs.len() < 8) || (rhs.len() < 8) {
        scalar::xor(lhs, rhs, visitor);
        return;
    }

    let len1: usize = lhs.len() / 8;
    let len2: usize = rhs.len() / 8;

    let v_a: u16x8<S> = load(simd, lhs);
    let v_b: u16x8<S> = load(simd, rhs);
    let [mut v_min, mut v_max] = simd_merge_u16(v_a, v_b);

    let mut i = 1;
    let mut j = 1;
    let (v, m) = handle_vector(u16x8::splat(simd, u16::MAX), v_min);
    visitor.visit_vector(v, m);
    let mut v_prev: u16x8<S> = v_min;
    if (i < len1) && (j < len2) {
        let mut v: u16x8<S>;
        let mut cur_a: u16 = lhs[8 * i];
        let mut cur_b: u16 = rhs[8 * j];
        loop {
            if cur_a <= cur_b {
                v = load(simd, &lhs[8 * i..]);
                i += 1;
                if i < len1 {
                    cur_a = lhs[8 * i];
                } else {
                    break;
                }
            } else {
                v = load(simd, &rhs[8 * j..]);
                j += 1;
                if j < len2 {
                    cur_b = rhs[8 * j];
                } else {
                    break;
                }
            }
            [v_min, v_max] = simd_merge_u16(v, v_max);
            let (v, m) = handle_vector(v_prev, v_min);
            visitor.visit_vector(v, m);
            v_prev = v_min;
        }
        [v_min, v_max] = simd_merge_u16(v, v_max);
        let (v, m) = handle_vector(v_prev, v_min);
        visitor.visit_vector(v, m);
        v_prev = v_min;
    }

    debug_assert!(i == len1 || j == len2);

    // we finish the rest off using a scalar algorithm
    // could be improved?
    // conditionally stores the last value of laststore as well as all but the
    // last value of vecMax,
    let mut buffer: [u16; 17] = [0; 17];
    // remaining size
    let (v, m) = handle_vector(v_prev, v_max);
    store(swizzle_to_front(v, m), buffer.as_mut_slice());
    let mut rem = m.count_ones() as usize;

    // Store `v_max` rather than reading its lanes with `as_array()`, which makes LLVM split
    // `v_max` into pieces in the merge loop above
    let mut arr_max = [0u16; LANES];
    store(v_max, &mut arr_max);
    let vec7 = arr_max[7];
    let vec6 = arr_max[6];
    if vec6 != vec7 {
        buffer[rem] = vec7;
        rem += 1;
    }

    let (tail_a, tail_b, tail_len) = if i == len1 {
        (&lhs[8 * i..], &rhs[8 * j..], lhs.len() - 8 * len1)
    } else {
        (&rhs[8 * j..], &lhs[8 * i..], rhs.len() - 8 * len2)
    };

    buffer[rem..rem + tail_len].copy_from_slice(tail_a);
    rem += tail_len;

    if rem == 0 {
        visitor.visit_slice(tail_b)
    } else {
        buffer[..rem].sort_unstable();
        rem = xor_slice(&mut buffer[..rem]);
        scalar::xor(&buffer[..rem], tail_b, visitor);
    }
}

pub fn sub(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    dispatch!(level(), simd => sub_impl(simd, lhs, rhs, visitor))
}

#[simd]
fn sub_impl<S: Simd>(simd: S, lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // we handle the degenerate cases
    if lhs.is_empty() {
        return;
    } else if rhs.is_empty() {
        visitor.visit_slice(lhs);
        return;
    }

    let st_a = (lhs.len() / LANES) * LANES;
    let st_b = (rhs.len() / LANES) * LANES;

    let mut i = 0;
    let mut j = 0;
    if (i < st_a) && (j < st_b) {
        let mut v_a: u16x8<S> = load(simd, &lhs[i..]);
        let mut v_b: u16x8<S> = load(simd, &rhs[j..]);
        // we have a running mask which indicates which values from a have been
        // spotted in b, these don't get written out.
        let mut runningmask_a_found_in_b: u8 = 0;
        loop {
            // a_found_in_b will contain a mask indicate for each entry in A
            // whether it is seen in B
            let a_found_in_b: u8 = matrix_cmp_u16(v_a, v_b).to_bitmask() as u8;
            runningmask_a_found_in_b |= a_found_in_b;
            // we always compare the last values of A and B
            let a_max: u16 = lhs[i + LANES - 1];
            let b_max: u16 = rhs[j + LANES - 1];
            if a_max <= b_max {
                // Ok. In this code path, we are ready to write our v_a
                // because there is no need to read more from B, they will
                // all be large values.
                let bitmask_belongs_to_difference = runningmask_a_found_in_b ^ 0xFF;
                visitor.visit_vector(v_a, bitmask_belongs_to_difference);
                i += LANES;
                if i == st_a {
                    break;
                }
                runningmask_a_found_in_b = 0;
                v_a = load(simd, &lhs[i..]);
            }
            if b_max <= a_max {
                // in this code path, the current v_b has become useless
                j += LANES;
                if j == st_b {
                    break;
                }
                v_b = load(simd, &rhs[j..]);
            }
        }

        debug_assert!(i == st_a || j == st_b);

        // End of main vectorized loop
        // At this point either i_a == st_a, which is the end of the vectorized processing,
        // or i_b == st_b and we are not done processing the vector...
        // so we need to finish it off.
        if i < st_a {
            let remaining_rhs = &rhs[j..];
            if !remaining_rhs.is_empty() {
                let mut buffer: [u16; 8] = [0; 8]; // buffer to do a masked load
                buffer[..remaining_rhs.len()].copy_from_slice(remaining_rhs);
                // Ensure the buffer is filled with a value we should remove: we do not want to
                // end up trying to remove zero values which aren't actually in rhs
                buffer[remaining_rhs.len()..].fill(remaining_rhs[0]);
                v_b = load(simd, &buffer);
                let a_found_in_b: u8 = matrix_cmp_u16(v_a, v_b).to_bitmask() as u8;
                runningmask_a_found_in_b |= a_found_in_b;
                // Read from `lhs` (which `v_a` was loaded from): reading a lane of `v_a` makes LLVM
                // split `v_a` into pieces in the loop above
                let max_va = lhs[i + LANES - 1];
                let used_rhs = remaining_rhs.partition_point(|&b| b <= max_va);
                j += used_rhs;
            }
            let bitmask_belongs_to_difference: u8 = runningmask_a_found_in_b ^ 0xFF;
            visitor.visit_vector(v_a, bitmask_belongs_to_difference);
            i += LANES;
        }
    }

    // do the tail using scalar code
    scalar::sub(&lhs[i..], &rhs[j..], visitor);
}

/// load the first `LANES` values of `src`
///
/// ### Panics
///   - If `src` is shorter than `LANES`
#[inline(always)]
fn load<S: Simd>(simd: S, src: &[u16]) -> u16x8<S> {
    u16x8::from_slice(simd, &src[..LANES])
}

/// write `v` to the first `LANES` values of `out`
///
/// ### Panics
///   - If `out` is shorter than `LANES`
#[inline(always)]
fn store<S: Simd>(v: u16x8<S>, out: &mut [u16]) {
    v.store_slice(&mut out[..LANES])
}

/// Compare all lanes in `a` to all lanes in `b`
///
/// Returns result mask will be set if any lane at `a[i]` is in any lane of `b`
///
/// ### Example
/// ```ignore
/// let a = u16x8::from_slice(simd, &[1, 2, 3, 4, 32, 33, 34, 35]);
/// let b = u16x8::from_slice(simd, &[2, 4, 6, 8, 10, 12, 14, 16]);
/// let result = matrix_cmp_u16(a, b);
/// assert_eq!(result.to_bitmask(), 0b0000_1010);
/// ```
#[inline(always)]
fn matrix_cmp_u16<S: Simd>(a: u16x8<S>, b: u16x8<S>) -> mask16x8<S> {
    a.simd_eq(b)
        | a.simd_eq(b.rotate_elements_left::<1>())
        | a.simd_eq(b.rotate_elements_left::<2>())
        | a.simd_eq(b.rotate_elements_left::<3>())
        | a.simd_eq(b.rotate_elements_left::<4>())
        | a.simd_eq(b.rotate_elements_left::<5>())
        | a.simd_eq(b.rotate_elements_left::<6>())
        | a.simd_eq(b.rotate_elements_left::<7>())
}

/// Assuming that a and b are sorted, returns an array of the sorted output.
/// Developed originally for merge sort using SIMD instructions.
/// Standard merge. See, e.g., Inoue and Taura, SIMD- and Cache-Friendly
/// Algorithm for Sorting an Array of Structures
#[inline(always)]
fn simd_merge_u16<S: Simd>(a: u16x8<S>, b: u16x8<S>) -> [u16x8<S>; 2] {
    let mut tmp: u16x8<S> = a.min(b);
    let mut max: u16x8<S> = a.max(b);
    tmp = tmp.rotate_elements_left::<1>();
    let mut min: u16x8<S> = tmp.min(max);
    for _ in 0..6 {
        max = tmp.max(max);
        tmp = min.rotate_elements_left::<1>();
        min = tmp.min(max);
    }
    max = tmp.max(max);
    min = min.rotate_elements_left::<1>();
    [min, max]
}

/// Move the values in `val` with the corresponding index in `bitmask`
/// set to the front of the return vector, preserving their order.
///
/// The values in the return vector after index bitmask.count_ones() is unspecified.
// Dynamic swizzles operate on bytes, so the swizzle table moves the `u16` lanes two bytes at a time.
//
// e.g. if `bitmask` is `0b0101`, then swizzle the first two bytes (the first u16 lane) to the
// first two positions, and the 5th and 6th bytes (the third u16 lane) to the next two positions.
#[inline(always)]
pub fn swizzle_to_front<S: Simd>(val: u16x8<S>, bitmask: u8) -> u16x8<S> {
    static SWIZZLE_TABLE: [[u8; 16]; 256] = {
        let mut table = [[0; 16]; 256];
        let mut n = 0usize;
        while n < table.len() {
            let mut x = n;
            let mut i = 0;
            while x > 0 {
                let lsb = x.trailing_zeros() as u8;
                x ^= 1 << lsb;
                table[n][i] = lsb * 2; // first byte
                table[n][i + 1] = lsb * 2 + 1; // second byte
                i += 2;
            }
            n += 1;
        }
        table
    };

    // Our swizzle table retains the order of the bytes in the 16 bit lanes,
    // so it works with either native byte order.
    val.swizzle_dyn(SWIZZLE_TABLE[bitmask as usize])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bitmap::store::array_store::visitor::VecWriter;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    /// Checks the vectorized op produces the same result as the scalar op
    fn check_op(
        vector: impl Fn(&[u16], &[u16], &mut VecWriter),
        scalar: impl Fn(&[u16], &[u16], &mut VecWriter),
        lhs: &[u16],
        rhs: &[u16],
    ) {
        let mut expected = VecWriter::new(lhs.len() + rhs.len());
        scalar(lhs, rhs, &mut expected);
        let mut actual = VecWriter::new(lhs.len() + rhs.len());
        vector(lhs, rhs, &mut actual);
        assert_eq!(actual.into_inner(), expected.into_inner());
    }

    /// Checks all vectorized ops produce the same results as the scalar ops
    fn check_all(lhs: &[u16], rhs: &[u16]) {
        check_op(or, scalar::or, lhs, rhs);
        check_op(and, scalar::and, lhs, rhs);
        check_op(xor, scalar::xor, lhs, rhs);
        check_op(sub, scalar::sub, lhs, rhs);
    }

    proptest! {
        #[test]
        fn vector_ops_match_scalar_dense(
            lhs in prop::collection::btree_set(0u16..256, 0..200),
            rhs in prop::collection::btree_set(0u16..256, 0..200),
        ) {
            check_all(&Vec::from_iter(lhs), &Vec::from_iter(rhs));
        }

        #[test]
        fn vector_ops_match_scalar_sparse(
            lhs in prop::collection::btree_set(any::<u16>(), 0..200),
            rhs in prop::collection::btree_set(any::<u16>(), 0..200),
        ) {
            check_all(&Vec::from_iter(lhs), &Vec::from_iter(rhs));
        }
    }

    #[test]
    fn swizzle_to_front_keeps_masked_lanes_in_order() {
        dispatch!(level(), simd => {
            let values: [u16; 8] = [10, 11, 12, 13, 14, 15, 16, 17];
            let v = u16x8::from_slice(simd, &values);
            for mask in 0..=255u8 {
                let expected: Vec<u16> =
                    (0..8).filter(|i| mask & (1 << i) != 0).map(|i| values[i]).collect();
                let result = swizzle_to_front(v, mask);
                assert_eq!(&result.as_slice()[..expected.len()], &expected[..]);
            }
        });
    }

    #[test]
    fn vector_ops_match_scalar_at_lane_boundaries() {
        // Lengths around multiples of `LANES` exercise the scalar fallback, the vector loops,
        // and the partial vector tails; the offsets give disjoint, interleaved, and equal inputs.
        let lens = [0, 1, 7, 8, 9, 15, 16, 17, 24, 25];
        for len_l in lens {
            for len_r in lens {
                for offset in [0, 1, 4, 8, 64] {
                    let lhs: Vec<u16> = (0..len_l).map(|x| 2 * x).collect();
                    let rhs: Vec<u16> = (0..len_r).map(|x| 2 * x + offset).collect();
                    check_all(&lhs, &rhs);
                }
            }
        }
    }

    #[test]
    fn matrix_cmp_finds_lanes_of_a_in_b() {
        dispatch!(level(), simd => {
            let a = u16x8::from_slice(simd, &[1, 2, 3, 4, 32, 33, 34, 35]);
            let b = u16x8::from_slice(simd, &[2, 4, 6, 8, 10, 12, 14, 35]);
            assert_eq!(matrix_cmp_u16(a, b).to_bitmask(), 0b1000_1010);
        });
    }
}
