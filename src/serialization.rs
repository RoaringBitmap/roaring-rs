use std::io;
use byteorder::{ LittleEndian, ReadBytesExt, WriteBytesExt };

use RoaringBitmap as RB;
use store::Store;
use container::Container;

const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;
const SERIAL_COOKIE: u16 = 12347;
const NO_OFFSET_THRESHOLD: u8 = 4;

pub fn serialize_into<W: io::Write>(this: &RB<u32>, mut writer: W) -> io::Result<()> {
    try!(writer.write_u32::<LittleEndian>(SERIAL_COOKIE_NO_RUNCONTAINER));
    try!(writer.write_u32::<LittleEndian>(this.containers.len() as u32));

    for container in &this.containers {
        try!(writer.write_u16::<LittleEndian>(container.key()));
        try!(writer.write_u16::<LittleEndian>((container.len() - 1) as u16));
    }

    let mut offset = 8 + 8 * this.containers.len() as u32;
    for container in &this.containers {
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

    for container in &this.containers {
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

pub fn deserialize_from<R: io::Read>(mut reader: R) -> io::Result<RB<u32>> {
    let size;
    let has_offsets;

    let cookie = try!(reader.read_u32::<LittleEndian>());
    if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
        size = try!(reader.read_u32::<LittleEndian>()) as usize;
        has_offsets = true;
    } else if (cookie as u16) == SERIAL_COOKIE {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "run containers are unsupported"));
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "unknown cookie value"));
    }

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

        let store;
        if len < 4096 {
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(try!(reader.read_u16::<LittleEndian>()));
            }
            store = Store::Array(values);
        } else {
            let mut values = Box::new([0; 1024]);
            for value in values.iter_mut() {
                *value = try!(reader.read_u64::<LittleEndian>());
            }
            store = Store::Bitmap(values);
        }

        containers.push(Container { key: key, len: len as u64, store: store });
    }

    Ok(RB { containers: containers })
}

