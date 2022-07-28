//! Ported from CRoaring and arXiv:1709.07821
//! Lemire et al, Roaring Bitmaps: Implementation of an Optimized Software Library
//!
//! Prior work: Schlegel et al., Fast Sorted-Set Intersection using SIMD Instructions
//!
//! Rust port notes:
//! The x86 PCMPESTRM instruction has been replaced with a simple vector or-shift
//! While several more instructions, this is what is available through LLVM intrinsics
//! and is portable.

#![cfg(feature = "simd")]

use super::scalar;
use core::simd::{
    mask16x8, simd_swizzle, u16x8, LaneCount, Mask, Simd, SimdElement, SimdPartialEq,
    SimdPartialOrd, SupportedLaneCount, ToBitMask,
};

// a one-pass SSE union algorithm
pub fn or(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
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

    #[inline]
    fn handle_vector(old: u16x8, new: u16x8, f: impl FnOnce(u16x8, u8)) {
        let tmp: u16x8 = Shr1::swizzle2(new, old);
        let mask = 255 - tmp.simd_eq(new).to_bitmask();
        f(new, mask);
    }

    if (lhs.len() < 8) || (rhs.len() < 8) {
        scalar::or(lhs, rhs, visitor);
        return;
    }

    let len1: usize = lhs.len() / 8;
    let len2: usize = rhs.len() / 8;

    let v_a: u16x8 = load(lhs);
    let v_b: u16x8 = load(rhs);
    let [mut v_min, mut v_max] = simd_merge_u16(v_a, v_b);

    let mut i = 1;
    let mut j = 1;
    handle_vector(Simd::splat(u16::MAX), v_min, |v, m| visitor.visit_vector(v, m));
    let mut v_prev: u16x8 = v_min;
    if (i < len1) && (j < len2) {
        let mut v: u16x8;
        let mut cur_a: u16 = lhs[8 * i];
        let mut cur_b: u16 = rhs[8 * j];
        loop {
            if cur_a <= cur_b {
                v = load(&lhs[8 * i..]);
                i += 1;
                if i < len1 {
                    cur_a = lhs[8 * i];
                } else {
                    break;
                }
            } else {
                v = load(&rhs[8 * j..]);
                j += 1;
                if j < len2 {
                    cur_b = rhs[8 * j];
                } else {
                    break;
                }
            }
            [v_min, v_max] = simd_merge_u16(v, v_max);
            handle_vector(v_prev, v_min, |v, m| visitor.visit_vector(v, m));
            v_prev = v_min;
        }
        [v_min, v_max] = simd_merge_u16(v, v_max);
        handle_vector(v_prev, v_min, |v, m| visitor.visit_vector(v, m));
        v_prev = v_min;
    }

    debug_assert!(i == len1 || j == len2);

    // we finish the rest off using a scalar algorithm
    // could be improved?
    //
    // copy the small end on a tmp buffer
    let mut buffer: [u16; 16] = [0; 16];
    let mut rem = 0;
    handle_vector(v_prev, v_max, |v, m| {
        store(swizzle_to_front(v, m), buffer.as_mut_slice());
        rem = m.count_ones() as usize;
    });

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
        buffer[..rem as usize].sort_unstable();
        rem = dedup(&mut buffer[..rem]);
        scalar::or(&buffer[..rem], tail_b, visitor);
    }
}

pub fn and(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    let st_a = (lhs.len() / u16x8::LANES) * u16x8::LANES;
    let st_b = (rhs.len() / u16x8::LANES) * u16x8::LANES;

    let mut i: usize = 0;
    let mut j: usize = 0;
    if (i < st_a) && (j < st_b) {
        let mut v_a: u16x8 = load(&lhs[i..]);
        let mut v_b: u16x8 = load(&rhs[j..]);
        loop {
            let mask = matrix_cmp_u16(v_a, v_b).to_bitmask();
            visitor.visit_vector(v_a, mask);

            let a_max: u16 = lhs[i + u16x8::LANES - 1];
            let b_max: u16 = rhs[j + u16x8::LANES - 1];
            if a_max <= b_max {
                i += u16x8::LANES;
                if i == st_a {
                    break;
                }
                v_a = load(&lhs[i..]);
            }
            if b_max <= a_max {
                j += u16x8::LANES;
                if j == st_b {
                    break;
                }
                v_b = load(&rhs[j..]);
            }
        }
    }

    // intersect the tail using scalar intersection
    scalar::and(&lhs[i..], &rhs[j..], visitor);
}

