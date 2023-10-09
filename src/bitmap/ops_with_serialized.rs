use bytemuck::cast_slice_mut;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    io::{self, Read},
    mem::size_of,
};

use crate::RoaringBitmap;

use super::{
    container::Container,
    store::{ArrayStore, BitmapStore, Store},
};

const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;
const SERIAL_COOKIE: u16 = 12347;

impl RoaringBitmap {
    pub fn union_with_serialized(&mut self, mut reader: impl Read) -> io::Result<()> {
        let (size, has_offsets) = {
            let cookie = reader.read_u32::<LittleEndian>()?;
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (reader.read_u32::<LittleEndian>()? as usize, true)
            } else if (cookie as u16) == SERIAL_COOKIE {
                return Err(io::Error::new(io::ErrorKind::Other, "run containers are unsupported"));
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "unknown cookie value"));
            }
        };

        if size > u16::MAX as usize + 1 {
            return Err(io::Error::new(io::ErrorKind::Other, "size is greater than supported"));
        }

        let mut description_bytes = vec![0u8; size * 4];
        reader.read_exact(&mut description_bytes)?;
        let mut description_bytes = &description_bytes[..];

        if has_offsets {
            let mut offsets = vec![0u8; size * 4];
            reader.read_exact(&mut offsets)?;
            drop(offsets); // Not useful when deserializing into memory
        }

        for _ in 0..size {
            let key = description_bytes.read_u16::<LittleEndian>()?;
            let len = u64::from(description_bytes.read_u16::<LittleEndian>()?) + 1;

            if len <= 4096 {
                match self.containers.binary_search_by_key(&key, |c| c.key) {
                    Ok(loc) => {
                        let container = &mut self.containers[loc];

                        for _ in 0..len {
                            let mut value = [0u8; size_of::<u16>()];
                            reader.read_exact(value.as_mut())?;
                            // TODO: since this is sorted it could probably be faster
                            let value = u16::from_le_bytes(value);
                            container.insert(value);
                        }
                    }
                    Err(loc) => {
                        let mut values = vec![0u16; len as usize];
                        reader.read_exact(cast_slice_mut(&mut values))?;
                        values.iter_mut().for_each(|n| *n = u16::from_le(*n));

                        let array = ArrayStore::from_vec_unchecked(values);
                        let mut container = Container::new(key);
                        container.store = Store::Array(array);
                        self.containers.insert(loc, container);
                    }
                }
            } else {
                match self.containers.binary_search_by_key(&key, |c| c.key) {
                    Ok(loc) => {
                        let current_store = std::mem::take(&mut self.containers[loc].store);

                        let mut values = Box::new([0; 1024]);
                        reader.read_exact(cast_slice_mut(&mut values[..]))?;
                        values.iter_mut().for_each(|n| *n = u64::from_le(*n));

                        let mut store = BitmapStore::from_unchecked(len, values);

                        match current_store {
                            Store::Array(array) => array.into_iter().for_each(|el| {
                                store.insert(el);
                            }),
                            Store::Bitmap(bitmap_store) => store |= &bitmap_store,
                        };

                        self.containers[loc].store = Store::Bitmap(store);
                    }
                    Err(loc) => {
                        let mut values = Box::new([0; 1024]);
                        reader.read_exact(cast_slice_mut(&mut values[..]))?;
                        values.iter_mut().for_each(|n| *n = u64::from_le(*n));

                        let array = BitmapStore::from_unchecked(len, values);
                        let mut container = Container::new(key);
                        container.store = Store::Bitmap(array);
                        self.containers.insert(loc, container);
                    }
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::RoaringBitmap;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_or_with_serialized(
            mut a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            let union = &a | &b;

            let mut b_ser = Vec::new();
            b.serialize_into(&mut b_ser).unwrap();
            a.union_with_serialized(&*b_ser).unwrap();

            prop_assert_eq!(a, union);
        }
    }

    #[test]
    fn or_with_serialized() {
        let unions = [
            (RoaringBitmap::new(), RoaringBitmap::new()),
            (RoaringBitmap::from_sorted_iter([0]).unwrap(), RoaringBitmap::new()),
            (RoaringBitmap::new(), RoaringBitmap::from_sorted_iter([0]).unwrap()),
            (
                RoaringBitmap::from_sorted_iter([0]).unwrap(),
                RoaringBitmap::from_sorted_iter([0]).unwrap(),
            ),
            (
                RoaringBitmap::from_sorted_iter([0]).unwrap(),
                RoaringBitmap::from_sorted_iter([1]).unwrap(),
            ),
            (
                RoaringBitmap::from_sorted_iter([0]).unwrap(),
                RoaringBitmap::from_sorted_iter(0..3000).unwrap(),
            ),
            (
                RoaringBitmap::from_sorted_iter([]).unwrap(),
                RoaringBitmap::from_sorted_iter(0..3000).unwrap(),
            ),
            (
                RoaringBitmap::from_sorted_iter(0..3000).unwrap(),
                RoaringBitmap::from_sorted_iter([3001]).unwrap(),
            ),
            (
                RoaringBitmap::from_sorted_iter(0..3000).unwrap(),
                RoaringBitmap::from_sorted_iter(3000..6000).unwrap(),
            ),
        ];

        for (mut a, b) in unions {
            let union = &a | &b;

            let mut b_ser = Vec::new();
            b.serialize_into(&mut b_ser).unwrap();
            a.union_with_serialized(&*b_ser).unwrap();

            assert_eq!(a, union, "When testing: {a:?} | {b:?}");
        }
    }
}
