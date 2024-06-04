use bytemuck::{cast_slice_mut, pod_collect_to_vec};
use byteorder::{LittleEndian, ReadBytesExt};
use core::convert::Infallible;
use core::ops::{BitAndAssign, RangeInclusive};
use std::error::Error;
use std::io;

use crate::bitmap::container::Container;
use crate::bitmap::serialization::{
    DESCRIPTION_BYTES, NO_OFFSET_THRESHOLD, OFFSET_BYTES, SERIAL_COOKIE,
    SERIAL_COOKIE_NO_RUNCONTAINER,
};
use crate::RoaringBitmap;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::container::ARRAY_LIMIT;
use super::store::{ArrayStore, BitmapStore, Store, BITMAP_LENGTH};

impl RoaringBitmap {
    /// Computes the len of the intersection with the specified other bitmap without creating a
    /// new bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.intersection_len(&rb2), (rb1 & rb2).len());
    /// ```
    // TODO convert this into a trait
    pub fn intersection_with_serialized_unchecked<R>(&self, other: R) -> io::Result<RoaringBitmap>
    where
        R: io::Read,
    {
        RoaringBitmap::intersection_with_serialized_impl::<R, _, Infallible, _, Infallible>(
            self,
            other,
            |values| Ok(ArrayStore::from_vec_unchecked(values)),
            |len, values| Ok(BitmapStore::from_unchecked(len, values)),
        )
    }

    fn intersection_with_serialized_impl<R, A, AErr, B, BErr>(
        &self,
        mut reader: R,
        a: A,
        b: B,
    ) -> io::Result<RoaringBitmap>
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

        if size > u16::MAX as usize + 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "size is greater than supported"));
        }

        // Read the container descriptions
        let mut description_bytes = vec![0u8; size * DESCRIPTION_BYTES];
        reader.read_exact(&mut description_bytes)?;
        let mut description_bytes = &description_bytes[..];

        if has_offsets {
            let mut offsets = vec![0u8; size * OFFSET_BYTES];
            reader.read_exact(&mut offsets)?;
            drop(offsets); // We could use these offsets but we are lazy
        }

        let mut containers = Vec::with_capacity(size);

        // Read each container
        for i in 0..size {
            let key = description_bytes.read_u16::<LittleEndian>()?;
            let container = match self.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(index) => self.containers.get(index),
                Err(_) => None,
            };
            let cardinality = u64::from(description_bytes.read_u16::<LittleEndian>()?) + 1;

            // If the run container bitmap is present, check if this container is a run container
            let is_run_container =
                run_container_bitmap.as_ref().map_or(false, |bm| bm[i / 8] & (1 << (i % 8)) != 0);

            let mut store = if is_run_container {
                let runs = reader.read_u16::<LittleEndian>()?;
                let mut intervals = vec![[0, 0]; runs as usize];
                reader.read_exact(cast_slice_mut(&mut intervals))?;
                if container.is_none() {
                    continue;
                }
                intervals.iter_mut().for_each(|[s, len]| {
                    *s = u16::from_le(*s);
                    *len = u16::from_le(*len);
                });

                let cardinality = intervals.iter().map(|[_, len]| *len as usize).sum();
                let mut store = Store::with_capacity(cardinality);
                intervals.into_iter().try_for_each(|[s, len]| -> Result<(), io::ErrorKind> {
                    let end = s.checked_add(len).ok_or(io::ErrorKind::InvalidData)?;
                    store.insert_range(RangeInclusive::new(s, end));
                    Ok(())
                })?;
                store
            } else if cardinality <= ARRAY_LIMIT {
                let mut values = vec![0; cardinality as usize];
                reader.read_exact(cast_slice_mut(&mut values))?;
                if container.is_none() {
                    continue;
                }
                values.iter_mut().for_each(|n| *n = u16::from_le(*n));
                let array = a(values).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Array(array)
            } else {
                let mut values = Box::new([0; BITMAP_LENGTH]);
                reader.read_exact(cast_slice_mut(&mut values[..]))?;
                if container.is_none() {
                    continue;
                }
                values.iter_mut().for_each(|n| *n = u64::from_le(*n));
                let bitmap = b(cardinality, values)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Bitmap(bitmap)
            };

            if let Some(container) = container {
                store &= &container.store;
                containers.push(Container { key, store });
            }
        }

        Ok(RoaringBitmap { containers })
    }
}
