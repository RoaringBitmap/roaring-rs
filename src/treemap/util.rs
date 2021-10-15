use std::ops::{Bound, RangeBounds, RangeInclusive};

#[inline]
pub fn split(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
}

#[inline]
pub fn join(high: u32, low: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}

/// Convert a `RangeBounds<u64>` object to `RangeInclusive<u64>`,
pub fn convert_range_to_inclusive<R>(range: R) -> Option<RangeInclusive<u64>>
where
    R: RangeBounds<u64>,
{
    if let Bound::Excluded(0) = range.end_bound() {
        return None;
    }
    let start: u64 = match range.start_bound() {
        Bound::Included(&i) => i,
        Bound::Unbounded => 0,
        _ => panic!("Should never be called (insert_range start with Excluded)"),
    };
    let end: u64 = match range.end_bound() {
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => i - 1,
        Bound::Unbounded => u64::MAX,
    };
    if end < start {
        return None;
    }
    Some(start..=end)
}

#[cfg(test)]
mod test {
    use super::{join, split};

    #[test]
    fn test_split_u64() {
        assert_eq!((0x0000_0000u32, 0x0000_0000u32), split(0x0000_0000_0000_0000u64));
        assert_eq!((0x0000_0000u32, 0x0000_0001u32), split(0x0000_0000_0000_0001u64));
        assert_eq!((0x0000_0000u32, 0xFFFF_FFFEu32), split(0x0000_0000_FFFF_FFFEu64));
        assert_eq!((0x0000_0000u32, 0xFFFF_FFFFu32), split(0x0000_0000_FFFF_FFFFu64));
        assert_eq!((0x0000_0001u32, 0x0000_0000u32), split(0x0000_0001_0000_0000u64));
        assert_eq!((0x0000_0001u32, 0x0000_0001u32), split(0x0000_0001_0000_0001u64));
        assert_eq!((0xFFFF_FFFFu32, 0xFFFF_FFFEu32), split(0xFFFF_FFFF_FFFF_FFFEu64));
        assert_eq!((0xFFFF_FFFFu32, 0xFFFF_FFFFu32), split(0xFFFF_FFFF_FFFF_FFFFu64));
    }

    #[test]
    fn test_join_u64() {
        assert_eq!(0x0000_0000_0000_0000u64, join(0x0000_0000u32, 0x0000_0000u32));
        assert_eq!(0x0000_0000_0000_0001u64, join(0x0000_0000u32, 0x0000_0001u32));
        assert_eq!(0x0000_0000_FFFF_FFFEu64, join(0x0000_0000u32, 0xFFFF_FFFEu32));
        assert_eq!(0x0000_0000_FFFF_FFFFu64, join(0x0000_0000u32, 0xFFFF_FFFFu32));
        assert_eq!(0x0000_0001_0000_0000u64, join(0x0000_0001u32, 0x0000_0000u32));
        assert_eq!(0x0000_0001_0000_0001u64, join(0x0000_0001u32, 0x0000_0001u32));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFEu64, join(0xFFFF_FFFFu32, 0xFFFF_FFFEu32));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFFu64, join(0xFFFF_FFFFu32, 0xFFFF_FFFFu32));
    }
}
