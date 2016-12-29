#[inline]
pub fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    ((high as u32) << 16) + (low as u32)
}

#[cfg(test)]
mod test {
    use super::{ split, join };

    #[test]
    fn test_split_u32() {
        assert_eq!((0x0000u16, 0x0000u16), split(0x00000000u32));
        assert_eq!((0x0000u16, 0x0001u16), split(0x00000001u32));
        assert_eq!((0x0000u16, 0xFFFEu16), split(0x0000FFFEu32));
        assert_eq!((0x0000u16, 0xFFFFu16), split(0x0000FFFFu32));
        assert_eq!((0x0001u16, 0x0000u16), split(0x00010000u32));
        assert_eq!((0x0001u16, 0x0001u16), split(0x00010001u32));
        assert_eq!((0xFFFFu16, 0xFFFEu16), split(0xFFFFFFFEu32));
        assert_eq!((0xFFFFu16, 0xFFFFu16), split(0xFFFFFFFFu32));
    }

    #[test]
    fn test_join_u32() {
        assert_eq!(0x00000000u32, join(0x0000u16, 0x0000u16));
        assert_eq!(0x00000001u32, join(0x0000u16, 0x0001u16));
        assert_eq!(0x0000FFFEu32, join(0x0000u16, 0xFFFEu16));
        assert_eq!(0x0000FFFFu32, join(0x0000u16, 0xFFFFu16));
        assert_eq!(0x00010000u32, join(0x0001u16, 0x0000u16));
        assert_eq!(0x00010001u32, join(0x0001u16, 0x0001u16));
        assert_eq!(0xFFFFFFFEu32, join(0xFFFFu16, 0xFFFEu16));
        assert_eq!(0xFFFFFFFFu32, join(0xFFFFu16, 0xFFFFu16));
    }
}
