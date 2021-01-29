use std::borrow::{Borrow, Cow};
use std::convert::TryInto;
use std::io::{self, Read, Error, ErrorKind};
use std::{slice, vec, iter, mem};

use byteorder::{ReadBytesExt, NativeEndian, LittleEndian};

use self::Store::{Array, Bitmap};

const BITMAP_LENGTH: usize = 1024;
const SERIAL_COOKIE: u16 = 12347;
const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;

#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
}

#[derive(PartialEq, Clone)]
pub struct BorrowedRoaringBitmap<'a> {
    containers: Vec<Container<'a>>,
}

#[derive(PartialEq, Clone)]
pub struct Container<'a> {
    pub key: u16,
    pub len: u64,
    pub store: Store<'a>,
}

impl<'a> IntoIterator for &'a Container<'_> {
    type Item = u32;
    type IntoIter = ContainerIter<'a>;

    fn into_iter(self) -> ContainerIter<'a> {
        ContainerIter {
            key: self.key,
            inner: (&self.store).into_iter(),
        }
    }
}

pub struct ContainerIter<'a> {
    pub key: u16,
    inner: StoreIter<'a>,
}

impl<'a> Iterator for ContainerIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.inner.next().map(|i| join(self.key, i))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

pub enum Store<'a> {
    Array(Cow<'a, [u16]>),
    Bitmap(Cow<'a, [u64; BITMAP_LENGTH]>),
}

impl<'a> IntoIterator for &'a Store<'_> {
    type Item = u16;
    type IntoIter = StoreIter<'a>;
    fn into_iter(self) -> StoreIter<'a> {
        match *self {
            Array(ref vec) => StoreIter::Array(vec.iter()),
            Bitmap(ref bits) => StoreIter::BitmapBorrowed(BitmapIter::new(&**bits)),
        }
    }
}

impl PartialEq for Store<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => vec1 == vec2,
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            _ => false,
        }
    }
}

impl Clone for Store<'_> {
    fn clone(&self) -> Self {
        match *self {
            Array(ref vec) => Array(vec.clone()),
            Bitmap(ref bits) => Bitmap(bits.clone()),
        }
    }
}

pub enum StoreIter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<&'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<Box<[u64; BITMAP_LENGTH]>>),
}

impl<'a> Iterator for StoreIter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match *self {
            StoreIter::Array(ref mut inner) => inner.next().cloned(),
            StoreIter::Vec(ref mut inner) => inner.next(),
            StoreIter::BitmapBorrowed(ref mut inner) => inner.next(),
            StoreIter::BitmapOwned(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

pub struct BitmapIter<B: Borrow<[u64; BITMAP_LENGTH]>> {
    key: usize,
    bit: usize,
    bits: B,
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> BitmapIter<B> {
    fn new(bits: B) -> BitmapIter<B> {
        BitmapIter {
            key: 0,
            bit: 0,
            bits,
        }
    }

    fn move_next(&mut self) {
        self.bit += 1;
        if self.bit == 64 {
            self.bit = 0;
            self.key += 1;
        }
    }
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> Iterator for BitmapIter<B> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if self.key == BITMAP_LENGTH {
                return None;
            } else if (unsafe { self.bits.borrow().get_unchecked(self.key) } & (1u64 << self.bit))
                != 0
            {
                let result = Some((self.key * 64 + self.bit) as u16);
                self.move_next();
                return result;
            } else {
                self.move_next();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

impl BorrowedRoaringBitmap<'_> {
    pub fn deserialize_from_slice(mut slice: &[u8]) -> io::Result<BorrowedRoaringBitmap> {
        let (size, has_offsets) = {
            let cookie = slice.read_u32::<LittleEndian>().unwrap();
            if cookie == SERIAL_COOKIE_NO_RUNCONTAINER {
                (slice.read_u32::<LittleEndian>().unwrap() as usize, true)
            } else if (cookie as u16) == SERIAL_COOKIE {
                return Err(Error::new(ErrorKind::Other, "run containers are unsupported"));
            } else {
                return Err(Error::new(ErrorKind::Other, "unknown cookie value"));
            }
        };

        if size > u16::max_value() as usize + 1 {
            return Err(Error::new(ErrorKind::Other, "size is greater than supported"));
        }

        let (mut description_bytes, mut slice) = if slice.len() >= size * 4 {
            slice.split_at(size * 4)
        } else {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        };

        // FIXME we need to use those offsets!
        if has_offsets {
            let mut offsets = vec![0u8; size * 4];
            slice.read_exact(&mut offsets).unwrap();
            drop(offsets); // Not useful when deserializing into memory
        }

        let mut containers = Vec::with_capacity(size);

        for _ in 0..size {
            let key = description_bytes.read_u16::<LittleEndian>().unwrap();
            let len = u64::from(description_bytes.read_u16::<LittleEndian>().unwrap()) + 1;

            let store = if len <= 4096 {
                let (left, right) = slice.split_at(len as usize);
                slice = right;
                let values = match bytemuck::try_cast_slice(left) {
                    Ok(values) => Cow::Borrowed(values),
                    Err(_err) => Cow::Owned(bytemuck::pod_collect_to_vec(left)),
                };
                Store::Array(values)
            } else {
                let (left, right) = slice.split_at(1024 * mem::size_of::<u64>() as usize);
                slice = right;
                let values = match bytemuck::try_cast_slice(left) {
                    Ok(values) => Cow::Borrowed(values.try_into().unwrap()),
                    Err(_err) => {
                        let mut array = [0; 1024];
                        bytemuck::bytes_of_mut(&mut array).copy_from_slice(left);
                        Cow::Owned(array)
                    },
                };
                Store::Bitmap(values)
            };

            containers.push(Container { key, len, store });
        }

        Ok(BorrowedRoaringBitmap { containers })
    }

    pub fn iter(&self) -> Iter {
        Iter::new(&self.containers)
    }
}

pub struct Iter<'a> {
    inner: iter::FlatMap<
        slice::Iter<'a, Container<'a>>,
        &'a Container<'a>,
        fn(&'a Container<'a>) -> &'a Container<'a>,
    >,
    size_hint: u64,
}

impl<'a> Iter<'a> {
    fn new(containers: &'a [Container<'a>]) -> Iter<'a> {
        fn identity<T>(t: T) -> T {
            t
        }
        let size_hint = containers.iter().map(|c| c.len).sum();
        Iter {
            inner: containers.iter().flat_map(identity as _),
            size_hint,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::max_value() as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::max_value(), None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;
    use std::iter::FromIterator;

    use quickcheck_macros::quickcheck;

    use crate::RoaringBitmap;
    use super::*;

    #[test]
    fn deserialization() {
        let bitmap = crate::RoaringBitmap::from_iter(0..=468509);
        let mut buffer = vec![];
        bitmap.serialize_into(&mut buffer).unwrap();
        let borrowed_bitmap = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let iter = bitmap.into_iter();
        let borrowed_iter = borrowed_bitmap.iter();

        assert!(iter.eq(borrowed_iter));
    }


    #[quickcheck]
    fn qc_deserialize_range(range: Range<u32>) {
        let bitmap = RoaringBitmap::from_iter(range);
        let mut buffer = vec![];
        bitmap.serialize_into(&mut buffer).unwrap();
        let borrowed_bitmap = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let iter = bitmap.into_iter();
        let borrowed_iter = borrowed_bitmap.iter();

        assert!(iter.eq(borrowed_iter));
    }
}
