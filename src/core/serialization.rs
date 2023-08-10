use super::{
    container::{Container, ARRAY_LIMIT},
    store::{ArrayStore, BitmapStore, Store, BITMAP_LENGTH},
};
use crate::{ContainerKey, RoaringBitmap, Value};
use bytemuck::cast_slice_mut;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    convert::{Infallible, TryFrom},
    error::Error,
    io, mem,
    ops::RangeInclusive,
};

const COOKIE_HEADER_SIZE: usize = 8; // In bytes.
const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;
const SERIAL_COOKIE: u16 = 12347;
const NO_OFFSET_THRESHOLD: usize = 4;

impl<V: Value> RoaringBitmap<V> {
    /// Return the size in bytes of the serialized output.
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::Roaring32;
    ///
    /// let rb1: Roaring32 = (1..4).collect();
    /// let mut bytes = Vec::with_capacity(rb1.serialized_size());
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = Roaring32::deserialize_from(&bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialized_size(&self) -> usize {
        let container_sizes: usize = self
            .containers
            .iter()
            .map(|container| {
                // Descriptive header: key + cardinality.
                let key_size = V::Key::size();
                let card_size = mem::size_of::<u16>();
                // Offset header.
                let offset_size = mem::size_of::<u32>();

                key_size
                    + card_size
                    + offset_size
                    + match container.store {
                        Store::Array(ref values) => values.len() as usize * mem::size_of::<u16>(),
                        Store::Bitmap(..) => 1024 * mem::size_of::<u64>(),
                    }
            })
            .sum();

        // Cookie header + container sizes
        COOKIE_HEADER_SIZE + container_sizes
    }

    /// Serialize this bitmap into [the standard Roaring on-disk format][format].
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::Roaring32;
    ///
    /// let rb1: Roaring32 = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = Roaring32::deserialize_from(&bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialize_into<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u32::<LittleEndian>(SERIAL_COOKIE_NO_RUNCONTAINER)?;
        writer.write_u32::<LittleEndian>(self.containers.len() as u32)?;

        for container in &self.containers {
            container.key.write(&mut writer)?;
            writer.write_u16::<LittleEndian>((container.len() - 1) as u16)?;
        }

        // Descriptive header: key + cardinality.
        let key_size = V::Key::size();
        let card_size = mem::size_of::<u16>();
        // Offset header.
        let offset_size = mem::size_of::<u32>();

        let mut offset =
            COOKIE_HEADER_SIZE + (key_size + card_size + offset_size) * self.containers.len();
        for container in &self.containers {
            writer.write_u32::<LittleEndian>(offset as u32)?;
            match container.store {
                Store::Array(ref values) => {
                    offset += values.len() as usize * mem::size_of::<u16>();
                }
                Store::Bitmap(..) => {
                    offset += 1024 * mem::size_of::<u64>();
                }
            }
        }

        for container in &self.containers {
            match container.store {
                Store::Array(ref values) => {
                    for &value in values.iter() {
                        writer.write_u16::<LittleEndian>(value)?;
                    }
                }
                Store::Bitmap(ref bits) => {
                    for &value in bits.as_array() {
                        writer.write_u64::<LittleEndian>(value)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Deserialize a bitmap into memory from [the standard Roaring on-disk
    /// format][format]. This is compatible with the official C/C++, Java and
    /// Go implementations. This method checks that all of the internal values
    /// are valid. If deserializing from a trusted source consider
    /// [RoaringBitmap::deserialize_unchecked_from]
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::Roaring32;
    ///
    /// let rb1: Roaring32 = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = Roaring32::deserialize_from(&bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn deserialize_from<R: io::Read>(reader: R) -> io::Result<RoaringBitmap<V>> {
        RoaringBitmap::deserialize_from_impl(reader, ArrayStore::try_from, BitmapStore::try_from)
    }

    /// Deserialize a bitmap into memory from [the standard Roaring on-disk
    /// format][format]. This is compatible with the official C/C++, Java and
    /// Go implementations. This method is memory safe but will not check if
    /// the data is a valid bitmap.
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::Roaring32;
    ///
    /// let rb1: Roaring32 = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = Roaring32::deserialize_unchecked_from(&bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn deserialize_unchecked_from<R: io::Read>(reader: R) -> io::Result<RoaringBitmap<V>> {
        RoaringBitmap::deserialize_from_impl::<R, _, Infallible, _, Infallible>(
            reader,
            |values| Ok(ArrayStore::from_vec_unchecked(values)),
            |len, values| Ok(BitmapStore::from_unchecked(len, values)),
        )
    }

    fn deserialize_from_impl<R, A, AErr, B, BErr>(
        mut reader: R,
        a: A,
        b: B,
    ) -> io::Result<RoaringBitmap<V>>
    where
        R: io::Read,
        A: Fn(Vec<u16>) -> Result<ArrayStore, AErr>,
        AErr: Error + Send + Sync + 'static,
        B: Fn(u64, Box<[u64; 1024]>) -> Result<BitmapStore, BErr>,
        BErr: Error + Send + Sync + 'static,
    {
        // First read the cookie to determine which version of the format we are reading
        let (size, has_offsets, has_run_containers) = {
            let cookie = reader.read_u32::<LittleEndian>()?;
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (reader.read_u32::<LittleEndian>()? as usize, true, false)
            } else if (cookie as u16) == SERIAL_COOKIE {
                let size = ((cookie >> 16) + 1) as usize;
                (size, size >= NO_OFFSET_THRESHOLD, true)
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "unknown cookie value"));
            }
        };

        // Read the run container bitmap if necessary
        let run_container_bitmap = if has_run_containers {
            let mut bitmap = vec![0u8; (size + 7) / 8];
            reader.read_exact(&mut bitmap)?;
            Some(bitmap)
        } else {
            None
        };

        if size > V::max_containers() {
            return Err(io::Error::new(io::ErrorKind::Other, "size is greater than supported"));
        }

        // Read the container descriptions
        let key_size = V::Key::size();
        let card_size = mem::size_of::<u16>();
        let mut description_bytes = vec![0u8; size * (key_size + card_size)];
        reader.read_exact(&mut description_bytes)?;
        let mut description_bytes = &description_bytes[..];

        if has_offsets {
            let mut offsets = vec![0u8; size * mem::size_of::<u32>()];
            reader.read_exact(&mut offsets)?;
            drop(offsets); // Not useful when deserializing into memory
        }

        let mut containers = Vec::with_capacity(size);

        // Read each container
        for i in 0..size {
            let key = <V::Key as ContainerKey>::read(&mut description_bytes)?;
            let cardinality = u64::from(description_bytes.read_u16::<LittleEndian>()?) + 1;

            // If the run container bitmap is present, check if this container is a run container
            let is_run_container =
                run_container_bitmap.as_ref().map_or(false, |bm| bm[i / 8] & (1 << (i % 8)) != 0);

            let store = if is_run_container {
                let runs = reader.read_u16::<LittleEndian>()?;
                let mut intervals = vec![[0, 0]; runs as usize];
                reader.read_exact(cast_slice_mut(&mut intervals))?;
                intervals.iter_mut().for_each(|[s, len]| {
                    *s = u16::from_le(*s);
                    *len = u16::from_le(*len);
                });

                let cardinality = intervals.iter().map(|[_, len]| *len as usize).sum();
                let mut store = Store::with_capacity(cardinality);
                intervals.into_iter().for_each(|[s, len]| {
                    store.insert_range(RangeInclusive::new(s, s + len));
                });
                store
            } else if cardinality <= ARRAY_LIMIT {
                let mut values = vec![0; cardinality as usize];
                reader.read_exact(cast_slice_mut(&mut values))?;
                values.iter_mut().for_each(|n| *n = u16::from_le(*n));
                let array = a(values).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Array(array)
            } else {
                let mut values = Box::new([0; BITMAP_LENGTH]);
                reader.read_exact(cast_slice_mut(&mut values[..]))?;
                values.iter_mut().for_each(|n| *n = u64::from_le(*n));
                let bitmap = b(cardinality, values)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Bitmap(bitmap)
            };

            containers.push(Container { key, store });
        }

        Ok(RoaringBitmap { containers })
    }
}

#[cfg(test)]
mod test {
    use crate::Roaring32;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serialization(bitmap in Roaring32::arbitrary()) {
            let mut buffer = Vec::new();
            bitmap.serialize_into(&mut buffer).unwrap();
            prop_assert_eq!(bitmap, Roaring32::deserialize_from(buffer.as_slice()).unwrap());
        }
    }
}
