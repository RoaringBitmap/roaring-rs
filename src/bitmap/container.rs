use std::fmt;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, Sub, SubAssign,
};

use super::store::{self, Store};
use super::util;

const ARRAY_LIMIT: u64 = 4096;

#[derive(PartialEq, Clone)]
pub struct Container {
    pub key: u16,
    pub len: u64,
    pub store: Store,
}

pub struct Iter<'a> {
    pub key: u16,
    inner: store::Iter<'a>,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container {
            key,
            len: 0,
            store: Store::Array(Vec::new()),
        }
    }
}

impl Container {
    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.len += 1;
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let inserted = self.store.insert_range(range);
        self.len += inserted;
        self.ensure_correct_store();
        inserted
    }

    pub fn push(&mut self, index: u16) {
        if self.store.push(index) {
            self.len += 1;
            self.ensure_correct_store();
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.len -= 1;
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let result = self.store.remove_range(range);
        self.len -= result;
        self.ensure_correct_store();
        result
    }

    pub fn contains(&self, index: u16) -> bool {
        self.store.contains(index)
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.store.is_disjoint(&other.store)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.len <= other.len && self.store.is_subset(&other.store)
    }

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

    pub fn difference_with(&mut self, other: &Self) {
        self.store.difference_with(&other.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }

    pub fn symmetric_difference_with(&mut self, other: &Self) {
        self.store.symmetric_difference_with(&other.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }

    pub fn min(&self) -> u16 {
        self.store.min()
    }

    pub fn max(&self) -> u16 {
        self.store.max()
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

impl<'a> IntoIterator for &'a Container {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter {
            key: self.key,
            inner: (&self.store).into_iter(),
        }
    }
}

impl IntoIterator for Container {
    type Item = u32;
    type IntoIter = Iter<'static>;

    fn into_iter(self) -> Iter<'static> {
        Iter {
            key: self.key,
            inner: self.store.into_iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.inner.next().map(|i| util::join(self.key, i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

impl fmt::Debug for Container {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        format!("Container<{:?} @ {:?}>", self.len, self.key).fmt(formatter)
    }
}
