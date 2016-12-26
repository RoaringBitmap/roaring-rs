use std::slice;
use std::iter::FromIterator;

use RoaringBitmap;
use util::{ self, Halveable, ExtInt };
use container::{ self, Container };

type HalfContainer<Size> = Container<<Size as Halveable>::HalfSize>;

enum Next<'a, Size: ExtInt + Halveable + 'a> {
    Done,
    Value(Size),
    NewIter(Option<container::Iter<'a, <Size as Halveable>::HalfSize>>),
}

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize: 'a {
    inner_iter: Option<container::Iter<'a, <Size as Halveable>::HalfSize>>,
    container_iters: slice::Iter<'a, HalfContainer<Size>>,
}

impl<'a, Size: ExtInt + Halveable> Iter<'a, Size> {
    #[inline]
    #[doc(hidden)] // TODO: pub(crate)
    pub fn new(mut container_iters: slice::Iter<HalfContainer<Size>>) -> Iter<Size> {
        Iter {
            inner_iter: container_iters.next().map(|i| i.iter()),
            container_iters: container_iters,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    #[inline]
    fn choose_next(&mut self) -> Next<'a, Size> {
        match self.inner_iter {
            Some(ref mut inner_iter) => match inner_iter.next() {
                Some(value) => Next::Value(Halveable::join(inner_iter.key, value)),
                None => Next::NewIter(self.container_iters.next().map(|i| i.iter())),
            },
            None => Next::Done,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for Iter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
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
        let next = self.container_iters.clone().map(|container| util::cast::<_, usize>(container.len())).sum();
        match self.inner_iter {
            Some(ref inner_iter) => match inner_iter.size_hint() {
                (min, max) => (next + min, max.map(|m| next + m)),
            },
            None => (next, Some(next)),
        }
    }
}

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb: RoaringBitmap<u32> = RoaringBitmap::new();
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
    #[inline]
    pub fn iter<'a>(&'a self) -> Iter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        self.into_iter()
    }
}

impl<Size: ExtInt + Halveable> IntoIterator for RoaringBitmap<Size> {
    type Item = Size;
    type IntoIter = <Vec<Size> as IntoIterator>::IntoIter;
    #[inline]
    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        Vec::from_iter(Iter::new(self.containers.iter())).into_iter()
    }
}

impl<'a, Size: ExtInt + Halveable> IntoIterator for &'a RoaringBitmap<Size> {
    type Item = Size;
    type IntoIter = Iter<'a, Size>;
    #[inline]
    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        Iter::new(self.containers.iter())
    }
}

impl<Size: ExtInt + Halveable> FromIterator<Size> for RoaringBitmap<Size> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = Size>>(iterator: I) -> Self {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> FromIterator<&'a Size> for RoaringBitmap<Size> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a Size>>(iterator: I) -> Self {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl<Size: ExtInt + Halveable> Extend<Size> for RoaringBitmap<Size> {
    #[inline]
    fn extend<I: IntoIterator<Item = Size>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Extend<&'a Size> for RoaringBitmap<Size> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a Size>>(&mut self, iterator: I) {
        for &value in iterator {
            self.insert(value);
        }
    }
}
