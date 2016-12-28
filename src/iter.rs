use std::slice;
use std::iter::FromIterator;

use RoaringBitmap;
use util;
use container::{ self, Container };

// NewIter case is 55 bytes, 's okay
// (would be nice to be able to allow this
// difference but warn if it gets even larger....)
#[allow(variant_size_differences)]
enum Next<'a> {
    Done,
    Value(u32),
    NewIter(Option<container::Iter<'a>>),
}

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    inner_iter: Option<container::Iter<'a>>,
    container_iters: slice::Iter<'a, Container>,
}

impl<'a> Iter<'a> {
    fn new(mut container_iters: slice::Iter<Container>) -> Iter {
        Iter {
            inner_iter: container_iters.next().map(|i| i.iter()),
            container_iters: container_iters,
        }
    }
}

impl<'a> Iter<'a> {
    fn choose_next(&mut self) -> Next<'a> {
        match self.inner_iter {
            Some(ref mut inner_iter) => match inner_iter.next() {
                Some(value) => Next::Value(util::join(inner_iter.key, value)),
                None => Next::NewIter(self.container_iters.next().map(|i| i.iter())),
            },
            None => Next::Done,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        match self.choose_next() {
            Next::Done => None,
            Next::Value(val) => Some(val),
            Next::NewIter(new_iter) => {
                self.inner_iter = new_iter;
                self.next()
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let next = self.container_iters.clone().map(|container| container.len as usize).sum();
        match self.inner_iter {
            Some(ref inner_iter) => match inner_iter.size_hint() {
                (min, max) => (next + min, max.map(|m| next + m)),
            },
            None => (next, Some(next)),
        }
    }
}

impl RoaringBitmap {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap = RoaringBitmap::new();
    ///
    /// rb.insert(1);
    /// rb.insert(6);
    /// rb.insert(4);
    ///
    /// let mut iter = rb.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(4));
    /// assert_eq!(iter.next(), Some(6));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a RoaringBitmap {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter::new(self.containers.iter())
    }
}

impl FromIterator<u32> for RoaringBitmap {
    fn from_iter<I: IntoIterator<Item = u32>>(iterator: I) -> Self {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl<'a> FromIterator<&'a u32> for RoaringBitmap {
    fn from_iter<I: IntoIterator<Item = &'a u32>>(iterator: I) -> Self {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl Extend<u32> for RoaringBitmap {
    fn extend<I: IntoIterator<Item = u32>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl<'a> Extend<&'a u32> for RoaringBitmap {
    fn extend<I: IntoIterator<Item = &'a u32>>(&mut self, iterator: I) {
        for &value in iterator {
            self.insert(value);
        }
    }
}
