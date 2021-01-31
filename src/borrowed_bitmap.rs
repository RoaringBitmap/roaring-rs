use std::cmp::Ordering::{self, Equal, Greater, Less};
use std::io::{Read, Error, ErrorKind};
use std::{mem, io};

use byteorder::{ByteOrder, ReadBytesExt, LittleEndian};
use bytemuck::{bytes_of_mut, pod_collect_to_vec};

use crate::bitmap::container::Container as OwnedContainer;
use crate::bitmap::store::Store as OwnedStore;

use self::Store::{Array, Bitmap};

const ARRAY_LIMIT: u64 = 4096;
const BITMAP_LENGTH: usize = 1024;

const SERIAL_COOKIE: u16 = 12347;
const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;

const U16_SIZE: usize = mem::size_of::<u16>();
const U64_SIZE: usize = mem::size_of::<u64>();

#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
}

#[inline]
fn key(index: u16) -> usize {
    index as usize / 64
}

#[inline]
fn bit(index: u16) -> usize {
    index as usize % 64
}

#[derive(Clone)]
pub struct BorrowedRoaringBitmap<'a> {
    pub containers: Vec<Container<'a>>,
}

impl<'a> BorrowedRoaringBitmap<'a> {
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
                let (left, right) = slice.split_at(len as usize * mem::size_of::<u16>());
                slice = right;
                Store::Array(LazyArray::new(left))
            } else {
                let (left, right) = slice.split_at(1024 * mem::size_of::<u64>() as usize);
                slice = right;
                Store::Bitmap(LazyBitmap::new(left))
            };

            containers.push(Container { key, len, store });
        }

        Ok(BorrowedRoaringBitmap { containers })
    }
}

#[derive(Clone)]
pub struct Container<'a> {
    pub key: u16,
    pub len: u64,
    pub store: Store<'a>,
}

impl Container<'_> {
    // FIXME use the ToOwned trait?
    pub fn to_owned(&self) -> OwnedContainer {
        OwnedContainer {
            key: self.key,
            len: self.len,
            store: self.store.to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct LazyArray<'a> {
    bytes: &'a [u8],
}

impl LazyArray<'_> {
    fn new(bytes: &[u8]) -> LazyArray {
        LazyArray { bytes }
    }

    pub fn len(&self) -> usize {
        self.bytes.len() / U16_SIZE
    }

    pub fn binary_search(&self, index: u16) -> Result<usize, usize> {
        self.binary_search_by(|p| p.cmp(&index))
    }

    #[inline]
    pub fn binary_search_by<F>(&self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(u16) -> Ordering,
    {
        let s = self;
        let mut size = s.len();
        if size == 0 {
            return Err(0);
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // SAFETY: the call is made safe by the following inconstants:
            // - `mid >= 0`: by definition
            // - `mid < size`: `mid = size / 2 + size / 4 + size / 8 ...`
            let cmp = f(unsafe { s.get_unchecked(mid) });
            base = if cmp == Greater { base } else { mid };
            size -= half;
        }
        // SAFETY: base is always in [0, size) because base <= mid.
        let cmp = f(unsafe { s.get_unchecked(base) });
        if cmp == Equal { Ok(base) } else { Err(base + (cmp == Less) as usize) }
    }

    pub fn get(&self, index: usize) -> Option<u16> {
        let base = index * U16_SIZE;
        self.bytes.get(base..base + U16_SIZE).map(LittleEndian::read_u16)
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> u16 {
        let bytes = self.bytes.get_unchecked(index * U16_SIZE..);
        LittleEndian::read_u16(bytes)
    }

    // FIXME more efficient type for skip/nth
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u16> + 'a {
        self.bytes.chunks_exact(U16_SIZE).map(LittleEndian::read_u16)
    }
}

#[derive(Clone)]
pub struct LazyBitmap<'a> {
    bytes: &'a [u8],
}

impl LazyBitmap<'_> {
    fn new(bytes: &[u8]) -> LazyBitmap {
        LazyBitmap { bytes }
    }

    pub fn get(&self, index: usize) -> Option<u64> {
        let base = index * U64_SIZE;
        self.bytes.get(base..base + U64_SIZE).map(LittleEndian::read_u64)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u64> + 'a {
        self.bytes.chunks_exact(U64_SIZE).map(LittleEndian::read_u64)
    }
}

#[derive(Clone)]
pub enum Store<'a> {
    Array(LazyArray<'a>),
    Bitmap(LazyBitmap<'a>),
}

impl Store<'_> {
    // FIXME use the ToOwned trait?
    pub fn to_owned(&self) -> OwnedStore {
        match self {
            Array(array) => OwnedStore::Array(pod_collect_to_vec(array.bytes)),
            Bitmap(bitmap) => {
                let mut new = Box::new([0u64; BITMAP_LENGTH]);
                bytes_of_mut(&mut *new).copy_from_slice(bitmap.bytes);
                OwnedStore::Bitmap(new)
            },
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match self {
            Array(array) => array.binary_search(index).is_ok(),
            Bitmap(bitmap) => {
                let bits = bitmap.get(key(index)).unwrap();
                bits & (1 << bit(index)) != 0
            },
        }
    }
}
