use std::{ u16 };
use std::fmt::{ Show, Formatter, Result };

use store::Store;
use store::Store::{ Array, Bitmap };

#[derive(PartialEq, Clone)]
pub struct Container {
    key: u16,
    len: u16,
    store: Store,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container {
            key: key,
            len: u16::MAX,
            store: Array(Vec::new()),
        }
    }
}

impl Container {
    #[inline]
    pub fn key(&self) -> u16 { self.key }

    #[inline]
    pub fn len(&self) -> u16 { self.len + 1 }

    #[inline]
    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.len += 1;
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.len -= 1;
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn contains(&self, index: u16) -> bool {
        self.store.contains(index)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = u16> + 'a> {
        self.store.iter()
    }

    #[inline]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.store.is_disjoint(&other.store)
    }

    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        if self.len > other.len {
            false
        } else {
            self.store.is_subset(&other.store)
        }
    }

    #[inline]
    pub fn union_with(&mut self, other: &Self) {
        self.store.union_with(&other.store);
        self.len = self.store.len() - 1;
        self.ensure_correct_store();
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &Self) {
        self.store.intersect_with(&other.store);
        self.len = self.store.len() - 1;
        self.ensure_correct_store();
    }

    #[inline]
    pub fn difference_with(&mut self, other: &Self) {
        self.store.difference_with(&other.store);
        self.len = self.store.len() - 1;
        self.ensure_correct_store();
    }

    #[inline]
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        self.store.symmetric_difference_with(&other.store);
        self.len = self.store.len() - 1;
        self.ensure_correct_store();
    }

    #[inline]
    pub fn min(&self) -> u16 {
        self.store.min()
    }

    #[inline]
    pub fn max(&self) -> u16 {
        self.store.max()
    }

    #[inline]
    fn ensure_correct_store(&mut self) {
        let new_store = match (&self.store, self.len) {
            (store @ &Bitmap(..), len) if len < 4096 => Some(store.to_array()),
            (store @ &Array(..), len) if len >= 4096 => Some(store.to_bitmap()),
            _ => None,
        };
        if let Some(new_store) = new_store {
            self.store = new_store;
        }
    }
}

impl Show for Container {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        format!("Container<{} @ {}>", self.len(), self.key()).fmt(formatter)
    }
}
