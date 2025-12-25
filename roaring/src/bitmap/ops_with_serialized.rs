use bytemuck::cast_slice_mut;
use byteorder::{LittleEndian, ReadBytesExt};

use std::cmp::Ordering;
use std::io::{self, SeekFrom};
use std::mem;

use crate::bitmap::container::Container;
use crate::bitmap::serialization::{
    NO_OFFSET_THRESHOLD, SERIAL_COOKIE, SERIAL_COOKIE_NO_RUNCONTAINER,
};
use crate::RoaringBitmap;

use super::container::ARRAY_LIMIT;
use super::store::{ArrayStore, BitmapStore, Store, BITMAP_LENGTH};

impl RoaringBitmap {
    /// Computes the intersection between a materialized [`RoaringBitmap`] and a serialized one.
    ///
    /// This is faster and more space efficient when you only need the intersection result.
    /// It reduces the number of deserialized internal container and therefore
    /// the number of allocations and copies of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use std::io::Cursor;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    /// // Let's say the rb2 bitmap is serialized
    /// let mut bytes = Vec::new();
    /// rb2.serialize_into(&mut bytes).unwrap();
    /// let rb2_bytes = Cursor::new(bytes);
    ///
    /// assert_eq!(
    ///     rb1.intersection_with_serialized_unchecked(rb2_bytes).unwrap(),
    ///     rb1 & rb2,
    /// );
    /// ```
    pub fn intersection_with_serialized_unchecked<R>(
        &self,
        mut other: R,
    ) -> io::Result<RoaringBitmap>
    where
        R: io::Read + io::Seek,
    {
        let metadata = BitmapReader::decode(&mut other)?;
        let containers = Visitor {
            containers: &self.containers,
            metadata: &metadata,
            handler: &mut BitAndHandler,
        }
        .visit(&mut other)?;
        Ok(RoaringBitmap { containers })
    }

    /// Computes the union between a materialized [`RoaringBitmap`] and a serialized one.
    ///
    /// This is faster and more space efficient when you only need the union result.
    /// It reduces the number of deserialized internal container and therefore
    /// the number of allocations and copies of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use std::io::Cursor;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    /// // Let's say the rb2 bitmap is serialized
    /// let mut bytes = Vec::new();
    /// rb2.serialize_into(&mut bytes).unwrap();
    /// let rb2_bytes = Cursor::new(bytes);
    ///
    /// assert_eq!(
    ///     rb1.union_with_serialized_unchecked(rb2_bytes).unwrap(),
    ///     rb1 | rb2,
    /// );
    /// ```
    pub fn union_with_serialized_unchecked<R>(&self, mut other: R) -> io::Result<RoaringBitmap>
    where
        R: io::Read + io::Seek,
    {
        let metadata = BitmapReader::decode(&mut other)?;
        let containers = Visitor {
            containers: &self.containers,
            metadata: &metadata,
            handler: &mut BitOrHandler,
        }
        .visit(&mut other)?;
        Ok(RoaringBitmap { containers })
    }
}

struct Visitor<'a, H> {
    containers: &'a [Container],
    metadata: &'a BitmapReader,
    handler: &'a mut H,
}

