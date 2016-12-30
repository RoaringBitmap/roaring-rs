use std::slice;
use std::iter::{ self, FromIterator };

use RoaringBitmap;
use container::Container;

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    inner: iter::FlatMap<slice::Iter<'a, Container>, &'a Container, fn(&'a Container) -> &'a Container>,
    size_hint: usize,
}

impl<'a> Iter<'a> {
    fn new(containers: slice::Iter<Container>) -> Iter {
        fn identity<T>(t: T) -> T { t }
        let size_hint = containers.clone().map(|c| c.len as usize).sum();
        Iter {
            inner: containers.flat_map(identity as _),
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
        (self.size_hint, Some(self.size_hint))
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
    /// let mut rb = RoaringBitmap::new();
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
    fn from_iter<I: IntoIterator<Item = u32>>(iterator: I) -> RoaringBitmap {
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
