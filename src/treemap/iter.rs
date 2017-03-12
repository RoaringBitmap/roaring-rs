use std::collections::BTreeMap;
use std::collections::btree_map;
use std::iter::{self, FromIterator};

use bitmap::Iter as Iter32;
use bitmap::IntoIter as IntoIter32;
use super::util;
use RoaringBitmap;
use RoaringTreemap;

struct To64Iter<'a> {
    hi: u32,
    inner: Iter32<'a>,
}

impl<'a> Iterator for To64Iter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|n| util::join(self.hi, n))
    }
}

fn to64iter<'a>(t: (&'a u32, &'a RoaringBitmap)) -> To64Iter<'a> {
    To64Iter {
        hi: *t.0,
        inner: t.1.iter(),
    }
}

struct To64IntoIter {
    hi: u32,
    inner: IntoIter32,
}

impl Iterator for To64IntoIter {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|n| util::join(self.hi, n))
    }
}

fn to64intoiter(t: (u32, RoaringBitmap)) -> To64IntoIter {
    To64IntoIter {
        hi: t.0,
        inner: t.1.into_iter(),
    }
}

type InnerIter<'a> = iter::FlatMap<btree_map::Iter<'a, u32, RoaringBitmap>,
                                   To64Iter<'a>,
                                   fn((&'a u32, &'a RoaringBitmap)) -> To64Iter<'a>>;
type InnerIntoIter = iter::FlatMap<btree_map::IntoIter<u32, RoaringBitmap>,
                                   To64IntoIter,
                                   fn((u32, RoaringBitmap)) -> To64IntoIter>;

/// An iterator for `RoaringTreemap`.
pub struct Iter<'a> {
    inner: InnerIter<'a>,
    size_hint: u64,
}

/// An iterator for `RoaringTreemap`.
pub struct IntoIter {
    inner: InnerIntoIter,
    size_hint: u64,
}

impl<'a> Iter<'a> {
    fn new(map: &BTreeMap<u32, RoaringBitmap>) -> Iter {
        let size_hint: u64 = map.iter().map(|(_, r)| r.len()).sum();
        let i = map.iter()
            .flat_map(to64iter as _);
        Iter {
            inner: i,
            size_hint: size_hint,
        }

    }
}

impl IntoIter {
    fn new(map: BTreeMap<u32, RoaringBitmap>) -> IntoIter {
        let size_hint = map.iter().map(|(_, r)| r.len()).sum();
        let i = map.into_iter()
            .flat_map(to64intoiter as _);
        IntoIter {
            inner: i,
            size_hint: size_hint,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size_hint.saturating_sub(1);
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

impl Iterator for IntoIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size_hint.saturating_sub(1);
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

impl RoaringTreemap {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by
    /// value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    /// use std::iter::FromIterator;
    ///
    /// let bitmap = RoaringTreemap::from_iter(1..3);
    /// let mut iter = bitmap.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter {
        Iter::new(&self.map)
    }
}

impl<'a> IntoIterator for &'a RoaringTreemap {
    type Item = u64;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl IntoIterator for RoaringTreemap {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(self.map)
    }
}

impl FromIterator<u64> for RoaringTreemap {
    fn from_iter<I: IntoIterator<Item = u64>>(iterator: I) -> RoaringTreemap {
        let mut rb = RoaringTreemap::new();
        rb.extend(iterator);
        rb
    }
}

impl Extend<u64> for RoaringTreemap {
    fn extend<I: IntoIterator<Item = u64>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}