impl<H> Visitor<'_, H>
where
    H: VisitorHandler,
{
    fn visit<R>(&mut self, reader: &mut R) -> io::Result<Vec<Container>>
    where
        R: io::Read + io::Seek,
    {
        let mut result = Vec::new();
        let mut descriptions = self
            .metadata
            .descriptions
            .iter()
            .enumerate()
            .map(|(i, &[key, len_minus_one])| MetaItem {
                key,
                cardinality: len_minus_one as u32 + 1,
                is_run: self.metadata.is_run_container(i),
                offset: self.metadata.offsets.as_ref().map(|offsets| offsets[i]),
            })
            .peekable();
        let mut containers = self.containers.iter().peekable();

        loop {
            match (containers.peek(), descriptions.peek()) {
                (Some(container), Some(item)) => match item.key.cmp(&container.key) {
                    Ordering::Equal => {
                        result.extend(self.consume_matched(reader, container, item)?);
                        descriptions.next();
                        containers.next();
                    }
                    Ordering::Less => {
                        result.extend(self.consume_right(reader, item)?);
                        descriptions.next();
                    }
                    Ordering::Greater => {
                        result.extend(self.consume_left(container)?);
                        containers.next();
                    }
                },
                (None, Some(item)) => {
                    result.extend(self.consume_right(reader, item)?);
                    descriptions.next();
                }
                (Some(container), None) => {
                    result.extend(self.consume_left(container)?);
                    containers.next();
                }
                (None, None) => {
                    return Ok(result);
                }
            }
        }
    }

    fn consume_left(&mut self, container: &Container) -> io::Result<Option<Container>> {
        self.handler.handle_left_only(container)
    }

    fn consume_right<R>(&mut self, reader: &mut R, item: &MetaItem) -> io::Result<Option<Container>>
    where
        R: io::Read + io::Seek,
    {
        if self.handler.need_handle_right_only(item.key) {
            let container = item.load_container(reader)?;
            self.handler.handle_right_only(container)
        } else if item.offset.is_some() {
            Ok(None)
        } else {
            item.skip(reader)?;
            Ok(None)
        }
    }

    fn consume_matched<R>(
        &mut self,
        reader: &mut R,
        left: &Container,
        item: &MetaItem,
    ) -> io::Result<Option<Container>>
    where
        R: io::Read + io::Seek,
    {
        if !self.handler.need_handle_matched(left) {
            if item.offset.is_none() {
                item.skip(reader)?;
            }
            return Ok(None);
        }

        if let Some(offset) = item.offset {
            let absolute_offset = self
                .metadata
                .base_offset
                .checked_add(offset as u64)
                .ok_or_else(|| io::Error::other("offset overflow"))?;
            reader.seek(SeekFrom::Start(absolute_offset))?;
        }
        let right = item.load_container(reader)?;
        self.handler.handel_matched(left, right)
    }
}

struct MetaItem {
    key: u16,
    cardinality: u32,
    is_run: bool,
    offset: Option<u32>,
}

impl MetaItem {
    fn load_container<R: io::Read>(&self, reader: &mut R) -> io::Result<Container> {
        let store = if self.is_run {
            let runs = reader.read_u16::<LittleEndian>()?;
            let mut intervals = vec![[0_u16, 0]; runs as usize];
            reader.read_u16_into::<LittleEndian>(cast_slice_mut(&mut intervals))?;

            let cardinality = intervals.iter().map(|[_, len]| *len as usize).sum();
            let mut store = Store::with_capacity(cardinality);

            for [s, len] in intervals {
                let end = s.checked_add(len).ok_or(io::ErrorKind::InvalidData)?;
                store.insert_range(s..=end);
            }
            store
        } else if self.cardinality as u64 <= ARRAY_LIMIT {
            let mut values = vec![0; self.cardinality as usize];
            reader.read_u16_into::<LittleEndian>(&mut values)?;
            let array = ArrayStore::from_vec_unchecked(values);
            Store::Array(array)
        } else {
            let mut values = Box::new([0; BITMAP_LENGTH]);
            reader.read_u64_into::<LittleEndian>(values.as_mut_slice())?;
            let bitmap = BitmapStore::from_unchecked(self.cardinality as u64, values);
            Store::Bitmap(bitmap)
        };
        Ok(Container { key: self.key, store })
    }

    fn skip<R: io::Read + io::Seek>(&self, reader: &mut R) -> io::Result<()> {
        if self.is_run {
            let runs = reader.read_u16::<LittleEndian>()?;
            let runs_size = mem::size_of::<u16>() * 2 * runs as usize;
            reader.seek_relative(runs_size as i64)?;
        } else if self.cardinality as u64 <= ARRAY_LIMIT {
            let array_size = mem::size_of::<u16>() * self.cardinality as usize;
            reader.seek_relative(array_size as i64)?;
        } else {
            let bitmap_size = mem::size_of::<u64>() * BITMAP_LENGTH;
            reader.seek_relative(bitmap_size as i64)?;
        }
        Ok(())
    }
}

trait VisitorHandler {
    fn handle_left_only(&mut self, container: &Container) -> io::Result<Option<Container>>;

    fn need_handle_right_only(&mut self, _key: u16) -> bool {
        false
    }

    fn handle_right_only(&mut self, _container: Container) -> io::Result<Option<Container>> {
        unreachable!()
    }

    fn need_handle_matched(&mut self, _container: &Container) -> bool {
        true
    }