// a one-pass SSE xor algorithm
pub fn xor(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
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

    // write vector new, while omitting repeated values assuming that previously
    // written vector was "old"
    #[inline]
    fn handle_vector(old: u16x8, new: u16x8, f: impl FnOnce(u16x8, u8)) {
        let tmp1: u16x8 = Shr2::swizzle2(new, old);
        let tmp2: u16x8 = Shr1::swizzle2(new, old);
        let eq_l: mask16x8 = tmp2.simd_eq(tmp1);
        let eq_r: mask16x8 = tmp2.simd_eq(new);
        let eq_l_or_r: mask16x8 = eq_l | eq_r;
        let mask: u8 = eq_l_or_r.to_bitmask();
        f(tmp2, 255 - mask);
    }

    if (lhs.len() < 8) || (rhs.len() < 8) {
        scalar::xor(lhs, rhs, visitor);
        return;
    }

    let len1: usize = lhs.len() / 8;
    let len2: usize = rhs.len() / 8;

    let v_a: u16x8 = load(lhs);
    let v_b: u16x8 = load(rhs);
    let [mut v_min, mut v_max] = simd_merge_u16(v_a, v_b);

    let mut i = 1;
    let mut j = 1;
    handle_vector(Simd::splat(u16::MAX), v_min, |v, m| visitor.visit_vector(v, m));
    let mut v_prev: u16x8 = v_min;
    if (i < len1) && (j < len2) {
        let mut v: u16x8;
        let mut cur_a: u16 = lhs[8 * i];
        let mut cur_b: u16 = rhs[8 * j];
        loop {
            if cur_a <= cur_b {
                v = load(&lhs[8 * i..]);
                i += 1;
                if i < len1 {
                    cur_a = lhs[8 * i];
                } else {
                    break;
                }
            } else {
                v = load(&rhs[8 * j..]);
                j += 1;
                if j < len2 {
                    cur_b = rhs[8 * j];
                } else {
                    break;
                }
            }
            [v_min, v_max] = simd_merge_u16(v, v_max);
            handle_vector(v_prev, v_min, |v, m| visitor.visit_vector(v, m));
            v_prev = v_min;
        }
        [v_min, v_max] = simd_merge_u16(v, v_max);
        handle_vector(v_prev, v_min, |v, m| visitor.visit_vector(v, m));
        v_prev = v_min;
    }

    debug_assert!(i == len1 || j == len2);

    // we finish the rest off using a scalar algorithm
    // could be improved?
    // conditionally stores the last value of laststore as well as all but the
    // last value of vecMax,
    let mut buffer: [u16; 17] = [0; 17];
    // remaining size
    let mut rem = 0;
    handle_vector(v_prev, v_max, |v, m| {
        store(swizzle_to_front(v, m), buffer.as_mut_slice());
        rem = m.count_ones() as usize;
    });

    let arr_max = v_max.as_array();
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
        buffer[..rem as usize].sort_unstable();
        rem = xor_slice(&mut buffer[..rem]);
        scalar::xor(&buffer[..rem], tail_b, visitor);
    }
}

pub fn sub(lhs: &[u16], rhs: &[u16], visitor: &mut impl BinaryOperationVisitor) {
    // we handle the degenerate cases
    if lhs.is_empty() {
        return;
    } else if rhs.is_empty() {
        visitor.visit_slice(lhs);
        return;
    }

    let st_a = (lhs.len() / u16x8::LANES) * u16x8::LANES;
    let st_b = (rhs.len() / u16x8::LANES) * u16x8::LANES;

    let mut i = 0;
    let mut j = 0;
    if (i < st_a) && (j < st_b) {
        let mut v_a: u16x8 = load(&lhs[i..]);
        let mut v_b: u16x8 = load(&rhs[j..]);
        // we have a running mask which indicates which values from a have been
        // spotted in b, these don't get written out.
        let mut runningmask_a_found_in_b: u8 = 0;
        loop {
            // a_found_in_b will contain a mask indicate for each entry in A
            // whether it is seen in B
            let a_found_in_b: u8 = matrix_cmp_u16(v_a, v_b).to_bitmask();
            runningmask_a_found_in_b |= a_found_in_b;
            // we always compare the last values of A and B
            let a_max: u16 = lhs[i + u16x8::LANES - 1];
            let b_max: u16 = rhs[j + u16x8::LANES - 1];
            if a_max <= b_max {
                // Ok. In this code path, we are ready to write our v_a
                // because there is no need to read more from B, they will
                // all be large values.
                let bitmask_belongs_to_difference = runningmask_a_found_in_b ^ 0xFF;
                visitor.visit_vector(v_a, bitmask_belongs_to_difference);
                i += u16x8::LANES;
                if i == st_a {
                    break;
                }
                runningmask_a_found_in_b = 0;
                v_a = load(&lhs[i..]);
            }
            if b_max <= a_max {
                // in this code path, the current v_b has become useless
                j += u16x8::LANES;
                if j == st_b {
                    break;
                }
                v_b = load(&rhs[j..]);
            }
        }

        debug_assert!(i == st_a || j == st_b);

        // End of main vectorized loop
        // At this point either i_a == st_a, which is the end of the vectorized processing,
        // or i_b == st_b and we are not done processing the vector...
        // so we need to finish it off.
        if i < st_a {
            let mut buffer: [u16; 8] = [0; 8]; // buffer to do a masked load
            buffer[..rhs.len() - j].copy_from_slice(&rhs[j..]);
            v_b = Simd::from_array(buffer);
            let a_found_in_b: u8 = matrix_cmp_u16(v_a, v_b).to_bitmask();
            runningmask_a_found_in_b |= a_found_in_b;
            let bitmask_belongs_to_difference: u8 = runningmask_a_found_in_b ^ 0xFF;
            visitor.visit_vector(v_a, bitmask_belongs_to_difference);
            i += u16x8::LANES;
        }
    }

    // do the tail using scalar code
    scalar::sub(&lhs[i..], &rhs[j..], visitor);
}

/// compute the min for each lane in `a` and `b`
#[inline]
fn lanes_min_u16<const LANES: usize>(
    lhs: Simd<u16, LANES>,
    rhs: Simd<u16, LANES>,
) -> Simd<u16, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    lhs.simd_le(rhs).select(lhs, rhs)
}

/// compute the max for each lane in `a` and `b`
#[inline]
fn lanes_max_u16<const LANES: usize>(
    lhs: Simd<u16, LANES>,
    rhs: Simd<u16, LANES>,
) -> Simd<u16, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    lhs.simd_gt(rhs).select(lhs, rhs)
}

