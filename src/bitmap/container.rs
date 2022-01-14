use std::fmt;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, Sub, SubAssign,
};

use crate::bitmap::store::{self, Store, Store::*};

use super::util;

const ARRAY_LIMIT: u64 = 4096;

#[derive(PartialEq, Clone)]
pub struct Container {
    pub key: u16,
    pub store: Store,
}

pub struct Iter<'a> {
    pub key: u16,
    inner: store::Iter<'a>,
}

impl Container {
    pub fn new(key: u16) -> Container {
        Container { key, store: Store::new() }
    }
}

impl Container {
    pub fn len(&self) -> u64 {
        self.store.len()
    }

    pub fn insert(&mut self, index: u16) -> bool {
        if self.store.insert(index) {
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let inserted = self.store.insert_range(range);
        self.ensure_correct_store();
        inserted
    }

    /// Pushes `index` at the end of the container only if `index` is the new max.
    ///
    /// Returns whether the `index` was effectively pushed.
    pub fn push(&mut self, index: u16) -> bool {
        if self.store.push(index) {
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    ///
    /// Pushes `index` at the end of the container.
    /// It is up to the caller to have validated index > self.max()
    ///
    /// # Panics
    ///
    /// If debug_assertions enabled and index is > self.max()
    pub(crate) fn push_unchecked(&mut self, index: u16) {
        self.store.push_unchecked(index);
        self.ensure_correct_store();
    }

    pub fn remove(&mut self, index: u16) -> bool {
        if self.store.remove(index) {
            self.ensure_correct_store();
            true
        } else {
            false
        }
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let result = self.store.remove_range(range);
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
        self.len() <= other.len() && self.store.is_subset(&other.store)
    }

    pub fn min(&self) -> Option<u16> {
        self.store.min()
    }

    pub fn max(&self) -> Option<u16> {
        self.store.max()
    }

    pub(crate) fn ensure_correct_store(&mut self) {
        match &self.store {
            Store::Bitmap(ref bits) => {
                if bits.len() <= ARRAY_LIMIT {
                    self.store = Store::Array(bits.to_array_store())
                }
            }
            Store::Array(ref vec) => {
                if vec.len() as u64 > ARRAY_LIMIT {
                    self.store = Store::Bitmap(vec.to_bitmap_store())
                }
            }
        };
    }
}

impl BitOr<&Container> for &Container {
    type Output = Container;

    fn bitor(self, rhs: &Container) -> Container {
        let store = match (&self.store, &rhs.store) {
            (Array(vec1), Array(vec2)) => Array(vec1 | vec2),
            (Bitmap(bits1), Array(vec2)) => {
                let mut bits1 = bits1.clone();
                bits1 |= vec2;
                Bitmap(bits1)
            }
            (Bitmap(bits1), Bitmap(bits2)) => {
                let mut bits1 = bits1.clone();
                bits1 |= bits2;
                Bitmap(bits1)
            }
            (Array(vec1), Bitmap(bits2)) => {
                let mut bits2 = bits2.clone();
                bits2 |= vec1;
                Bitmap(bits2)
            }
        };
        let mut container = Container { key: self.key, store };
        container.ensure_correct_store();
        container
    }
}

impl BitOrAssign<Container> for Container {
    fn bitor_assign(&mut self, rhs: Container) {
        match (&mut self.store, rhs.store) {
            (Array(ref vec1), Array(ref vec2)) => {
                self.store = Array(vec1 | vec2);
            }
            (Bitmap(bits1), Array(ref vec2)) => {
                *bits1 |= vec2;
            }
            (Bitmap(bits1), Bitmap(ref bits2)) => {
                *bits1 |= bits2;
            }
            (Array(ref vec1), Bitmap(mut bits2)) => {
                bits2 |= vec1;
                self.store = Bitmap(bits2);
            }
        }
        self.ensure_correct_store();
    }
}

impl BitOrAssign<&Container> for Container {
    fn bitor_assign(&mut self, rhs: &Container) {
        match (&mut self.store, &rhs.store) {
            (Array(ref vec1), Array(vec2)) => {
                self.store = Array(vec1 | vec2);
            }
            (Bitmap(bits1), Array(vec2)) => {
                *bits1 |= vec2;
            }
            (Bitmap(bits1), Bitmap(bits2)) => {
                *bits1 |= bits2;
            }
            (Array(ref vec1), Bitmap(bits2)) => {
                let mut bits2 = bits2.clone();
                bits2 |= vec1;
                self.store = Bitmap(bits2);
            }
        }
        self.ensure_correct_store();
    }
}

impl BitAnd<&Container> for &Container {
    type Output = Container;

    fn bitand(self, rhs: &Container) -> Container {
        let store = BitAnd::bitand(&self.store, &rhs.store);
        let mut container = Container { key: self.key, store };
        container.ensure_correct_store();
        container
    }
}

impl BitAndAssign<Container> for Container {
    fn bitand_assign(&mut self, rhs: Container) {
        BitAndAssign::bitand_assign(&mut self.store, rhs.store);
        self.ensure_correct_store();
    }
}

impl BitAndAssign<&Container> for Container {
    fn bitand_assign(&mut self, rhs: &Container) {
        BitAndAssign::bitand_assign(&mut self.store, &rhs.store);
        self.ensure_correct_store();
    }
}

impl Sub<&Container> for &Container {
    type Output = Container;

    fn sub(self, rhs: &Container) -> Container {
        let store = Sub::sub(&self.store, &rhs.store);
        let mut container = Container { key: self.key, store };
        container.ensure_correct_store();
        container
    }
}

impl SubAssign<&Container> for Container {
    fn sub_assign(&mut self, rhs: &Container) {
        SubAssign::sub_assign(&mut self.store, &rhs.store);
        self.ensure_correct_store();
    }
}

impl BitXor<&Container> for &Container {
    type Output = Container;

    fn bitxor(self, rhs: &Container) -> Container {
        let store = BitXor::bitxor(&self.store, &rhs.store);
        let mut container = Container { key: self.key, store };
        container.ensure_correct_store();
        container
    }
}

impl BitXorAssign<Container> for Container {
    fn bitxor_assign(&mut self, rhs: Container) {
        BitXorAssign::bitxor_assign(&mut self.store, rhs.store);
        self.ensure_correct_store();
    }
}

impl BitXorAssign<&Container> for Container {
    fn bitxor_assign(&mut self, rhs: &Container) {
        BitXorAssign::bitxor_assign(&mut self.store, &rhs.store);
        self.ensure_correct_store();
    }
}

impl<'a> IntoIterator for &'a Container {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter { key: self.key, inner: (&self.store).into_iter() }
    }
}

impl IntoIterator for Container {
    type Item = u32;
    type IntoIter = Iter<'static>;

    fn into_iter(self) -> Iter<'static> {
        Iter { key: self.key, inner: self.store.into_iter() }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.inner.next().map(|i| util::join(self.key, i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Debug for Container {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        format!("Container<{:?} @ {:?}>", self.len(), self.key).fmt(formatter)
    }
}
