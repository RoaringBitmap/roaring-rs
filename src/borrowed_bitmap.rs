use std::borrow::Borrow;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::convert::identity;
use std::io::{self, Read, Error, ErrorKind};
use std::ops::{Deref, DerefMut};
use std::{slice, vec, iter, mem};

use byteorder::{ReadBytesExt, NativeEndian, LittleEndian};
use bytemuck::{pod_collect_to_vec, bytes_of_mut};
use once_cell::sync::OnceCell;

use crate::retain_mut;
use self::Store::{Array, Bitmap};

const ARRAY_LIMIT: u64 = 4096;
const BITMAP_LENGTH: usize = 1024;

const SERIAL_COOKIE: u16 = 12347;
const SERIAL_COOKIE_NO_RUNCONTAINER: u32 = 12346;

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

#[derive(PartialEq, Clone)]
pub struct BorrowedRoaringBitmap<'a> {
    containers: Vec<Container<'a>>,
}

impl<'a, 'c> BorrowedRoaringBitmap<'c> {
    pub fn union_with(&mut self, other: &BorrowedRoaringBitmap<'c>) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => self.containers[loc].union_with(container),
            }
        }
    }

    pub fn intersect_with(&mut self, other: &BorrowedRoaringBitmap<'c>) {
        retain_mut(&mut self.containers, |cont| {
            match other.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    cont.intersect_with(&other.containers[loc]);
                    cont.len != 0
                }
                Err(_) => false,
            }
        })
    }

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
                Store::Array(LazyArray::uninit(left))
            } else {
                let (left, right) = slice.split_at(1024 * mem::size_of::<u64>() as usize);
                slice = right;
                Store::Bitmap(LazyBitmap::uninit(left))
            };

            containers.push(Container { key, len, store });
        }

        Ok(BorrowedRoaringBitmap { containers })
    }

    pub fn iter(&'a self) -> Iter<'a, 'c> {
        Iter::new(&self.containers)
    }
}

#[derive(PartialEq, Clone)]
pub struct Container<'a> {
    pub key: u16,
    pub len: u64,
    pub store: Store<'a>,
}

impl Container<'_> {
    pub fn union_with(&mut self, other: &Self) {
        self.store.union_with(&other.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }

    pub fn intersect_with(&mut self, other: &Self) {
        self.store.intersect_with(&other.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }

    fn ensure_correct_store(&mut self) {
        let new_store = match (&self.store, self.len) {
            (store @ &Store::Bitmap(..), len) if len <= ARRAY_LIMIT => Some(store.to_array()),
            (store @ &Store::Array(..), len) if len > ARRAY_LIMIT => Some(store.to_bitmap()),
            _ => None,
        };
        if let Some(new_store) = new_store {
            self.store = new_store;
        }
    }
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

#[derive(Clone)]
pub struct LazyArray<'a> {
    data: OnceCell<Vec<u16>>,
    slice: &'a [u8],
}

impl LazyArray<'_> {
    fn uninit(slice: &[u8]) -> LazyArray {
        LazyArray { data: OnceCell::new(), slice }
    }

    fn init(data: Vec<u16>) -> LazyArray<'static> {
        LazyArray { data: OnceCell::from(data), slice: &[] }
    }
}

impl Deref for LazyArray<'_> {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        self.data.get_or_init(|| pod_collect_to_vec(self.slice))
    }
}

impl DerefMut for LazyArray<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.get_or_init(|| pod_collect_to_vec(self.slice));
        self.data.get_mut().unwrap()
    }
}

#[derive(Clone)]
pub struct LazyBitmap<'a> {
    data: OnceCell<Box<[u64; BITMAP_LENGTH]>>,
    slice: &'a [u8],
}

impl LazyBitmap<'_> {
    fn uninit(slice: &[u8]) -> LazyBitmap {
        LazyBitmap { data: OnceCell::new(), slice }
    }

    fn init(data: Box<[u64; BITMAP_LENGTH]>) -> LazyBitmap<'static> {
        LazyBitmap { data: OnceCell::from(data), slice: &[] }
    }
}