    fn handel_matched(
        &mut self,
        left: &Container,
        right: Container,
    ) -> io::Result<Option<Container>>;
}

struct BitAndHandler;

impl VisitorHandler for BitAndHandler {
    fn handle_left_only(&mut self, _container: &Container) -> io::Result<Option<Container>> {
        Ok(None)
    }

    fn handel_matched(
        &mut self,
        left: &Container,
        mut right: Container,
    ) -> io::Result<Option<Container>> {
        right &= left;
        if right.is_empty() {
            Ok(None)
        } else {
            Ok(Some(right))
        }
    }
}

struct BitOrHandler;

impl VisitorHandler for BitOrHandler {
    fn handle_left_only(&mut self, container: &Container) -> io::Result<Option<Container>> {
        Ok(Some(container.clone()))
    }

    fn need_handle_right_only(&mut self, _key: u16) -> bool {
        true
    }

    fn handle_right_only(&mut self, container: Container) -> io::Result<Option<Container>> {
        Ok(Some(container))
    }

    fn handel_matched(
        &mut self,
        left: &Container,
        mut right: Container,
    ) -> io::Result<Option<Container>> {
        right |= left;
        if right.is_empty() {
            Ok(None)
        } else {
            Ok(Some(right))
        }
    }
}

#[derive(Debug, Clone)]
struct BitmapReader {
    base_offset: u64,
    descriptions: Box<[[u16; 2]]>,
    offsets: Option<Box<[u32]>>,
    run_container_bitmap: Option<Box<[u8]>>,
}

impl BitmapReader {
    pub fn decode<R: io::Read + io::Seek>(reader: &mut R) -> io::Result<BitmapReader> {
        let base_offset = reader.stream_position()?;

        let (size, has_offsets, has_run_containers) = {
            let cookie = reader.read_u32::<LittleEndian>()?;
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (reader.read_u32::<LittleEndian>()? as usize, true, false)
            } else if (cookie as u16) == SERIAL_COOKIE {
                let size = (cookie >> 16) as usize + 1;
                (size, size >= NO_OFFSET_THRESHOLD, true)
            } else {
                return Err(io::Error::other("unknown cookie value"));
            }
        };

        if size > u16::MAX as usize + 1 {
            return Err(io::Error::other("size is greater than supported"));
        }

        let run_container_bitmap = if has_run_containers {
            let mut bitmap = vec![0u8; size.div_ceil(8)].into_boxed_slice();
            reader.read_exact(&mut bitmap)?;
            Some(bitmap)
        } else {
            None
        };

        let mut descriptions = vec![[0; 2]; size].into_boxed_slice();
        reader.read_u16_into::<LittleEndian>(cast_slice_mut(descriptions.as_mut()))?;

        let offsets = if has_offsets {
            let mut offsets = vec![0u32; size].into_boxed_slice();
            reader.read_u32_into::<LittleEndian>(offsets.as_mut())?;
            Some(offsets)
        } else {
            None
        };

        Ok(BitmapReader { base_offset, descriptions, offsets, run_container_bitmap })
    }

    pub fn is_run_container(&self, index: usize) -> bool {
        self.run_container_bitmap.as_ref().is_some_and(|bm| bm[index / 8] & (1 << (index % 8)) != 0)
    }
}

#[cfg(test)]
mod test {
    use crate::RoaringBitmap;
    use proptest::prelude::*;
    use std::io::Cursor;

    // fast count tests
    proptest! {
        #[test]
        fn intersection_with_serialized_eq_materialized_intersection(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            let mut serialized_bytes_b = Vec::new();
            b.serialize_into(&mut serialized_bytes_b).unwrap();
            let serialized_bytes_b = &serialized_bytes_b[..];

            prop_assert_eq!(a.intersection_with_serialized_unchecked(Cursor::new(serialized_bytes_b)).unwrap(), a & b);
        }
    }

    proptest! {
        #[test]
        fn union_with_serialized_eq_materialized_intersection(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            let mut serialized_bytes_b = Vec::new();
            b.serialize_into(&mut serialized_bytes_b).unwrap();
            let serialized_bytes_b = &serialized_bytes_b[..];

            prop_assert_eq!(a.union_with_serialized_unchecked(Cursor::new(serialized_bytes_b)).unwrap(), a | b);
        }
    }
}