#[inline]
pub fn load<U, const LANES: usize>(src: &[U]) -> Simd<U, LANES>
where
    U: SimdElement + PartialOrd,
    LaneCount<LANES>: SupportedLaneCount,
{
    debug_assert!(src.len() >= LANES);
    unsafe { load_unchecked(src) }
}

/// write `v` to slice `out` without checking bounds
///
/// ### Safety
///   - The caller must ensure `LANES` does not exceed the allocation for `out`
#[inline]
pub unsafe fn load_unchecked<U, const LANES: usize>(src: &[U]) -> Simd<U, LANES>
where
    U: SimdElement + PartialOrd,
    LaneCount<LANES>: SupportedLaneCount,
{
    unsafe { std::ptr::read_unaligned(src as *const _ as *const Simd<U, LANES>) }
}

/// write `v` to slice `out`
#[inline]
pub fn store<U, const LANES: usize>(v: Simd<U, LANES>, out: &mut [U])
where
    U: SimdElement + PartialOrd,
    LaneCount<LANES>: SupportedLaneCount,
{
    debug_assert!(out.len() >= LANES);
    unsafe {
        store_unchecked(v, out);
    }
}

/// write `v` to slice `out` without checking bounds
///
/// ### Safety
///   - The caller must ensure `LANES` does not exceed the allocation for `out`
#[inline]
unsafe fn store_unchecked<U, const LANES: usize>(v: Simd<U, LANES>, out: &mut [U])
where
    U: SimdElement + PartialOrd,
    LaneCount<LANES>: SupportedLaneCount,
{
    unsafe { std::ptr::write_unaligned(out as *mut _ as *mut Simd<U, LANES>, v) }
}

/// Compare all lanes in `a` to all lanes in `b`
///
/// Returns result mask will be set if any lane at `a[i]` is in any lane of `b`
///
/// ### Example
/// ```ignore
/// let a = Simd::from_array([1, 2, 3, 4, 32, 33, 34, 35]);
/// let b = Simd::from_array([2, 4, 6, 8, 10, 12, 14, 16]);
/// let result = matrix_cmp_u16(a, b);
/// assert_eq!(result, Mask::from_array([false, true, false, true, false, false, false, false]));
/// ```
#[inline]
// It would be nice to implement this for all supported lane counts
// However, we currently only support u16x8 so it's not really necessary
fn matrix_cmp_u16(a: Simd<u16, 8>, b: Simd<u16, 8>) -> Mask<i16, 8> {
    a.simd_eq(b)
        | a.simd_eq(b.rotate_lanes_left::<1>())
        | a.simd_eq(b.rotate_lanes_left::<2>())
        | a.simd_eq(b.rotate_lanes_left::<3>())
        | a.simd_eq(b.rotate_lanes_left::<4>())
        | a.simd_eq(b.rotate_lanes_left::<5>())
        | a.simd_eq(b.rotate_lanes_left::<6>())
        | a.simd_eq(b.rotate_lanes_left::<7>())
}

use crate::bitmap::store::array_store::visitor::BinaryOperationVisitor;
use core::simd::{Swizzle2, Which, Which::First as A, Which::Second as B};

/// Append to vectors to an imaginary 16 lane vector,  shift the lanes right by 1, then
/// truncate to the low order 8 lanes
pub struct Shr1;
impl Swizzle2<8, 8> for Shr1 {
    const INDEX: [Which; 8] = [B(7), A(0), A(1), A(2), A(3), A(4), A(5), A(6)];
}

/// Append to vectors to an imaginary 16 lane vector,  shift the lanes right by 2, then
/// truncate to the low order 8 lanes
pub struct Shr2;
impl Swizzle2<8, 8> for Shr2 {
    const INDEX: [Which; 8] = [B(6), B(7), A(0), A(1), A(2), A(3), A(4), A(5)];
}

/// Assuming that a and b are sorted, returns an array of the sorted output.
/// Developed originally for merge sort using SIMD instructions.
/// Standard merge. See, e.g., Inoue and Taura, SIMD- and Cache-Friendly
/// Algorithm for Sorting an Array of Structures
#[inline]
fn simd_merge_u16(a: Simd<u16, 8>, b: Simd<u16, 8>) -> [Simd<u16, 8>; 2] {
    let mut tmp: Simd<u16, 8> = lanes_min_u16(a, b);
    let mut max: Simd<u16, 8> = lanes_max_u16(a, b);
    tmp = tmp.rotate_lanes_left::<1>();
    let mut min: Simd<u16, 8> = lanes_min_u16(tmp, max);
    for _ in 0..6 {
        max = lanes_max_u16(tmp, max);
        tmp = min.rotate_lanes_left::<1>();
        min = lanes_min_u16(tmp, max);
    }
    max = lanes_max_u16(tmp, max);
    min = min.rotate_lanes_left::<1>();
    [min, max]
}