impl Deref for LazyBitmap<'_> {
    type Target = Box<[u64; BITMAP_LENGTH]>;

    fn deref(&self) -> &Self::Target {
        self.data.get_or_init(|| {
            let mut array = [0u64; 1024];
            bytes_of_mut(&mut array).copy_from_slice(self.slice);
            Box::new(array)
        })
    }
}

impl DerefMut for LazyBitmap<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.get_or_init(|| {
            let mut array = [0u64; 1024];
            bytes_of_mut(&mut array).copy_from_slice(self.slice);
            Box::new(array)
        });
        self.data.get_mut().unwrap()
    }
}

pub enum Store<'a> {
    Array(LazyArray<'a>),
    Bitmap(LazyBitmap<'a>),
}

impl Store<'_> {
    pub fn len(&self) -> u64 {
        match *self {
            Array(ref vec) => vec.len() as u64,
            Bitmap(ref bits) => bits.iter().map(|bit| u64::from(bit.count_ones())).sum(),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => vec.binary_search(&index).is_ok(),
            Bitmap(ref bits) => bits[key(index)] & (1 << bit(index)) != 0,
        }
    }

    pub fn insert(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => vec
                .binary_search(&index)
                .map_err(|loc| vec.insert(loc, index))
                .is_err(),
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) == 0 {
                    bits[key] |= 1 << bit;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0;
                let mut iter2 = vec2.iter();
                'outer: for &index2 in &mut iter2 {
                    while i1 < vec1.len() {
                        match vec1[i1].cmp(&index2) {
                            Less => i1 += 1,
                            Greater => vec1.insert(i1, index2),
                            Equal => continue 'outer,
                        }
                    }
                    vec1.push(index2);
                    break;
                }
                vec1.extend(iter2);
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec)) => {
                for &index in vec.deref() {
                    this.insert(index);
                }
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 |= index2;
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            }
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i = 0;
                vec1.retain(|x| {
                    i += vec2
                        .iter()
                        .skip(i)
                        .position(|y| y >= x)
                        .unwrap_or(vec2.len());
                    vec2.get(i).map_or(false, |y| x == y)
                });
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= index2;
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                vec.retain(|x| store.contains(*x));
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = other.clone();
                new.intersect_with(this);
                *this = new;
            }
        }
    }

    pub fn to_array(&self) -> Self {
        match self {
            Array(..) => panic!("Cannot convert array to array"),
            Bitmap(bits) => {
                let mut vec = Vec::new();
                for (index, mut bit) in bits.iter().cloned().enumerate() {
                    while bit != 0 {
                        vec.push((u64::trailing_zeros(bit) + (64 * index as u32)) as u16);
                        bit &= bit - 1;
                    }
                }
                Array(LazyArray::init(vec))
            }
        }
    }

    pub fn to_bitmap(&self) -> Self {
        match self {
            Array(vec) => {
                let mut bits = Box::new([0; BITMAP_LENGTH]);
                for &index in vec.deref() {
                    bits[key(index)] |= 1 << bit(index);
                }
                Bitmap(LazyBitmap::init(bits))
            }
            Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
        }
    }
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
            (Array(vec1), Array(vec2)) => vec1.deref() == vec2.deref(),
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            _ => false,
        }
    }
}

