use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, SubAssign};
use std::{fmt, ops::Range};

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

    pub fn insert_range(&mut self, range: Range<u16>) -> u64 {
        // If the range is larger than the array limit, skip populating the
        // array to then have to convert it to a bitmap anyway.
        if matches!(self.store, Store::Array(_)) && range.end - range.start > ARRAY_LIMIT as u16 {
            self.store = self.store.to_bitmap()
        }

        let inserted = self.store.insert_range(range);
        self.len += inserted;
        self.ensure_correct_store();
        inserted
    }

    /// Pushes `index` at the end of the container only if `index` is the new max.
    ///
    /// Returns whether the `index` was effectively pushed.
    pub fn push(&mut self, index: u16) -> bool {
        if self.store.push(index) {
            self.len += 1;
            self.ensure_correct_store();
            true
        } else {
            false
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

    pub fn remove_range(&mut self, start: u32, end: u32) -> u64 {
        debug_assert!(start <= end);
        if start == end {
            return 0;
        }
        let result = self.store.remove_range(start, end);
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

    pub fn min(&self) -> Option<u16> {
        self.store.min()
    }

    pub fn max(&self) -> Option<u16> {
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

impl BitOr<&Container> for &Container {
    type Output = Container;

    fn bitor(self, rhs: &Container) -> Container {
        let store = BitOr::bitor(&self.store, &rhs.store);
        let mut container = Container {
            key: self.key,
            len: store.len(),
            store,
        };
        container.ensure_correct_store();
        container
    }
}

impl BitOrAssign<Container> for Container {
    fn bitor_assign(&mut self, rhs: Container) {
        BitOrAssign::bitor_assign(&mut self.store, rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl BitOrAssign<&Container> for Container {
    fn bitor_assign(&mut self, rhs: &Container) {
        BitOrAssign::bitor_assign(&mut self.store, &rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl BitAnd<&Container> for &Container {
    type Output = Container;

    fn bitand(self, rhs: &Container) -> Container {
        let store = BitAnd::bitand(&self.store, &rhs.store);
        let mut container = Container {
            key: self.key,
            len: store.len(),
            store,
        };
        container.ensure_correct_store();
        container
    }
}

impl BitAndAssign<Container> for Container {
    fn bitand_assign(&mut self, rhs: Container) {
        BitAndAssign::bitand_assign(&mut self.store, rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl BitAndAssign<&Container> for Container {
    fn bitand_assign(&mut self, rhs: &Container) {
        BitAndAssign::bitand_assign(&mut self.store, &rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl SubAssign<&Container> for Container {
    fn sub_assign(&mut self, rhs: &Container) {
        SubAssign::sub_assign(&mut self.store, &rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl BitXor<&Container> for &Container {
    type Output = Container;

    fn bitxor(self, rhs: &Container) -> Container {
        let store = BitXor::bitxor(&self.store, &rhs.store);
        let mut container = Container {
            key: self.key,
            len: store.len(),
            store,
        };
        container.ensure_correct_store();
        container
    }
}

impl BitXorAssign<Container> for Container {
    fn bitxor_assign(&mut self, rhs: Container) {
        BitXorAssign::bitxor_assign(&mut self.store, rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
    }
}

impl BitXorAssign<&Container> for Container {
    fn bitxor_assign(&mut self, rhs: &Container) {
        BitXorAssign::bitxor_assign(&mut self.store, &rhs.store);
        self.len = self.store.len();
        self.ensure_correct_store();
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
