use std::collections::BTreeMap;
use std::collections::btree_map;
use std::iter::{self, FromIterator};
use std::slice;
use std::vec;

use iter::Iter as Iter32;
use super::util;
use RoaringBitmap;
use RoaringBitmap64;

struct To64Iter<'a> {
    hi: u32,
    inner: Iter32<'a>,
}

impl<'a> Iterator for To64Iter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        //self.size_hint.saturating_sub(1);
        self.inner.next().map(|n| util::join(self.hi, n))
    }
}

/// An iterator for `RoaringBitmap64`.
pub struct Iter<'a> {
    inner: iter::FlatMap<btree_map::Iter<'a, u32, RoaringBitmap>,
                         To64Iter<'a>,
                         fn((&'a u32, &'a RoaringBitmap)) -> To64Iter<'a>>,
    size_hint: u64,
}

/// An iterator for `RoaringBitmap64`.
pub struct IntoIter {
    inner: iter::FlatMap<btree_map::IntoIter<u32, RoaringBitmap>,
                         RoaringBitmap,
                         fn(RoaringBitmap) -> RoaringBitmap>,
    size_hint: u64,
}

fn to64iter<'a>(t: (&'a u32, &'a RoaringBitmap)) -> To64Iter<'a> {
    To64Iter {
        hi: *t.0,
        inner: t.1.iter(),
    }
}

impl<'a> Iter<'a> {
    fn new(map: &BTreeMap<u32, RoaringBitmap>) -> Iter {

        fn identity<T>(t: T) -> T {
            t
        };
        let size_hint: u64 = map.iter().map(|(&hi, &r)| r.len()).sum();



        let i = map.iter()
            .flat_map(to64iter as _);
        Iter {
            inner: i,
            size_hint: 0,
        }

    }
}

impl IntoIter {
    fn new(containers: Vec<RoaringBitmap>) -> IntoIter {
        fn identity<T>(t: T) -> T {
            t
        }
        let size_hint = containers.iter().map(|c| c.len).sum();
        IntoIter {
            inner: containers.into_iter().flat_map(identity as _),
            size_hint: size_hint,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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

impl RoaringBitmap64 {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by
    /// value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use std::iter::FromIterator;
    ///
    /// let bitmap = RoaringBitmap::from_iter(1..3);
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

impl<'a> IntoIterator for &'a RoaringBitmap64 {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl IntoIterator for RoaringBitmap64 {
    type Item = u32;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(self.containers)
    }
}

impl FromIterator<u32> for RoaringBitmap64 {
    fn from_iter<I: IntoIterator<Item = u32>>(iterator: I) -> RoaringBitmap64 {
        let mut rb = RoaringBitmap64::new();
        rb.extend(iterator);
        rb
    }
}

impl Extend<u32> for RoaringBitmap64 {
    fn extend<I: IntoIterator<Item = u32>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}
