use std::num::Int;
use std::fmt::{ Show, Formatter, Result };

use util::ExtInt;
use store::Store;
use store::Store::{ Array, Bitmap };

#[derive(PartialEq, Clone)]
pub struct Container<Size: ExtInt> {
    key: Size,
    len: Size,
    store: Store<Size>,
}

impl<Size: ExtInt> Container<Size> {
    pub fn new(key: Size) -> Container<Size> {
        Container {
            key: key,
            len: Int::max_value(),
            store: Array(Vec::new()),
        }
    }
}

impl<Size: ExtInt> Container<Size> {
    #[inline]
    pub fn key(&self) -> Size { self.key }

    #[inline]
    pub fn len(&self) -> Size { self.len + Int::one() }

    #[inline]
    pub fn insert(&mut self, index: Size) -> bool {
        if self.store.insert(index) {
            self.len = self.len + Int::one();
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn remove(&mut self, index: Size) -> bool {
        if self.store.remove(index) {
            self.len = self.len - Int::one();
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn contains(&self, index: Size) -> bool {
        self.store.contains(index)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = Size> + 'a> {
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
        self.len = self.store.len() - Int::one();
        self.ensure_correct_store();
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &Self) {
        self.store.intersect_with(&other.store);
        self.len = self.store.len() - Int::one();
        self.ensure_correct_store();
    }

    #[inline]
    pub fn difference_with(&mut self, other: &Self) {
        self.store.difference_with(&other.store);
        self.len = self.store.len() - Int::one();
        self.ensure_correct_store();
    }

    #[inline]
    pub fn symmetric_difference_with(&mut self, other: &Self) {
        self.store.symmetric_difference_with(&other.store);
        self.len = self.store.len() - Int::one();
        self.ensure_correct_store();
    }

    #[inline]
    pub fn min(&self) -> Size {
        self.store.min()
    }

    #[inline]
    pub fn max(&self) -> Size {
        self.store.max()
    }

    #[inline]
    fn ensure_correct_store(&mut self) {
        let one: Size = Int::one();
        let limit = one.rotate_right(4);
        let new_store = match (&self.store, self.len) {
            (store @ &Bitmap(..), len) if len < limit => Some(store.to_array()),
            (store @ &Array(..), len) if len >= limit => Some(store.to_bitmap()),
            _ => None,
        };
        if let Some(new_store) = new_store {
            self.store = new_store;
        }
    }
}

impl<Size: ExtInt + Show> Show for Container<Size> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        format!("Container<{:?} @ {:?}>", self.len(), self.key()).fmt(formatter)
    }
}
