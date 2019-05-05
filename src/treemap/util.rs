#[inline]
pub fn split(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
}

#[inline]
pub fn join(high: u32, low: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}

#[cfg(test)]
mod test {
    use super::{split, join};

    #[test]
    fn test_split_u64() {
        assert_eq!((0x00000000u32, 0x00000000u32), split(0x0000000000000000u64));
        assert_eq!((0x00000000u32, 0x00000001u32), split(0x0000000000000001u64));
        assert_eq!((0x00000000u32, 0xFFFFFFFEu32), split(0x00000000FFFFFFFEu64));
        assert_eq!((0x00000000u32, 0xFFFFFFFFu32), split(0x00000000FFFFFFFFu64));
        assert_eq!((0x00000001u32, 0x00000000u32), split(0x0000000100000000u64));
        assert_eq!((0x00000001u32, 0x00000001u32), split(0x0000000100000001u64));
        assert_eq!((0xFFFFFFFFu32, 0xFFFFFFFEu32), split(0xFFFFFFFFFFFFFFFEu64));
        assert_eq!((0xFFFFFFFFu32, 0xFFFFFFFFu32), split(0xFFFFFFFFFFFFFFFFu64));
    }

    #[test]
    fn test_join_u64() {
        assert_eq!(0x0000000000000000u64, join(0x00000000u32, 0x00000000u32));
        assert_eq!(0x0000000000000001u64, join(0x00000000u32, 0x00000001u32));
        assert_eq!(0x00000000FFFFFFFEu64, join(0x00000000u32, 0xFFFFFFFEu32));
        assert_eq!(0x00000000FFFFFFFFu64, join(0x00000000u32, 0xFFFFFFFFu32));
        assert_eq!(0x0000000100000000u64, join(0x00000001u32, 0x00000000u32));
        assert_eq!(0x0000000100000001u64, join(0x00000001u32, 0x00000001u32));
        assert_eq!(0xFFFFFFFFFFFFFFFEu64, join(0xFFFFFFFFu32, 0xFFFFFFFEu32));
        assert_eq!(0xFFFFFFFFFFFFFFFFu64, join(0xFFFFFFFFu32, 0xFFFFFFFFu32));
    }
}
