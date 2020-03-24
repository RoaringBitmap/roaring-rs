#[inline]
pub fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
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