impl Clone for Store<'_> {
    fn clone(&self) -> Self {
        match self {
            Array(vec) => Array(vec.clone()),
            Bitmap(bits) => Bitmap(bits.clone()),
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

pub struct Iter<'a, 'c> {
    inner: iter::FlatMap<
        slice::Iter<'a, Container<'c>>,
        &'a Container<'c>,
        fn(&'a Container<'c>) -> &'a Container<'c>,
    >,
    size_hint: u64,
}

impl<'a, 'c> Iter<'a, 'c> {
    fn new(containers: &'a [Container<'c>]) -> Iter<'a, 'c> {
        let size_hint = containers.iter().map(|c| c.len).sum();
        Iter {
            inner: containers.iter().flat_map(identity as _),
            size_hint,
        }
    }
}

impl<'a, 'c> Iterator for Iter<'a, 'c> {
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
    fn deserialize_big() {
        let bitmap = crate::RoaringBitmap::from_iter(0..=468509);
        let mut buffer = vec![];
        bitmap.serialize_into(&mut buffer).unwrap();
        let borrowed_bitmap = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let iter = bitmap.into_iter();
        let borrowed_iter = borrowed_bitmap.iter();

        assert!(iter.eq(borrowed_iter));
    }

    #[test]
    fn deserialize_small() {
        let bitmap = crate::RoaringBitmap::from_iter(0..=40);
        let mut buffer = vec![];
        bitmap.serialize_into(&mut buffer).unwrap();
        let borrowed_bitmap = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let iter = bitmap.into_iter();
        let borrowed_iter = borrowed_bitmap.iter();

        assert!(iter.eq(borrowed_iter));
    }

    #[test]
    fn deserialize_big_intersect() {
        let mut bitmap_a = crate::RoaringBitmap::from_iter(0..=468509);
        let mut buffer = vec![];
        bitmap_a.serialize_into(&mut buffer).unwrap();
        let mut borrowed_a = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let bitmap_b = crate::RoaringBitmap::from_iter(34567..=543790);
        let mut buffer = vec![];
        bitmap_b.serialize_into(&mut buffer).unwrap();
        let borrowed_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        bitmap_a.intersect_with(&bitmap_b);
        borrowed_a.intersect_with(&borrowed_b);
        assert!(bitmap_a.iter().eq(borrowed_a.iter()));
    }

    #[test]
    fn deserialize_small_intersect() {
        let mut bitmap_a = crate::RoaringBitmap::from_iter(0..=40);
        let mut buffer = vec![];
        bitmap_a.serialize_into(&mut buffer).unwrap();
        let mut borrowed_a = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let bitmap_b = crate::RoaringBitmap::from_iter(7..=23);
        let mut buffer = vec![];
        bitmap_b.serialize_into(&mut buffer).unwrap();
        let borrowed_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        bitmap_a.intersect_with(&bitmap_b);
        borrowed_a.intersect_with(&borrowed_b);
        assert!(bitmap_a.iter().eq(borrowed_a.iter()));
    }

    #[test]
    fn deserialize_big_union() {
        let mut bitmap_a = crate::RoaringBitmap::from_iter(0..=468509);
        let mut buffer = vec![];
        bitmap_a.serialize_into(&mut buffer).unwrap();
        let mut borrowed_a = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let bitmap_b = crate::RoaringBitmap::from_iter(34567..=543790);
        let mut buffer = vec![];
        bitmap_b.serialize_into(&mut buffer).unwrap();
        let borrowed_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        bitmap_a.union_with(&bitmap_b);
        borrowed_a.union_with(&borrowed_b);
        assert!(bitmap_a.iter().eq(borrowed_a.iter()));
    }

    #[test]
    fn deserialize_small_union() {
        let mut bitmap_a = crate::RoaringBitmap::from_iter(0..=40);
        let mut buffer = vec![];
        bitmap_a.serialize_into(&mut buffer).unwrap();
        let mut borrowed_a = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        let bitmap_b = crate::RoaringBitmap::from_iter(7..=23);
        let mut buffer = vec![];
        bitmap_b.serialize_into(&mut buffer).unwrap();
        let borrowed_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer).unwrap();

        bitmap_a.union_with(&bitmap_b);
        borrowed_a.union_with(&borrowed_b);
        assert!(bitmap_a.iter().eq(borrowed_a.iter()));
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
