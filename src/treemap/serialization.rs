use super::RoaringTreemap;
use crate::RoaringBitmap;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{io, mem::size_of};

impl RoaringTreemap {
    /// Return the size in bytes of the serialized output.
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let mut bytes = Vec::with_capacity(rb1.serialized_size());
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringTreemap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialized_size(&self) -> usize {
        self.map.values().fold(size_of::<u64>(), |acc, bitmap| {
            acc + size_of::<u32>() + bitmap.serialized_size()
        })
    }

    /// Serialize this bitmap.
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringTreemap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialize_into<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(self.map.len() as u64)?;

        for (key, bitmap) in &self.map {
            writer.write_u32::<LittleEndian>(*key)?;
            bitmap.serialize_into(&mut writer)?;
        }

        Ok(())
    }

    /// Deserialize a bitmap into memory.
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringTreemap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn deserialize_from<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let size = reader.read_u64::<LittleEndian>()?;

        let mut s = Self::new();

        for _ in 0..size {
            let key = reader.read_u32::<LittleEndian>()?;
            let bitmap = RoaringBitmap::deserialize_from(&mut reader)?;

            s.map.insert(key, bitmap);
        }

        Ok(s)
    }
}
