#[inline]
pub fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
}


/// Branchless binary search going after 4 values at once.
/// Assumes that array is sorted.
///
/// You have that array[*index1] >= target1, array[*index12] >= target2, ...
/// except when *index1 = n, in which case you know that all values in array are
/// smaller than target1, and so forth.
///
/// It has logarithmic complexity.
#[inline]
fn binary_search_4(
    array: &[u16],
    target1: u16,
    target2: u16,
    target3: u16,
    target4: u16,
    index1: &mut usize,
    index2: &mut usize,
    index3: &mut usize,
    index4: &mut usize,
) {
    if array.is_empty() {
        return;
    }

    let mut base1 = 0;
    let mut base2 = 0;
    let mut base3 = 0;
    let mut base4 = 0;
    let mut n = array.len();

    while n > 1 {
        let half = n >> 1;
        base1 = if array[base1 + half] < target1 { base1 + half } else { base1 };
        base2 = if array[base2 + half] < target2 { base2 + half } else { base2 };
        base3 = if array[base3 + half] < target3 { base3 + half } else { base3 };
        base4 = if array[base4 + half] < target4 { base4 + half } else { base4 };
        n -= half;
    }

    *index1 = (array[base1] < target1) as usize + base1;
    *index2 = (array[base2] < target2) as usize + base2;
    *index3 = (array[base3] < target3) as usize + base3;
    *index4 = (array[base4] < target4) as usize + base4;
}

/// Branchless binary search going after 2 values at once.
/// Assumes that array is sorted.
///
/// You have that array[*index1] >= target1, array[*index12] >= target2.
/// except when *index1 = n, in which case you know that all values in array are
/// smaller than target1, and so forth.
///
/// It has logarithmic complexity.
#[inline]
fn binary_search_2(
    array: &[u16],
    target1: u16,
    target2: u16,
    index1: &mut usize,
    index2: &mut usize,
) {
    if array.is_empty() {
        return;
    }

    let mut base1 = 0;
    let mut base2 = 0;
    let mut n = array.len();

    while n > 1 {
        let half = n >> 1;
        base1 = if array[base1 + half] < target1 { base1 + half } else { base1 };
        base2 = if array[base2 + half] < target2 { base2 + half } else { base2 };
        n -= half;
    }

    *index1 = (array[base1] < target1) as usize + base1;
    *index2 = (array[base2] < target2) as usize + base2;
}

/// Computes the intersection between one small and one large set of uint16_t.
///
/// Stores the result into buffer.
/// Processes the small set in blocks of 4 values calling binary_search_4
/// and binary_search_2. This approach can be slightly superior to a conventional
/// galloping search in some instances.
#[inline]
pub fn intersect_skewed_u16(small: &[u16], large: &[u16]) -> Vec<u16> {
    if small.is_empty() {
        return Vec::new();
    }

    let mut buffer = Vec::with_capacity(small.len());
    let mut idx_small = 0;
    let mut idx_large = 0;
    let mut index1 = 0;
    let mut index2 = 0;
    let mut index3 = 0;
    let mut index4 = 0;

    while idx_small + 4 <= small.len() && idx_large < large.len() {
        let target1 = small[idx_small];
        let target2 = small[idx_small + 1];
        let target3 = small[idx_small + 2];
        let target4 = small[idx_small + 3];

        binary_search_4(
            &large[idx_large..],
            target1, target2, target3, target4,
            &mut index1, &mut index2, &mut index3, &mut index4,
        );

        if index1 + idx_large < large.len() && large[idx_large + index1] == target1 {
            buffer.push(target1);
        }
        if index2 + idx_large < large.len() && large[idx_large + index2] == target2 {
            buffer.push(target2);
        }
        if index3 + idx_large < large.len() && large[idx_large + index3] == target3 {
            buffer.push(target3);
        }
        if index4 + idx_large < large.len() && large[idx_large + index4] == target4 {
            buffer.push(target4);
        }

        idx_small += 4;
        idx_large += index4;
    }

    if idx_small + 2 <= small.len() && idx_large < large.len() {
        let target1 = small[idx_small];
        let target2 = small[idx_small + 1];

        binary_search_2(&large[idx_large..], target1, target2, &mut index1, &mut index2);

        if index1 + idx_large < large.len() && large[idx_large + index1] == target1 {
            buffer.push(target1);
        }
        if index2 + idx_large < large.len() && large[idx_large + index2] == target2 {
            buffer.push(target2);
        }

        idx_small += 2;
        idx_large += index2;
    }

    if idx_small < small.len() && idx_large < large.len() {
        let val_s = small[idx_small];
        if large[idx_large..].binary_search(&val_s).is_ok() {
            buffer.push(val_s);
        }
    }

    buffer
}

