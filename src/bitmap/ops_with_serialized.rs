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
        mut other: R,
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
            let cookie = other.read_u32::<LittleEndian>()?;
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (other.read_u32::<LittleEndian>()? as usize, true, false)
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
            other.read_exact(&mut bitmap)?;
            Some(bitmap)
        } else {
            None
        };

        if size > u16::MAX as usize + 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "size is greater than supported"));
        }

        // Read the container descriptions
        let mut description_bytes = vec![0u8; size * DESCRIPTION_BYTES];
        other.read_exact(&mut description_bytes)?;
        let mut description_bytes: Vec<[u16; 2]> = pod_collect_to_vec(&description_bytes);
        description_bytes.iter_mut().for_each(|[ref mut k, ref mut c]| {
            *k = u16::from_le(*k);
            *c = u16::from_le(*c);
        });

        if has_offsets {
            let mut offsets = vec![0u8; size * OFFSET_BYTES];
            other.read_exact(&mut offsets)?;
            drop(offsets); // Not useful when deserializing into memory
        }

        let mut containers = Vec::with_capacity(size);
        for container in &self.containers {
            let (i, key, cardinality) =
                match description_bytes.binary_search_by_key(&container.key, |[k, _]| *k) {
                    Ok(index) => {
                        let [key, cardinality] = description_bytes[index];
                        (index, key, u64::from(cardinality) + 1)
                    }
                    Err(_) => continue,
                };

            // If the run container bitmap is present, check if this container is a run container
            let is_run_container =
                run_container_bitmap.as_ref().map_or(false, |bm| bm[i / 8] & (1 << (i % 8)) != 0);

            let mut store = if is_run_container {
                todo!("support run containers")
            } else if cardinality <= ARRAY_LIMIT {
                let mut values = vec![0; cardinality as usize];
                other.read_exact(cast_slice_mut(&mut values))?;
                values.iter_mut().for_each(|n| *n = u16::from_le(*n));
                let array = a(values).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Array(array)
            } else {
                let mut values = Box::new([0; BITMAP_LENGTH]);
                other.read_exact(cast_slice_mut(&mut values[..]))?;
                values.iter_mut().for_each(|n| *n = u64::from_le(*n));
                let bitmap = b(cardinality, values)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Store::Bitmap(bitmap)
            };

            store &= &container.store;

            containers.push(Container { key, store });
        }

        Ok(RoaringBitmap { containers })
    }
}
