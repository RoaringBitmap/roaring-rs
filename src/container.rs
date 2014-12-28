use std::{ u32 };
use std::slice::Iter;

use store::Store;
use store::Store::{ Array, Bitmap };

pub struct Container {
    key: u16,
    len: u16,
    store: Store,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container {
            key: key,
            len: 0,
            store: Array(Vec::new()),
        }
    }
}

impl Container {
    #[inline]
    pub fn key(&self) -> u16 { self.key }

    #[inline]
    pub fn len(&self) -> u16 { self.len }

    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.len += 1;
            if self.len == 4097 {
                self.store = self.store.to_bitmap();
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.len -= 1;
            if self.len == 4096 {
                self.store = self.store.to_array();
            }
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn contains(&self, index: u16) -> bool {
        self.store.contains(index)
    }

    pub fn iter<'a>(&'a self) -> ContainerIter<'a> {
        match self.store {
            Array(ref vec) => ContainerIter::ArrayIter(vec.iter()),
            Bitmap(ref bits) => ContainerIter::BitmapIter(BitmapIter::new(bits)),
        }
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.store.is_disjoint(&other.store)
    }
}

pub enum ContainerIter<'a> {
    ArrayIter(Iter<'a, u16>),
    BitmapIter(BitmapIter<'a>),
}

pub struct BitmapIter<'a> {
    key: uint,
    bit: uint,
    bits: &'a [u32, ..2048],
}

impl<'a> BitmapIter<'a> {
    fn new(bits: &'a [u32, ..2048]) -> BitmapIter<'a> {
        BitmapIter {
            key: 0,
            bit: 0,
            bits: bits,
        }
    }
}

impl<'a> Iterator<u16> for BitmapIter<'a> {
    fn next(&mut self) -> Option<u16> {
        loop {
            if self.key == 2049 {
                break;
            }
            self.bit += 1;
            if self.bit == u32::BITS {
                self.bit = 0;
                self.key += 1;
            }
            if self.key == 2048 {
                break;
            }
            if (self.bits[self.key] & (1 << self.bit)) != 0 {
                break;
            }
        }
        if self.key == 2049 {
            None
        } else {
            Some((self.key * u32::BITS + self.bit) as u16)
        }
    }
}