/// Generic intersection function.
pub fn intersect_uint16(a: &[u16], b: &[u16]) -> Vec<u16> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }

    let mut buffer = Vec::with_capacity(a.len().min(b.len()));
    let mut idx_a = 0;
    let mut idx_b = 0;

    loop {
        while a[idx_a] < b[idx_b] {
            idx_a += 1;
            if idx_a >= a.len() {
                return buffer;
            }
        }

        while a[idx_a] > b[idx_b] {
            idx_b += 1;
            if idx_b >= b.len() {
                return buffer;
            };
        }

        if a[idx_a] == b[idx_b] {
            buffer.push(a[idx_a]);
            idx_a += 1;
            if idx_a >= a.len() || idx_b >= b.len() {
                return buffer;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_u32() {
        assert_eq!((0x0000u16, 0x0000u16), split(0x0000_0000u32));
        assert_eq!((0x0000u16, 0x0001u16), split(0x0000_0001u32));
        assert_eq!((0x0000u16, 0xFFFEu16), split(0x0000_FFFEu32));
        assert_eq!((0x0000u16, 0xFFFFu16), split(0x0000_FFFFu32));
        assert_eq!((0x0001u16, 0x0000u16), split(0x0001_0000u32));
        assert_eq!((0x0001u16, 0x0001u16), split(0x0001_0001u32));
        assert_eq!((0xFFFFu16, 0xFFFEu16), split(0xFFFF_FFFEu32));
        assert_eq!((0xFFFFu16, 0xFFFFu16), split(0xFFFF_FFFFu32));
    }

    #[test]
    fn test_join_u32() {
        assert_eq!(0x0000_0000u32, join(0x0000u16, 0x0000u16));
        assert_eq!(0x0000_0001u32, join(0x0000u16, 0x0001u16));
        assert_eq!(0x0000_FFFEu32, join(0x0000u16, 0xFFFEu16));
        assert_eq!(0x0000_FFFFu32, join(0x0000u16, 0xFFFFu16));
        assert_eq!(0x0001_0000u32, join(0x0001u16, 0x0000u16));
        assert_eq!(0x0001_0001u32, join(0x0001u16, 0x0001u16));
        assert_eq!(0xFFFF_FFFEu32, join(0xFFFFu16, 0xFFFEu16));
        assert_eq!(0xFFFF_FFFFu32, join(0xFFFFu16, 0xFFFFu16));
    }

    #[test]
    fn test_binary_search_2() {
        let array: Vec<u16> = (0..=3534u16).collect();
        let target1 = 10;
        let target2 = 450;

        let mut found_i1 = 0;
        let mut found_i2 = 0;
        binary_search_2(&array, target1, target2, &mut found_i1, &mut found_i2);

        let expected_i1 = array.binary_search(&target1).unwrap();
        let expected_i2 = array.binary_search(&target2).unwrap();

        assert_eq!(found_i1, expected_i1);
        assert_eq!(found_i2, expected_i2);
    }

    #[test]
    fn test_binary_search_4() {
        let array: Vec<u16> = (0..=3534u16).collect();
        let target1 = 10;
        let target2 = 450;
        let target3 = 1786;
        let target4 = 2923;

        let mut found_i1 = 0;
        let mut found_i2 = 0;
        let mut found_i3 = 0;
        let mut found_i4 = 0;
        binary_search_4(&array,
            target1, target2, target3, target4,
            &mut found_i1, &mut found_i2, &mut found_i3, &mut found_i4);

        let expected_i1 = array.binary_search(&target1).unwrap();
        let expected_i2 = array.binary_search(&target2).unwrap();
        let expected_i3 = array.binary_search(&target3).unwrap();
        let expected_i4 = array.binary_search(&target4).unwrap();

        assert_eq!(found_i1, expected_i1);
        assert_eq!(found_i2, expected_i2);
        assert_eq!(found_i3, expected_i3);
        assert_eq!(found_i4, expected_i4);
    }
}
