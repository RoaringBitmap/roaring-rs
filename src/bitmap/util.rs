use std::ops::{Bound, RangeBounds, RangeInclusive};

/// Returns the container key and the index
/// in this container for a given integer.
#[inline]
pub fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

/// Returns the original integer from the container
/// key and the index of it in the container.
#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
}

/// Convert a `RangeBounds` object to `RangeInclusive<u32>`,
/// and return if the range is empty.
fn convert_range_to_inclusive<R>(range: R) -> (RangeInclusive<u32>, bool)
where
    R: RangeBounds<u32>,
{
    if let Bound::Excluded(0) = range.end_bound() {
        return (0..=0, true);
    }
    let start: u32 = match range.start_bound() {
        Bound::Included(&i) => i,
        Bound::Unbounded => 0,
        _ => panic!("Should never be called (insert_range start with Excluded)"),
    };
    let end: u32 = match range.end_bound() {
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => i - 1,
        Bound::Unbounded => u32::MAX,
    };
    if end < start {
        return (0..=0, true);
    }
    (start..=end, false)
}

#[cfg(test)]
mod test {
    use super::{join, split};

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
}
