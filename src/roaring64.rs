use crate::{ContainerKey, RoaringBitmap, Value, ValueRange};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io,
    ops::{Bound, RangeBounds, RangeInclusive},
};

/// A compressed bitmap for 64-bit values.
///
/// # Examples
///
/// ```rust
/// use roaring::Roaring64;
///
/// let mut rb = Roaring64::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
pub type Roaring64 = RoaringBitmap<u64>;

impl Value for u64 {
    type Key = u64;
    type Range = RangeInclusive<Self>;

    fn split(self) -> (Self::Key, u16) {
        (self >> 16, self as u16)
    }

    fn join(key: Self::Key, index: u16) -> Self {
        (key << 16) + u64::from(index)
    }

    fn range(range: impl RangeBounds<Self>) -> Option<Self::Range> {
        let start: u64 = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.checked_add(1)?,
            Bound::Unbounded => 0,
        };
        let end: u64 = match range.end_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.checked_sub(1)?,
            Bound::Unbounded => u64::MAX,
        };

        if end < start {
            return None;
        }

        Some(start..=end)
    }

    fn max_containers() -> usize {
        // Theoretically, u64::MAX + 1.
        // Realistically we're probably capped at usize anyway.
        usize::MAX
    }
}

impl ContainerKey for u64 {
    #[inline(always)]
    fn size() -> usize {
        // Key is coded on 48-bit, the 16 upper ones are unused.
        6
    }

    fn write(self, writer: &mut impl WriteBytesExt) -> io::Result<()> {
        writer.write_u48::<LittleEndian>(self)
    }

    fn read(reader: &mut impl ReadBytesExt) -> io::Result<Self> {
        reader.read_u48::<LittleEndian>()
    }
}

impl ValueRange<u64> for RangeInclusive<u64> {
    type KeyIterator = RangeInclusive<u64>;

    fn start(&self) -> (<u64 as Value>::Key, u16) {
        self.start().split()
    }

    fn end(&self) -> (<u64 as Value>::Key, u16) {
        self.end().split()
    }

    fn containers_count(&self) -> usize {
        let start = ValueRange::start(self).0;
        let end = ValueRange::end(self).0;
        (end - start) as usize + 1
    }

    fn keys(self) -> Self::KeyIterator {
        let start = ValueRange::start(&self).0;
        let end = ValueRange::end(&self).0;
        start..=end
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split() {
        assert_eq!((0x0000_0000_0000u64, 0x0000u16), 0x0000_0000_0000_0000u64.split());
        assert_eq!((0x0000_0000_0000u64, 0x0001u16), 0x0000_0000_0000_0001u64.split());
        assert_eq!((0x0000_0000_FFFFu64, 0xFFFEu16), 0x0000_0000_FFFF_FFFEu64.split());
        assert_eq!((0x0000_0000_FFFFu64, 0xFFFFu16), 0x0000_0000_FFFF_FFFFu64.split());
        assert_eq!((0x0000_0001_0000u64, 0x0000u16), 0x0000_0001_0000_0000u64.split());
        assert_eq!((0x0000_0001_0000u64, 0x0001u16), 0x0000_0001_0000_0001u64.split());
        assert_eq!((0xFFFF_FFFF_FFFFu64, 0xFFFEu16), 0xFFFF_FFFF_FFFF_FFFEu64.split());
        assert_eq!((0xFFFF_FFFF_FFFFu64, 0xFFFFu16), 0xFFFF_FFFF_FFFF_FFFFu64.split());
    }

    #[test]
    fn join() {
        assert_eq!(0x0000_0000_0000_0000u64, u64::join(0x0000_0000_0000u64, 0x0000u16));
        assert_eq!(0x0000_0000_0000_0001u64, u64::join(0x0000_0000_0000u64, 0x0001u16));
        assert_eq!(0x0000_0000_FFFF_FFFEu64, u64::join(0x0000_0000_FFFFu64, 0xFFFEu16));
        assert_eq!(0x0000_0000_FFFF_FFFFu64, u64::join(0x0000_0000_FFFFu64, 0xFFFFu16));
        assert_eq!(0x0000_0001_0000_0000u64, u64::join(0x0000_0001_0000u64, 0x0000u16));
        assert_eq!(0x0000_0001_0000_0001u64, u64::join(0x0000_0001_0000u64, 0x0001u16));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFEu64, u64::join(0xFFFF_FFFF_FFFFu64, 0xFFFEu16));
        assert_eq!(0xFFFF_FFFF_FFFF_FFFFu64, u64::join(0xFFFF_FFFF_FFFFu64, 0xFFFFu16));
    }

    #[test]
    fn range() {
        assert_eq!(Some(1..=5), u64::range(1..6));
        assert_eq!(Some(1..=u64::MAX), u64::range(1..));
        assert_eq!(Some(0..=u64::MAX), u64::range(..));
        assert_eq!(None, u64::range(5..5));
        assert_eq!(Some(16..=16), u64::range(16..=16))
    }
}
