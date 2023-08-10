use crate::{ContainerKey, RoaringBitmap, Value, ValueRange};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io,
    ops::{Bound, RangeBounds, RangeInclusive},
};

/// A compressed bitmap for 32-bit values.
///
/// # Examples
///
/// ```rust
/// use roaring::Roaring32;
///
/// let mut rb = Roaring32::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
pub type Roaring32 = RoaringBitmap<u32>;

impl Value for u32 {
    type Key = u16;
    type Range = RangeInclusive<Self>;

    fn split(self) -> (Self::Key, u16) {
        ((self >> 16) as Self::Key, self as u16)
    }

    fn join(key: Self::Key, index: u16) -> Self {
        (u32::from(key) << 16) + u32::from(index)
    }

    fn range(range: impl RangeBounds<Self>) -> Option<Self::Range> {
        let start: u32 = match range.start_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.checked_add(1)?,
            Bound::Unbounded => 0,
        };
        let end: u32 = match range.end_bound() {
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.checked_sub(1)?,
            Bound::Unbounded => u32::MAX,
        };

        if end < start {
            return None;
        }

        Some(start..=end)
    }

    fn max_containers() -> usize {
        usize::from(Self::Key::MAX) + 1
    }
}

impl ContainerKey for u16 {
    #[inline(always)]
    fn size() -> usize {
        std::mem::size_of::<u16>()
    }

    fn write(self, writer: &mut impl WriteBytesExt) -> io::Result<()> {
        writer.write_u16::<LittleEndian>(self)
    }

    fn read(reader: &mut impl ReadBytesExt) -> io::Result<Self> {
        reader.read_u16::<LittleEndian>()
    }
}

impl ValueRange<u16> for RangeInclusive<u32> {
    type KeyIterator = RangeInclusive<u16>;

    fn start(&self) -> (<u32 as Value>::Key, u16) {
        self.start().split()
    }

    fn end(&self) -> (<u32 as Value>::Key, u16) {
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
    use std::ops::Bound;

    #[test]
    fn split() {
        assert_eq!((0x0000u16, 0x0000u16), 0x0000_0000u32.split());
        assert_eq!((0x0000u16, 0x0001u16), 0x0000_0001u32.split());
        assert_eq!((0x0000u16, 0xFFFEu16), 0x0000_FFFEu32.split());
        assert_eq!((0x0000u16, 0xFFFFu16), 0x0000_FFFFu32.split());
        assert_eq!((0x0001u16, 0x0000u16), 0x0001_0000u32.split());
        assert_eq!((0x0001u16, 0x0001u16), 0x0001_0001u32.split());
        assert_eq!((0xFFFFu16, 0xFFFEu16), 0xFFFF_FFFEu32.split());
        assert_eq!((0xFFFFu16, 0xFFFFu16), 0xFFFF_FFFFu32.split());
    }

    #[test]
    fn join() {
        assert_eq!(0x0000_0000u32, u32::join(0x0000u16, 0x0000u16));
        assert_eq!(0x0000_0001u32, u32::join(0x0000u16, 0x0001u16));
        assert_eq!(0x0000_FFFEu32, u32::join(0x0000u16, 0xFFFEu16));
        assert_eq!(0x0000_FFFFu32, u32::join(0x0000u16, 0xFFFFu16));
        assert_eq!(0x0001_0000u32, u32::join(0x0001u16, 0x0000u16));
        assert_eq!(0x0001_0001u32, u32::join(0x0001u16, 0x0001u16));
        assert_eq!(0xFFFF_FFFEu32, u32::join(0xFFFFu16, 0xFFFEu16));
        assert_eq!(0xFFFF_FFFFu32, u32::join(0xFFFFu16, 0xFFFFu16));
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn range() {
        assert_eq!(Some(1..=5), u32::range(1..6));
        assert_eq!(Some(1..=u32::MAX), u32::range(1..));
        assert_eq!(Some(0..=u32::MAX), u32::range(..));
        assert_eq!(Some(16..=16), u32::range(16..=16));
        assert_eq!(Some(11..=19), u32::range((Bound::Excluded(10), Bound::Excluded(20))));

        assert_eq!(None, u32::range(0..0));
        assert_eq!(None, u32::range(5..5));
        assert_eq!(None, u32::range(1..0));
        assert_eq!(None, u32::range(10..5));
        assert_eq!(None, u32::range((Bound::Excluded(u32::MAX), Bound::Included(u32::MAX))));
        assert_eq!(None, u32::range((Bound::Excluded(u32::MAX), Bound::Included(u32::MAX))));
        assert_eq!(None, u32::range((Bound::Excluded(0), Bound::Included(0))));
    }
}