/// Move the values in `val` with the corresponding index in `bitmask`
/// set to the front of the return vector, preserving their order.
///
/// This had to be implemented as a jump table to be portable,
/// as LLVM swizzle intrinsic only supports swizzle by a const
/// value. https://github.com/rust-lang/portable-simd/issues/11
///
/// The values in the return vector after index bitmask.count_ones() is unspecified.
///
/// The masks can be constructed with the following snippet
/// ```ignore
/// for n in 0usize..256 {
///      let mut x = n;
///      let mut arr = [0; 8];
///      let mut i = 0;
///      while x > 0 {
///          let lsb = x.trailing_zeros();
///          arr[i] = lsb;
///          x ^= 1 << lsb;
///          i += 1;
///      }
/// }
/// ```
pub fn swizzle_to_front(val: u16x8, bitmask: u8) -> u16x8 {
    match bitmask {
        0x00 => simd_swizzle!(val, [0, 0, 0, 0, 0, 0, 0, 0]),
        0x01 => simd_swizzle!(val, [0, 0, 0, 0, 0, 0, 0, 0]),
        0x02 => simd_swizzle!(val, [1, 0, 0, 0, 0, 0, 0, 0]),
        0x03 => simd_swizzle!(val, [0, 1, 0, 0, 0, 0, 0, 0]),
        0x04 => simd_swizzle!(val, [2, 0, 0, 0, 0, 0, 0, 0]),
        0x05 => simd_swizzle!(val, [0, 2, 0, 0, 0, 0, 0, 0]),
        0x06 => simd_swizzle!(val, [1, 2, 0, 0, 0, 0, 0, 0]),
        0x07 => simd_swizzle!(val, [0, 1, 2, 0, 0, 0, 0, 0]),
        0x08 => simd_swizzle!(val, [3, 0, 0, 0, 0, 0, 0, 0]),
        0x09 => simd_swizzle!(val, [0, 3, 0, 0, 0, 0, 0, 0]),
        0x0A => simd_swizzle!(val, [1, 3, 0, 0, 0, 0, 0, 0]),
        0x0B => simd_swizzle!(val, [0, 1, 3, 0, 0, 0, 0, 0]),
        0x0C => simd_swizzle!(val, [2, 3, 0, 0, 0, 0, 0, 0]),
        0x0D => simd_swizzle!(val, [0, 2, 3, 0, 0, 0, 0, 0]),
        0x0E => simd_swizzle!(val, [1, 2, 3, 0, 0, 0, 0, 0]),
        0x0F => simd_swizzle!(val, [0, 1, 2, 3, 0, 0, 0, 0]),
        0x10 => simd_swizzle!(val, [4, 0, 0, 0, 0, 0, 0, 0]),
        0x11 => simd_swizzle!(val, [0, 4, 0, 0, 0, 0, 0, 0]),
        0x12 => simd_swizzle!(val, [1, 4, 0, 0, 0, 0, 0, 0]),
        0x13 => simd_swizzle!(val, [0, 1, 4, 0, 0, 0, 0, 0]),
        0x14 => simd_swizzle!(val, [2, 4, 0, 0, 0, 0, 0, 0]),
        0x15 => simd_swizzle!(val, [0, 2, 4, 0, 0, 0, 0, 0]),
        0x16 => simd_swizzle!(val, [1, 2, 4, 0, 0, 0, 0, 0]),
        0x17 => simd_swizzle!(val, [0, 1, 2, 4, 0, 0, 0, 0]),
        0x18 => simd_swizzle!(val, [3, 4, 0, 0, 0, 0, 0, 0]),
        0x19 => simd_swizzle!(val, [0, 3, 4, 0, 0, 0, 0, 0]),
        0x1A => simd_swizzle!(val, [1, 3, 4, 0, 0, 0, 0, 0]),
        0x1B => simd_swizzle!(val, [0, 1, 3, 4, 0, 0, 0, 0]),
        0x1C => simd_swizzle!(val, [2, 3, 4, 0, 0, 0, 0, 0]),
        0x1D => simd_swizzle!(val, [0, 2, 3, 4, 0, 0, 0, 0]),
        0x1E => simd_swizzle!(val, [1, 2, 3, 4, 0, 0, 0, 0]),
        0x1F => simd_swizzle!(val, [0, 1, 2, 3, 4, 0, 0, 0]),
        0x20 => simd_swizzle!(val, [5, 0, 0, 0, 0, 0, 0, 0]),
        0x21 => simd_swizzle!(val, [0, 5, 0, 0, 0, 0, 0, 0]),
        0x22 => simd_swizzle!(val, [1, 5, 0, 0, 0, 0, 0, 0]),
        0x23 => simd_swizzle!(val, [0, 1, 5, 0, 0, 0, 0, 0]),
        0x24 => simd_swizzle!(val, [2, 5, 0, 0, 0, 0, 0, 0]),
        0x25 => simd_swizzle!(val, [0, 2, 5, 0, 0, 0, 0, 0]),
        0x26 => simd_swizzle!(val, [1, 2, 5, 0, 0, 0, 0, 0]),
        0x27 => simd_swizzle!(val, [0, 1, 2, 5, 0, 0, 0, 0]),
        0x28 => simd_swizzle!(val, [3, 5, 0, 0, 0, 0, 0, 0]),
        0x29 => simd_swizzle!(val, [0, 3, 5, 0, 0, 0, 0, 0]),
        0x2A => simd_swizzle!(val, [1, 3, 5, 0, 0, 0, 0, 0]),
        0x2B => simd_swizzle!(val, [0, 1, 3, 5, 0, 0, 0, 0]),
        0x2C => simd_swizzle!(val, [2, 3, 5, 0, 0, 0, 0, 0]),
        0x2D => simd_swizzle!(val, [0, 2, 3, 5, 0, 0, 0, 0]),
        0x2E => simd_swizzle!(val, [1, 2, 3, 5, 0, 0, 0, 0]),
        0x2F => simd_swizzle!(val, [0, 1, 2, 3, 5, 0, 0, 0]),
        0x30 => simd_swizzle!(val, [4, 5, 0, 0, 0, 0, 0, 0]),
        0x31 => simd_swizzle!(val, [0, 4, 5, 0, 0, 0, 0, 0]),
        0x32 => simd_swizzle!(val, [1, 4, 5, 0, 0, 0, 0, 0]),
        0x33 => simd_swizzle!(val, [0, 1, 4, 5, 0, 0, 0, 0]),
        0x34 => simd_swizzle!(val, [2, 4, 5, 0, 0, 0, 0, 0]),
        0x35 => simd_swizzle!(val, [0, 2, 4, 5, 0, 0, 0, 0]),
        0x36 => simd_swizzle!(val, [1, 2, 4, 5, 0, 0, 0, 0]),
        0x37 => simd_swizzle!(val, [0, 1, 2, 4, 5, 0, 0, 0]),
        0x38 => simd_swizzle!(val, [3, 4, 5, 0, 0, 0, 0, 0]),
        0x39 => simd_swizzle!(val, [0, 3, 4, 5, 0, 0, 0, 0]),
        0x3A => simd_swizzle!(val, [1, 3, 4, 5, 0, 0, 0, 0]),
        0x3B => simd_swizzle!(val, [0, 1, 3, 4, 5, 0, 0, 0]),
        0x3C => simd_swizzle!(val, [2, 3, 4, 5, 0, 0, 0, 0]),
        0x3D => simd_swizzle!(val, [0, 2, 3, 4, 5, 0, 0, 0]),
        0x3E => simd_swizzle!(val, [1, 2, 3, 4, 5, 0, 0, 0]),
        0x3F => simd_swizzle!(val, [0, 1, 2, 3, 4, 5, 0, 0]),
        0x40 => simd_swizzle!(val, [6, 0, 0, 0, 0, 0, 0, 0]),
        0x41 => simd_swizzle!(val, [0, 6, 0, 0, 0, 0, 0, 0]),
        0x42 => simd_swizzle!(val, [1, 6, 0, 0, 0, 0, 0, 0]),
        0x43 => simd_swizzle!(val, [0, 1, 6, 0, 0, 0, 0, 0]),
        0x44 => simd_swizzle!(val, [2, 6, 0, 0, 0, 0, 0, 0]),
        0x45 => simd_swizzle!(val, [0, 2, 6, 0, 0, 0, 0, 0]),
        0x46 => simd_swizzle!(val, [1, 2, 6, 0, 0, 0, 0, 0]),
        0x47 => simd_swizzle!(val, [0, 1, 2, 6, 0, 0, 0, 0]),
        0x48 => simd_swizzle!(val, [3, 6, 0, 0, 0, 0, 0, 0]),
        0x49 => simd_swizzle!(val, [0, 3, 6, 0, 0, 0, 0, 0]),
        0x4A => simd_swizzle!(val, [1, 3, 6, 0, 0, 0, 0, 0]),
        0x4B => simd_swizzle!(val, [0, 1, 3, 6, 0, 0, 0, 0]),
        0x4C => simd_swizzle!(val, [2, 3, 6, 0, 0, 0, 0, 0]),
        0x4D => simd_swizzle!(val, [0, 2, 3, 6, 0, 0, 0, 0]),
        0x4E => simd_swizzle!(val, [1, 2, 3, 6, 0, 0, 0, 0]),
        0x4F => simd_swizzle!(val, [0, 1, 2, 3, 6, 0, 0, 0]),
        0x50 => simd_swizzle!(val, [4, 6, 0, 0, 0, 0, 0, 0]),
        0x51 => simd_swizzle!(val, [0, 4, 6, 0, 0, 0, 0, 0]),
        0x52 => simd_swizzle!(val, [1, 4, 6, 0, 0, 0, 0, 0]),
        0x53 => simd_swizzle!(val, [0, 1, 4, 6, 0, 0, 0, 0]),
        0x54 => simd_swizzle!(val, [2, 4, 6, 0, 0, 0, 0, 0]),
        0x55 => simd_swizzle!(val, [0, 2, 4, 6, 0, 0, 0, 0]),
        0x56 => simd_swizzle!(val, [1, 2, 4, 6, 0, 0, 0, 0]),
        0x57 => simd_swizzle!(val, [0, 1, 2, 4, 6, 0, 0, 0]),
        0x58 => simd_swizzle!(val, [3, 4, 6, 0, 0, 0, 0, 0]),
        0x59 => simd_swizzle!(val, [0, 3, 4, 6, 0, 0, 0, 0]),
        0x5A => simd_swizzle!(val, [1, 3, 4, 6, 0, 0, 0, 0]),
        0x5B => simd_swizzle!(val, [0, 1, 3, 4, 6, 0, 0, 0]),
        0x5C => simd_swizzle!(val, [2, 3, 4, 6, 0, 0, 0, 0]),
        0x5D => simd_swizzle!(val, [0, 2, 3, 4, 6, 0, 0, 0]),
        0x5E => simd_swizzle!(val, [1, 2, 3, 4, 6, 0, 0, 0]),
        0x5F => simd_swizzle!(val, [0, 1, 2, 3, 4, 6, 0, 0]),
        0x60 => simd_swizzle!(val, [5, 6, 0, 0, 0, 0, 0, 0]),
        0x61 => simd_swizzle!(val, [0, 5, 6, 0, 0, 0, 0, 0]),
        0x62 => simd_swizzle!(val, [1, 5, 6, 0, 0, 0, 0, 0]),
        0x63 => simd_swizzle!(val, [0, 1, 5, 6, 0, 0, 0, 0]),
        0x64 => simd_swizzle!(val, [2, 5, 6, 0, 0, 0, 0, 0]),
        0x65 => simd_swizzle!(val, [0, 2, 5, 6, 0, 0, 0, 0]),
        0x66 => simd_swizzle!(val, [1, 2, 5, 6, 0, 0, 0, 0]),
        0x67 => simd_swizzle!(val, [0, 1, 2, 5, 6, 0, 0, 0]),
        0x68 => simd_swizzle!(val, [3, 5, 6, 0, 0, 0, 0, 0]),
        0x69 => simd_swizzle!(val, [0, 3, 5, 6, 0, 0, 0, 0]),
        0x6A => simd_swizzle!(val, [1, 3, 5, 6, 0, 0, 0, 0]),
        0x6B => simd_swizzle!(val, [0, 1, 3, 5, 6, 0, 0, 0]),
        0x6C => simd_swizzle!(val, [2, 3, 5, 6, 0, 0, 0, 0]),
        0x6D => simd_swizzle!(val, [0, 2, 3, 5, 6, 0, 0, 0]),
        0x6E => simd_swizzle!(val, [1, 2, 3, 5, 6, 0, 0, 0]),
        0x6F => simd_swizzle!(val, [0, 1, 2, 3, 5, 6, 0, 0]),
        0x70 => simd_swizzle!(val, [4, 5, 6, 0, 0, 0, 0, 0]),
        0x71 => simd_swizzle!(val, [0, 4, 5, 6, 0, 0, 0, 0]),
        0x72 => simd_swizzle!(val, [1, 4, 5, 6, 0, 0, 0, 0]),
        0x73 => simd_swizzle!(val, [0, 1, 4, 5, 6, 0, 0, 0]),
        0x74 => simd_swizzle!(val, [2, 4, 5, 6, 0, 0, 0, 0]),
        0x75 => simd_swizzle!(val, [0, 2, 4, 5, 6, 0, 0, 0]),
        0x76 => simd_swizzle!(val, [1, 2, 4, 5, 6, 0, 0, 0]),
        0x77 => simd_swizzle!(val, [0, 1, 2, 4, 5, 6, 0, 0]),
        0x78 => simd_swizzle!(val, [3, 4, 5, 6, 0, 0, 0, 0]),
        0x79 => simd_swizzle!(val, [0, 3, 4, 5, 6, 0, 0, 0]),
        0x7A => simd_swizzle!(val, [1, 3, 4, 5, 6, 0, 0, 0]),
        0x7B => simd_swizzle!(val, [0, 1, 3, 4, 5, 6, 0, 0]),
        0x7C => simd_swizzle!(val, [2, 3, 4, 5, 6, 0, 0, 0]),
        0x7D => simd_swizzle!(val, [0, 2, 3, 4, 5, 6, 0, 0]),
        0x7E => simd_swizzle!(val, [1, 2, 3, 4, 5, 6, 0, 0]),
        0x7F => simd_swizzle!(val, [0, 1, 2, 3, 4, 5, 6, 0]),
        0x80 => simd_swizzle!(val, [7, 0, 0, 0, 0, 0, 0, 0]),
        0x81 => simd_swizzle!(val, [0, 7, 0, 0, 0, 0, 0, 0]),
        0x82 => simd_swizzle!(val, [1, 7, 0, 0, 0, 0, 0, 0]),
        0x83 => simd_swizzle!(val, [0, 1, 7, 0, 0, 0, 0, 0]),
        0x84 => simd_swizzle!(val, [2, 7, 0, 0, 0, 0, 0, 0]),
        0x85 => simd_swizzle!(val, [0, 2, 7, 0, 0, 0, 0, 0]),
        0x86 => simd_swizzle!(val, [1, 2, 7, 0, 0, 0, 0, 0]),
        0x87 => simd_swizzle!(val, [0, 1, 2, 7, 0, 0, 0, 0]),
        0x88 => simd_swizzle!(val, [3, 7, 0, 0, 0, 0, 0, 0]),
        0x89 => simd_swizzle!(val, [0, 3, 7, 0, 0, 0, 0, 0]),
        0x8A => simd_swizzle!(val, [1, 3, 7, 0, 0, 0, 0, 0]),
        0x8B => simd_swizzle!(val, [0, 1, 3, 7, 0, 0, 0, 0]),
        0x8C => simd_swizzle!(val, [2, 3, 7, 0, 0, 0, 0, 0]),
        0x8D => simd_swizzle!(val, [0, 2, 3, 7, 0, 0, 0, 0]),
        0x8E => simd_swizzle!(val, [1, 2, 3, 7, 0, 0, 0, 0]),
        0x8F => simd_swizzle!(val, [0, 1, 2, 3, 7, 0, 0, 0]),
        0x90 => simd_swizzle!(val, [4, 7, 0, 0, 0, 0, 0, 0]),
        0x91 => simd_swizzle!(val, [0, 4, 7, 0, 0, 0, 0, 0]),
        0x92 => simd_swizzle!(val, [1, 4, 7, 0, 0, 0, 0, 0]),
        0x93 => simd_swizzle!(val, [0, 1, 4, 7, 0, 0, 0, 0]),
        0x94 => simd_swizzle!(val, [2, 4, 7, 0, 0, 0, 0, 0]),
        0x95 => simd_swizzle!(val, [0, 2, 4, 7, 0, 0, 0, 0]),
        0x96 => simd_swizzle!(val, [1, 2, 4, 7, 0, 0, 0, 0]),
        0x97 => simd_swizzle!(val, [0, 1, 2, 4, 7, 0, 0, 0]),
        0x98 => simd_swizzle!(val, [3, 4, 7, 0, 0, 0, 0, 0]),
        0x99 => simd_swizzle!(val, [0, 3, 4, 7, 0, 0, 0, 0]),
        0x9A => simd_swizzle!(val, [1, 3, 4, 7, 0, 0, 0, 0]),
        0x9B => simd_swizzle!(val, [0, 1, 3, 4, 7, 0, 0, 0]),
        0x9C => simd_swizzle!(val, [2, 3, 4, 7, 0, 0, 0, 0]),
        0x9D => simd_swizzle!(val, [0, 2, 3, 4, 7, 0, 0, 0]),
        0x9E => simd_swizzle!(val, [1, 2, 3, 4, 7, 0, 0, 0]),
        0x9F => simd_swizzle!(val, [0, 1, 2, 3, 4, 7, 0, 0]),
        0xA0 => simd_swizzle!(val, [5, 7, 0, 0, 0, 0, 0, 0]),
        0xA1 => simd_swizzle!(val, [0, 5, 7, 0, 0, 0, 0, 0]),
        0xA2 => simd_swizzle!(val, [1, 5, 7, 0, 0, 0, 0, 0]),
        0xA3 => simd_swizzle!(val, [0, 1, 5, 7, 0, 0, 0, 0]),
        0xA4 => simd_swizzle!(val, [2, 5, 7, 0, 0, 0, 0, 0]),
        0xA5 => simd_swizzle!(val, [0, 2, 5, 7, 0, 0, 0, 0]),
        0xA6 => simd_swizzle!(val, [1, 2, 5, 7, 0, 0, 0, 0]),
        0xA7 => simd_swizzle!(val, [0, 1, 2, 5, 7, 0, 0, 0]),
        0xA8 => simd_swizzle!(val, [3, 5, 7, 0, 0, 0, 0, 0]),
        0xA9 => simd_swizzle!(val, [0, 3, 5, 7, 0, 0, 0, 0]),
        0xAA => simd_swizzle!(val, [1, 3, 5, 7, 0, 0, 0, 0]),
        0xAB => simd_swizzle!(val, [0, 1, 3, 5, 7, 0, 0, 0]),
        0xAC => simd_swizzle!(val, [2, 3, 5, 7, 0, 0, 0, 0]),
        0xAD => simd_swizzle!(val, [0, 2, 3, 5, 7, 0, 0, 0]),
        0xAE => simd_swizzle!(val, [1, 2, 3, 5, 7, 0, 0, 0]),
        0xAF => simd_swizzle!(val, [0, 1, 2, 3, 5, 7, 0, 0]),
        0xB0 => simd_swizzle!(val, [4, 5, 7, 0, 0, 0, 0, 0]),
        0xB1 => simd_swizzle!(val, [0, 4, 5, 7, 0, 0, 0, 0]),
        0xB2 => simd_swizzle!(val, [1, 4, 5, 7, 0, 0, 0, 0]),
        0xB3 => simd_swizzle!(val, [0, 1, 4, 5, 7, 0, 0, 0]),
        0xB4 => simd_swizzle!(val, [2, 4, 5, 7, 0, 0, 0, 0]),
        0xB5 => simd_swizzle!(val, [0, 2, 4, 5, 7, 0, 0, 0]),
        0xB6 => simd_swizzle!(val, [1, 2, 4, 5, 7, 0, 0, 0]),
        0xB7 => simd_swizzle!(val, [0, 1, 2, 4, 5, 7, 0, 0]),
        0xB8 => simd_swizzle!(val, [3, 4, 5, 7, 0, 0, 0, 0]),
        0xB9 => simd_swizzle!(val, [0, 3, 4, 5, 7, 0, 0, 0]),
        0xBA => simd_swizzle!(val, [1, 3, 4, 5, 7, 0, 0, 0]),
        0xBB => simd_swizzle!(val, [0, 1, 3, 4, 5, 7, 0, 0]),
        0xBC => simd_swizzle!(val, [2, 3, 4, 5, 7, 0, 0, 0]),
        0xBD => simd_swizzle!(val, [0, 2, 3, 4, 5, 7, 0, 0]),
        0xBE => simd_swizzle!(val, [1, 2, 3, 4, 5, 7, 0, 0]),
        0xBF => simd_swizzle!(val, [0, 1, 2, 3, 4, 5, 7, 0]),
        0xC0 => simd_swizzle!(val, [6, 7, 0, 0, 0, 0, 0, 0]),
        0xC1 => simd_swizzle!(val, [0, 6, 7, 0, 0, 0, 0, 0]),
        0xC2 => simd_swizzle!(val, [1, 6, 7, 0, 0, 0, 0, 0]),
        0xC3 => simd_swizzle!(val, [0, 1, 6, 7, 0, 0, 0, 0]),
        0xC4 => simd_swizzle!(val, [2, 6, 7, 0, 0, 0, 0, 0]),
        0xC5 => simd_swizzle!(val, [0, 2, 6, 7, 0, 0, 0, 0]),
        0xC6 => simd_swizzle!(val, [1, 2, 6, 7, 0, 0, 0, 0]),
        0xC7 => simd_swizzle!(val, [0, 1, 2, 6, 7, 0, 0, 0]),
        0xC8 => simd_swizzle!(val, [3, 6, 7, 0, 0, 0, 0, 0]),
        0xC9 => simd_swizzle!(val, [0, 3, 6, 7, 0, 0, 0, 0]),
        0xCA => simd_swizzle!(val, [1, 3, 6, 7, 0, 0, 0, 0]),
        0xCB => simd_swizzle!(val, [0, 1, 3, 6, 7, 0, 0, 0]),
        0xCC => simd_swizzle!(val, [2, 3, 6, 7, 0, 0, 0, 0]),
        0xCD => simd_swizzle!(val, [0, 2, 3, 6, 7, 0, 0, 0]),
        0xCE => simd_swizzle!(val, [1, 2, 3, 6, 7, 0, 0, 0]),
        0xCF => simd_swizzle!(val, [0, 1, 2, 3, 6, 7, 0, 0]),
        0xD0 => simd_swizzle!(val, [4, 6, 7, 0, 0, 0, 0, 0]),
        0xD1 => simd_swizzle!(val, [0, 4, 6, 7, 0, 0, 0, 0]),
        0xD2 => simd_swizzle!(val, [1, 4, 6, 7, 0, 0, 0, 0]),
        0xD3 => simd_swizzle!(val, [0, 1, 4, 6, 7, 0, 0, 0]),
        0xD4 => simd_swizzle!(val, [2, 4, 6, 7, 0, 0, 0, 0]),
        0xD5 => simd_swizzle!(val, [0, 2, 4, 6, 7, 0, 0, 0]),
        0xD6 => simd_swizzle!(val, [1, 2, 4, 6, 7, 0, 0, 0]),
        0xD7 => simd_swizzle!(val, [0, 1, 2, 4, 6, 7, 0, 0]),
        0xD8 => simd_swizzle!(val, [3, 4, 6, 7, 0, 0, 0, 0]),
        0xD9 => simd_swizzle!(val, [0, 3, 4, 6, 7, 0, 0, 0]),
        0xDA => simd_swizzle!(val, [1, 3, 4, 6, 7, 0, 0, 0]),
        0xDB => simd_swizzle!(val, [0, 1, 3, 4, 6, 7, 0, 0]),
        0xDC => simd_swizzle!(val, [2, 3, 4, 6, 7, 0, 0, 0]),
        0xDD => simd_swizzle!(val, [0, 2, 3, 4, 6, 7, 0, 0]),
        0xDE => simd_swizzle!(val, [1, 2, 3, 4, 6, 7, 0, 0]),
        0xDF => simd_swizzle!(val, [0, 1, 2, 3, 4, 6, 7, 0]),
        0xE0 => simd_swizzle!(val, [5, 6, 7, 0, 0, 0, 0, 0]),
        0xE1 => simd_swizzle!(val, [0, 5, 6, 7, 0, 0, 0, 0]),
        0xE2 => simd_swizzle!(val, [1, 5, 6, 7, 0, 0, 0, 0]),
        0xE3 => simd_swizzle!(val, [0, 1, 5, 6, 7, 0, 0, 0]),
        0xE4 => simd_swizzle!(val, [2, 5, 6, 7, 0, 0, 0, 0]),
        0xE5 => simd_swizzle!(val, [0, 2, 5, 6, 7, 0, 0, 0]),
        0xE6 => simd_swizzle!(val, [1, 2, 5, 6, 7, 0, 0, 0]),
        0xE7 => simd_swizzle!(val, [0, 1, 2, 5, 6, 7, 0, 0]),
        0xE8 => simd_swizzle!(val, [3, 5, 6, 7, 0, 0, 0, 0]),
        0xE9 => simd_swizzle!(val, [0, 3, 5, 6, 7, 0, 0, 0]),
        0xEA => simd_swizzle!(val, [1, 3, 5, 6, 7, 0, 0, 0]),
        0xEB => simd_swizzle!(val, [0, 1, 3, 5, 6, 7, 0, 0]),
        0xEC => simd_swizzle!(val, [2, 3, 5, 6, 7, 0, 0, 0]),
        0xED => simd_swizzle!(val, [0, 2, 3, 5, 6, 7, 0, 0]),
        0xEE => simd_swizzle!(val, [1, 2, 3, 5, 6, 7, 0, 0]),
        0xEF => simd_swizzle!(val, [0, 1, 2, 3, 5, 6, 7, 0]),
        0xF0 => simd_swizzle!(val, [4, 5, 6, 7, 0, 0, 0, 0]),
        0xF1 => simd_swizzle!(val, [0, 4, 5, 6, 7, 0, 0, 0]),
        0xF2 => simd_swizzle!(val, [1, 4, 5, 6, 7, 0, 0, 0]),
        0xF3 => simd_swizzle!(val, [0, 1, 4, 5, 6, 7, 0, 0]),
        0xF4 => simd_swizzle!(val, [2, 4, 5, 6, 7, 0, 0, 0]),
        0xF5 => simd_swizzle!(val, [0, 2, 4, 5, 6, 7, 0, 0]),
        0xF6 => simd_swizzle!(val, [1, 2, 4, 5, 6, 7, 0, 0]),
        0xF7 => simd_swizzle!(val, [0, 1, 2, 4, 5, 6, 7, 0]),
        0xF8 => simd_swizzle!(val, [3, 4, 5, 6, 7, 0, 0, 0]),
        0xF9 => simd_swizzle!(val, [0, 3, 4, 5, 6, 7, 0, 0]),
        0xFA => simd_swizzle!(val, [1, 3, 4, 5, 6, 7, 0, 0]),
        0xFB => simd_swizzle!(val, [0, 1, 3, 4, 5, 6, 7, 0]),
        0xFC => simd_swizzle!(val, [2, 3, 4, 5, 6, 7, 0, 0]),
        0xFD => simd_swizzle!(val, [0, 2, 3, 4, 5, 6, 7, 0]),
        0xFE => simd_swizzle!(val, [1, 2, 3, 4, 5, 6, 7, 0]),
        0xFF => simd_swizzle!(val, [0, 1, 2, 3, 4, 5, 6, 7]),
    }
}
