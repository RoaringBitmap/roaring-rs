use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{fmt, io, ops::RangeBounds};

/// A Roaring Bitmap value.
///
/// Internally, a value is split into a container key and a container index.
pub trait Value: fmt::Debug + Copy + Ord + Into<u64> {
    /// Type for the container key.
    type Key: ContainerKey;
    /// Type for a range of values.
    type Range: ValueRange<Self::Key>;

    /// Splits the values into a (key, index) pair.
    fn split(self) -> (Self::Key, u16);

    /// Returns the original value from a (key, index) pair.
    fn join(key: Self::Key, index: u16) -> Self;

    /// Returns a range of value from the givem bounds.
    fn range(range: impl RangeBounds<Self>) -> Option<Self::Range>;

    /// Return the number of containers used to cover every possible values.
    fn max_containers() -> usize;
}

/// Key for a Roaring Bitmap container.
pub trait ContainerKey: fmt::Debug + Copy + Ord {
    /// Returns the size (in byte) of the key.
    fn size() -> usize;

    /// Writes the container key to the given writer.
    fn write(self, writer: &mut impl WriteBytesExt) -> io::Result<()>;

    /// Reads the container key from the given reader.
    fn read(reader: &mut impl ReadBytesExt) -> io::Result<Self>;
}

/// A range of value to insert.
pub trait ValueRange<K> {
    /// Iterator over the keys covered by the values.
    type KeyIterator: Iterator<Item = K>;

    /// Returns the start of the value range.
    fn start(&self) -> (K, u16);

    /// Returns the end of the value range.
    fn end(&self) -> (K, u16);

    /// Returns the number of containers covered by the range.
    fn containers_count(&self) -> usize;

    /// Returns an iterator over the container keys.
    fn keys(self) -> Self::KeyIterator;
}
