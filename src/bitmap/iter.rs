use std::iter::{self, FromIterator};
use std::slice;
use std::vec;

use super::container::Container;
use crate::RoaringBitmap;

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    inner: iter::FlatMap<
        slice::Iter<'a, Container>,
        &'a Container,
        fn(&'a Container) -> &'a Container,
    >,
    size_hint: u64,
}

/// An iterator for `RoaringBitmap`.
pub struct IntoIter {
    inner: iter::FlatMap<vec::IntoIter<Container>, Container, fn(Container) -> Container>,
    size_hint: u64,
}

impl<'a> Iter<'a> {
    fn new(containers: &[Container]) -> Iter {
        fn identity<T>(t: T) -> T {
            t
        }
        let size_hint = containers.iter().map(|c| c.len).sum();
        Iter {
            inner: containers.iter().flat_map(identity as _),
            size_hint,
        }
    }
}

impl IntoIter {
    fn new(containers: Vec<Container>) -> IntoIter {
        fn identity<T>(t: T) -> T {
            t
        }
        let size_hint = containers.iter().map(|c| c.len).sum();
        IntoIter {
            inner: containers.into_iter().flat_map(identity as _),
            size_hint,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.size_hint = self.size_hint.saturating_sub(1);
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
        self.size_hint = self.size_hint.saturating_sub(1);
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

impl RoaringBitmap {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use std::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter {
        Iter::new(&self.containers)
    }
}

impl<'a> IntoIterator for &'a RoaringBitmap {
    type Item = u32;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl IntoIterator for RoaringBitmap {
    type Item = u32;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(self.containers)
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

impl RoaringBitmap {
    /// Create the set from a sorted iterator. Values **must** be sorted.
    ///
    /// This method can be faster than `from_iter` because it skips the binary searches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::from_sorted_iter(0..10);
    ///
    /// assert_eq!(rb.iter().collect::<Vec<u32>>(), (0..10).collect::<Vec<u32>>());
    /// ```
    pub fn from_sorted_iter<I: IntoIterator<Item = u32>>(iterator: I) -> RoaringBitmap {
        let mut rb = RoaringBitmap::new();
        rb.append(iterator);
        rb
    }

    /// Extend the set with a sorted iterator.
    /// All value of the iterator **must** be strictly bigger than the max element
    /// contained in the set.
    ///
    /// This method can be faster than `extend` because it skips the binary searches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.append(0..10);
    ///
    /// assert_eq!(rb.iter().collect::<Vec<u32>>(), (0..10).collect::<Vec<u32>>());
    /// ```
    pub fn append<I: IntoIterator<Item = u32>>(&mut self, iterator: I) {
        for value in iterator {
            self.push(value);
        }
    }
}
