use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

use super::container::Container;
use super::store::Store;
use crate::bitmap::container::ARRAY_LIMIT;
use crate::bitmap::store::{Interval, BITMAP_LENGTH};
use crate::RoaringBitmap;

const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;
const SERIAL_COOKIE: u16 = 12347;
const NO_OFFSET_THRESHOLD: usize = 4;

// Sizes of header structures
const COOKIE_BYTES: usize = 4;
const SIZE_BYTES: usize = 4;
const DESCRIPTION_BYTES: usize = 4;
const OFFSET_BYTES: usize = 4;

// Sizes of container structures
const BITMAP_BYTES: usize = BITMAP_LENGTH * 8;
const ARRAY_ELEMENT_BYTES: usize = 2;
const RUN_NUM_BYTES: usize = 2;
const RUN_ELEMENT_BYTES: usize = 4;

impl RoaringBitmap {
    /// Return the size in bytes of the serialized output.
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let mut bytes = Vec::with_capacity(rb1.serialized_size());
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringBitmap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialized_size(&self) -> usize {
        let mut has_run_containers = false;
        let size = self.containers.len();
        let container_sizes: usize = self
            .containers
            .iter()
            .map(|container| match container.store {
                Store::Array(ref values) => values.len() * ARRAY_ELEMENT_BYTES,
                Store::Bitmap(..) => BITMAP_BYTES,
                Store::Run(ref intervals) => {
                    has_run_containers = true;
                    RUN_NUM_BYTES + (RUN_ELEMENT_BYTES * intervals.len())
                }
            })
            .sum();

        // header + container sizes
        header_size(size, has_run_containers) + container_sizes
    }

    /// Serialize this bitmap into [the standard Roaring on-disk format][format].
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringBitmap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialize_into<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let has_run_containers = self.containers.iter().any(|c| {
            if let Store::Run(_) = c.store {
                true
            } else {
                false
            }
        });
        let size = self.containers.len();

        // Depending on if run containers are present or not write the appropriate header
        if has_run_containers {
            // The new format stores the container count in the most significant bits of the header
            let cookie = SERIAL_COOKIE as u32 | ((size as u32 - 1) << 16);
            writer.write_u32::<LittleEndian>(cookie)?;
            // It is then followed by a bitset indicating which containers are run containers
            let run_container_bitmap_size = (size + 7) / 8;
            let mut run_container_bitmap = vec![0; run_container_bitmap_size];
            for (i, container) in self.containers.iter().enumerate() {
                if let Store::Run(_) = container.store {
                    run_container_bitmap[i / 8] |= 1 << (i % 8);
                }
            }
            writer.write_all(&run_container_bitmap)?;
        } else {
            // Write old format, cookie followed by container count
            writer.write_u32::<LittleEndian>(SERIAL_COOKIE_NO_RUNCONTAINER)?;
            writer.write_u32::<LittleEndian>(size as u32)?;
        }

        // Write the container descriptions
        for container in &self.containers {
            writer.write_u16::<LittleEndian>(container.key)?;
            writer.write_u16::<LittleEndian>((container.len - 1) as u16)?;
        }

        // Write offsets if there are no runs or NO_OFFSET_THRESHOLD containers is reached
        if !has_run_containers || size >= NO_OFFSET_THRESHOLD {
            let mut offset = header_size(size, has_run_containers) as u32;
            for container in &self.containers {
                writer.write_u32::<LittleEndian>(offset)?;
                match container.store {
                    Store::Array(ref values) => {
                        offset += (values.len() * ARRAY_ELEMENT_BYTES) as u32;
                    }
                    Store::Bitmap(..) => {
                        offset += BITMAP_BYTES as u32;
                    }
                    Store::Run(ref intervals) => {
                        offset += (RUN_NUM_BYTES + (intervals.len() * RUN_ELEMENT_BYTES)) as u32;
                    }
                }
            }
        }

