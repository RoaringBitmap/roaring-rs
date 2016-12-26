use std::io;
use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use RoaringBitmap;
use util;
use store::Store;
use container::Container;

const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;
const SERIAL_COOKIE: u16 = 12347;
// TODO: Need this once run containers are supported
// const NO_OFFSET_THRESHOLD: u8 = 4;

impl RoaringBitmap<u32> {
    /// Serialize this bitmap into [the standard Roaring on-disk format][format].
    /// This is compatible with the official C/C++, Java and Go implementations.
    ///
    /// [format]: https://github.com/RoaringBitmap/RoaringFormatSpec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use std::iter::FromIterator;
    ///
    /// let rb1 = RoaringBitmap::from_iter(1..4u32);
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringBitmap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn serialize_into<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        try!(writer.write_u32::<LittleEndian>(SERIAL_COOKIE_NO_RUNCONTAINER));
        try!(writer.write_u32::<LittleEndian>(self.containers.len() as u32));

        for container in &self.containers {
            try!(writer.write_u16::<LittleEndian>(container.key()));
            try!(writer.write_u16::<LittleEndian>((container.len() - 1) as u16));
        }

        let mut offset = 8 + 8 * self.containers.len() as u32;
        for container in &self.containers {
            try!(writer.write_u32::<LittleEndian>(offset));
            match container.store {
                Store::Array(ref values) => {
                    offset += values.len() as u32 * 2;
                }
                Store::Bitmap(..) => {
                    offset += 8 * 1024;
                }
            }
        }

        for container in &self.containers {
            match container.store {
                Store::Array(ref values) => {
                    for &value in values {
                        try!(writer.write_u16::<LittleEndian>(value));
                    }
                }
                Store::Bitmap(ref values) => {
                    for &value in values.iter() {
                        try!(writer.write_u64::<LittleEndian>(value));
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
    /// use std::iter::FromIterator;
    ///
    /// let rb1 = RoaringBitmap::from_iter(1..4u32);
    /// let mut bytes = vec![];
    /// rb1.serialize_into(&mut bytes).unwrap();
    /// let rb2 = RoaringBitmap::deserialize_from(&mut &bytes[..]).unwrap();
    ///
    /// assert_eq!(rb1, rb2);
    /// ```
    pub fn deserialize_from<R: io::Read>(mut reader: R) -> io::Result<RoaringBitmap<u32>> {
        let (size, has_offsets) = {
            let cookie = try!(reader.read_u32::<LittleEndian>());
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (try!(reader.read_u32::<LittleEndian>()) as usize, true)
            } else if (cookie as u16) == SERIAL_COOKIE {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "run containers are unsupported"));
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "unknown cookie value"));
            }
        };

        if size > u16::max_value() as usize {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "size is greater than supported"));
        }

        let mut description_bytes = vec![0u8; size * 4];
        try!(reader.read_exact(&mut description_bytes));
        let description_bytes = &mut &description_bytes[..];

        if has_offsets {
            let mut offsets = vec![0u8; size * 4];
            try!(reader.read_exact(&mut offsets));
            drop(offsets); // Not useful when deserializing into memory
        }

        let mut containers = Vec::with_capacity(size);

        for _ in 0..size {
            let key = try!(description_bytes.read_u16::<LittleEndian>());
            let len = try!(description_bytes.read_u16::<LittleEndian>()) as usize + 1;

            let store = if len < 4096 {
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(try!(reader.read_u16::<LittleEndian>()));
                }
                Store::Array(values)
            } else {
                let mut values = Box::new([0; 1024]);
                for value in values.iter_mut() {
                    *value = try!(reader.read_u64::<LittleEndian>());
                }
                Store::Bitmap(values)
            };

            containers.push(Container { key: key, len: util::cast(len), store: store });
        }

        Ok(RoaringBitmap { containers: containers })
    }
}