        // Finally serialize each of the containers
        for container in &self.containers {
            match container.store {
                Store::Array(ref values) => {
                    for &value in values {
                        writer.write_u16::<LittleEndian>(value)?;
                    }
                }
                Store::Bitmap(ref values) => {
                    for &value in values.iter() {
                        writer.write_u64::<LittleEndian>(value)?;
                    }
                }
                Store::Run(ref intervals) => {
                    writer.write_u16::<LittleEndian>(intervals.len() as u16)?;
                    for iv in intervals {
                        writer.write_u16::<LittleEndian>(iv.start)?;
                        writer.write_u16::<LittleEndian>(iv.end - iv.start)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Deserialize a bitmap into memory from [the standard Roaring on-disk
    /// format][format].  This is compatible with the official C/C++, Java and
    /// Go implementations.
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringBitmap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn deserialize_from<R: io::Read>(mut reader: R) -> io::Result<RoaringBitmap> {
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

        if size > u16::max_value() as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "size is greater than supported",
            ));
        }

        // Read the container descriptions
        let mut description_bytes = vec![0u8; size * DESCRIPTION_BYTES];
        reader.read_exact(&mut description_bytes)?;
        let description_bytes = &mut &description_bytes[..];

        // Read the offsets if present
        if has_offsets {
            let mut offsets = vec![0u8; size * OFFSET_BYTES];
            reader.read_exact(&mut offsets)?;
            drop(offsets); // Not useful when deserializing into memory
        }

        let mut containers = Vec::with_capacity(size);

        // Read each of the containers
        for i in 0..size {
            let key = description_bytes.read_u16::<LittleEndian>()?;
            let cardinality = u64::from(description_bytes.read_u16::<LittleEndian>()?) + 1;

            // If the run container bitmap is present, check if this container is a run container
            let is_run_container = match run_container_bitmap {
                Some(ref bm) => bm[i / 8] & (1 << (i % 8)) != 0,
                None => false,
            };

            let store = if is_run_container {
                let runs = reader.read_u16::<LittleEndian>()?;
                let mut intervals = Vec::with_capacity(runs as usize);
                for _ in 0..runs {
                    let start = reader.read_u16::<LittleEndian>()?;
                    let run_len = reader.read_u16::<LittleEndian>()?;
                    let end = start + run_len;
                    intervals.push(Interval { start, end })
                }
                Store::Run(intervals)
            } else if cardinality <= ARRAY_LIMIT {
                let mut values = Vec::with_capacity(cardinality as usize);
                for _ in 0..cardinality {
                    values.push(reader.read_u16::<LittleEndian>()?);
                }
                Store::Array(values)
            } else {
                let mut values = Box::new([0; BITMAP_LENGTH]);
                for value in values.iter_mut() {
                    *value = reader.read_u64::<LittleEndian>()?;
                }
                Store::Bitmap(values)
            };

            containers.push(Container {
                key,
                len: cardinality,
                store,
            });
        }

        Ok(RoaringBitmap { containers })
    }
}

fn header_size(size: usize, has_run_containers: bool) -> usize {
    if has_run_containers {
        // New format encodes the size (number of containers) into the 4 byte cookie
        // Additionally a bitmap is included marking which containers are run containers
        let run_container_bitmap_size = (size + 7) / 8;
        // New format conditionally includes offsets if there are 4 or more containers
        if size >= NO_OFFSET_THRESHOLD {
            COOKIE_BYTES + ((DESCRIPTION_BYTES + OFFSET_BYTES) * size) + run_container_bitmap_size
        } else {
            COOKIE_BYTES + (DESCRIPTION_BYTES * size) + run_container_bitmap_size
        }
    } else {
        // Old format encodes cookie followed by container count
        // It also always includes the offsets
        COOKIE_BYTES + SIZE_BYTES + ((DESCRIPTION_BYTES + OFFSET_BYTES) * size)
    }
}
