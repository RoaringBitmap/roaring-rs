use super::container::Container;
use crate::{NonSortedIntegers, RoaringBitmap};

use crate::bitmap::container;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use alloc::collections::vec_deque::VecDeque;
use core::iter::Peekable;

use iter_inner::IterInternal;
/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    containers: &'a [Container],
    iter_front: Option<Peekable<container::Iter<'a>>>,
    iter_back: Option<Peekable<container::Iter<'a>>>,
    size_hint: u64,
}
impl<'a> Iter<'a> {
    fn new(containers: &'a [Container]) -> Self {
        let size_hint = containers.iter().map(|c| c.len()).sum();
        Self { containers, iter_front: None, iter_back: None, size_hint }
    }

    /// Advance the iterator to the first position where the item has a value >= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.iter();
    /// iter.advance_to(2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn advance_to(&mut self, n: u32) {
        self.advance_to_inner(n);
    }
}
impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.next_inner()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.fold_inner(init, f)
    }
}
impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_back_inner()
    }

    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        self.rfold_inner(init, f)
    }
}
#[cfg(target_pointer_width = "64")]
impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}
/// An iterator for `RoaringBitmap`.
pub struct IntoIter {
    containers: VecDeque<Container>,
    iter_front: Option<Peekable<container::Iter<'static>>>,
    iter_back: Option<Peekable<container::Iter<'static>>>,
    size_hint: u64,
}
impl IntoIter {
    fn new(containers: Vec<Container>) -> Self {
        let size_hint = containers.iter().map(|c| c.len()).sum();
        Self {
            containers: VecDeque::from(containers),
            iter_front: None,
            iter_back: None,
            size_hint,
        }
    }

    /// Advance the iterator to the first position where the item has a value >= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.into_iter();
    /// iter.advance_to(2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn advance_to(&mut self, n: u32) {
        self.advance_to_inner(n)
    }
}
impl Iterator for IntoIter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.next_inner()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.fold_inner(init, f)
    }
}
impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_back_inner()
    }
    #[inline]
    fn rfold<Acc, Fold>(self, init: Acc, f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        self.rfold_inner(init, f)
    }
}
#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}

impl RoaringBitmap {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
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
impl<const N: usize> From<[u32; N]> for RoaringBitmap {
    fn from(arr: [u32; N]) -> Self {
        RoaringBitmap::from_iter(arr)
    }
}
impl FromIterator<u32> for RoaringBitmap {
    fn from_iter<I: IntoIterator<Item = u32>>(iterator: I) -> RoaringBitmap {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}
impl<'a> FromIterator<&'a u32> for RoaringBitmap {
    fn from_iter<I: IntoIterator<Item = &'a u32>>(iterator: I) -> RoaringBitmap {
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
        for value in iterator {
            self.insert(*value);
        }
    }
}
impl RoaringBitmap {
    /// Create the set from a sorted iterator. Values must be sorted and deduplicated.
    ///
    /// The values of the iterator must be ordered and strictly greater than the greatest value
    /// in the set. If a value in the iterator doesn't satisfy this requirement, it is not added
    /// and the append operation is stopped.
    ///
    /// Returns `Ok` with the requested `RoaringBitmap`, `Err` with the number of elements
    /// that were correctly appended before failure.
    ///
    /// # Example: Create a set from an ordered list of integers.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::from_sorted_iter(0..10).unwrap();
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    ///
    /// # Example: Try to create a set from a non-ordered list of integers.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let integers = 0..10u32;
    /// let error = RoaringBitmap::from_sorted_iter(integers.rev()).unwrap_err();
    ///
    /// assert_eq!(error.valid_until(), 1);
    /// ```
    pub fn from_sorted_iter<I: IntoIterator<Item = u32>>(
        iterator: I,
    ) -> Result<RoaringBitmap, NonSortedIntegers> {
        let mut rb = RoaringBitmap::new();
        rb.append(iterator).map(|_| rb)
    }
    /// Extend the set with a sorted iterator.
    ///
    /// The values of the iterator must be ordered and strictly greater than the greatest value
    /// in the set. If a value in the iterator doesn't satisfy this requirement, it is not added
    /// and the append operation is stopped.
    ///
    /// Returns `Ok` with the number of elements appended to the set, `Err` with
    /// the number of elements we effectively appended before an error occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// assert_eq!(rb.append(0..10), Ok(10));
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    pub fn append<I: IntoIterator<Item = u32>>(
        &mut self,
        iterator: I,
    ) -> Result<u64, NonSortedIntegers> {
        let mut iterator = iterator.into_iter();
        let mut prev = match (iterator.next(), self.max()) {
            (None, _) => return Ok(0),
            (Some(first), Some(max)) if first <= max => {
                return Err(NonSortedIntegers { valid_until: 0 });
            }
            (Some(first), _) => first,
        };
        self.push_unchecked(prev);
        let mut count = 1;
        for value in iterator {
            if value <= prev {
                return Err(NonSortedIntegers { valid_until: count });
            } else {
                self.push_unchecked(value);
                prev = value;
                count += 1;
            }
        }
        Ok(count)
    }
}

mod iter_inner {
    use crate::bitmap::container::Container;
    use crate::bitmap::{container, util, IntoIter, Iter};
    use core::iter::Peekable;
    use core::slice;

    // To get rid of clippy complex type warning
    pub(super) type DecomposeInnerIter<T> = Option<Peekable<T>>;
    pub(super) trait IterInternal {
        type InnerIter: Iterator<Item = u32> + DoubleEndedIterator;
        type Container: IntoIterator<IntoIter = Self::InnerIter, Item = u32> + AsRef<Container>;

        type ContainerIterator: Iterator<Item = Self::Container> + DoubleEndedIterator;
        type IntoContainerIterator: IntoIterator<IntoIter = Self::ContainerIterator>;

        fn pop_container_front(&mut self) -> Option<Self::Container>;
        fn pop_container_back(&mut self) -> Option<Self::Container>;

        fn drain_containers_until(&mut self, index: usize);

        fn clear_containers(&mut self);

        fn find_container(&self, key: u16) -> Option<usize>;

        fn iter_front_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>>;
        fn iter_back_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>>;

        fn dec_size_hint(&mut self);

        fn decompose(
            self,
        ) -> (
            DecomposeInnerIter<Self::InnerIter>,
            DecomposeInnerIter<Self::InnerIter>,
            Self::IntoContainerIterator,
        );

        #[inline]
        fn next_inner(&mut self) -> Option<u32> {
            self.dec_size_hint();
            loop {
                if let Some(iter) = self.iter_front_mut() {
                    if let item @ Some(_) = iter.next() {
                        return item;
                    }
                }
                if self.advance_container().is_some() {
                    continue;
                } else {
                    return None;
                }
            }
        }

        #[inline]
        fn next_back_inner(&mut self) -> Option<u32> {
            self.dec_size_hint();
            loop {
                if let Some(iter) = self.iter_back_mut() {
                    if let item @ Some(_) = iter.next_back() {
                        return item;
                    }
                }
                if self.advance_container_back().is_some() {
                    continue;
                } else {
                    return None;
                }
            }
        }

        fn advance_to_inner(&mut self, n: u32) {
            let (key, _) = util::split(n);
            fn advance_iter(iter: &mut Peekable<impl Iterator<Item = u32>>, n: u32) -> Option<u32> {
                while let Some(next) = iter.peek().cloned() {
                    if next < n {
                        iter.next();
                    } else {
                        return Some(next);
                    }
                }
                None
            }
            if let Some(index) = self.find_container(key) {
                self.drain_containers_until(index);
                let container = self.pop_container_front().expect("bug!");
                let mut iter = container.into_iter().peekable();
                advance_iter(&mut iter, n);
                *self.iter_front_mut() = Some(iter);
            } else {
                // there are no containers with given key. Look in iter_front and iter_back.
                self.clear_containers();
                if let Some(iter_front) = self.iter_front_mut() {
                    if iter_front.peek().cloned().map(|n| util::split(n).0) == Some(key) {
                        advance_iter(iter_front, n);
                        return;
                    }
                }
                if let Some(iter_back) = self.iter_back_mut() {
                    if iter_back.peek().cloned().map(|n| util::split(n).0) == Some(key) {
                        advance_iter(iter_back, n);
                        *self.iter_front_mut() = None;
                        return;
                    }
                }
                *self.iter_front_mut() = None;
                *self.iter_back_mut() = None;
            }
        }

        #[inline]
        fn fold_inner<B, F>(self, init: B, f: F) -> B
        where
            F: FnMut(B, u32) -> B,
            Self: Sized,
        {
            let (iter_front, iter_back, containers) = self.decompose();
            match (iter_front, iter_back) {
                (Some(iter_front), Some(iter_back)) => iter_front
                    .chain(containers.into_iter().flatten())
                    .chain(iter_back)
                    .fold(init, f),
                (Some(iter_front), None) => {
                    iter_front.chain(containers.into_iter().flatten()).fold(init, f)
                }
                (None, Some(iter_back)) => {
                    containers.into_iter().flatten().chain(iter_back).fold(init, f)
                }
                (None, None) => containers.into_iter().flatten().fold(init, f),
            }
        }

        fn rfold_inner<Acc, Fold>(self, init: Acc, f: Fold) -> Acc
        where
            Fold: FnMut(Acc, u32) -> Acc,
            Self: Sized,
        {
            let (iter_front, iter_back, containers) = self.decompose();
            match (iter_front, iter_back) {
                (Some(iter_front), Some(iter_back)) => iter_front
                    .chain(containers.into_iter().flatten())
                    .chain(iter_back)
                    .rfold(init, f),
                (Some(iter_front), None) => {
                    iter_front.chain(containers.into_iter().flatten()).rfold(init, f)
                }
                (None, Some(iter_back)) => {
                    containers.into_iter().flatten().chain(iter_back).rfold(init, f)
                }
                (None, None) => containers.into_iter().flatten().rfold(init, f),
            }
        }

        fn advance_container(&mut self) -> Option<u16> {
            if let Some(container) = self.pop_container_front() {
                let result = container.as_ref().key;
                *self.iter_front_mut() = Some(container.into_iter().peekable());
                Some(result)
            } else if self.iter_back_mut().is_some() {
                let mut iter_none = None;
                core::mem::swap(&mut iter_none, self.iter_back_mut());
                core::mem::swap(&mut iter_none, self.iter_front_mut());
                *self.iter_back_mut() = None;
                if let Some(v) = self.iter_front_mut().as_mut().and_then(|i| i.peek().cloned()) {
                    let (key, _) = util::split(v);
                    Some(key)
                } else {
                    None
                }
            } else {
                None
            }
        }

        fn advance_container_back(&mut self) -> Option<u16> {
            if let Some(container) = self.pop_container_back() {
                let result = container.as_ref().key;
                *self.iter_back_mut() = Some(container.into_iter().peekable());
                Some(result)
            } else if self.iter_front_mut().is_some() {
                let mut iter_none = None;
                core::mem::swap(&mut iter_none, self.iter_front_mut());
                core::mem::swap(&mut iter_none, self.iter_back_mut());

                if let Some(v) = self.iter_back_mut().as_mut().and_then(|i| i.peek().cloned()) {
                    let (key, _) = util::split(v);
                    Some(key)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    impl IterInternal for IntoIter {
        type InnerIter = container::Iter<'static>;
        type Container = Container;
        type ContainerIterator = alloc::collections::vec_deque::IntoIter<Self::Container>;
        type IntoContainerIterator = alloc::collections::vec_deque::IntoIter<Self::Container>;

        fn pop_container_front(&mut self) -> Option<Self::Container> {
            self.containers.pop_front()
        }

        fn pop_container_back(&mut self) -> Option<Self::Container> {
            self.containers.pop_back()
        }

        fn drain_containers_until(&mut self, index: usize) {
            self.containers.drain(0..index);
        }

        fn clear_containers(&mut self) {
            self.containers.clear();
        }

        fn find_container(&self, key: u16) -> Option<usize> {
            match self.containers.binary_search_by_key(&key, |container| container.key) {
                Ok(index) | Err(index) if index < self.containers.len() => Some(index),
                _ => None,
            }
        }

        fn iter_front_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>> {
            &mut self.iter_front
        }

        fn iter_back_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>> {
            &mut self.iter_back
        }

        fn dec_size_hint(&mut self) {
            self.size_hint = self.size_hint.saturating_sub(1);
        }

        fn decompose(
            self,
        ) -> (
            Option<Peekable<Self::InnerIter>>,
            Option<Peekable<Self::InnerIter>>,
            Self::IntoContainerIterator,
        ) {
            (self.iter_front, self.iter_back, self.containers.into_iter())
        }
    }

    impl<'a> IterInternal for Iter<'a> {
        type InnerIter = container::Iter<'a>;
        type Container = &'a Container;
        type ContainerIterator = slice::Iter<'a, Container>;

        type IntoContainerIterator = slice::Iter<'a, Container>;

        fn pop_container_front(&mut self) -> Option<Self::Container> {
            if let Some((first, rest)) = self.containers.split_first() {
                self.containers = rest;
                Some(first)
            } else {
                None
            }
        }

        fn pop_container_back(&mut self) -> Option<Self::Container> {
            if let Some((last, rest)) = self.containers.split_last() {
                self.containers = rest;
                Some(last)
            } else {
                None
            }
        }

        fn drain_containers_until(&mut self, index: usize) {
            self.containers = &self.containers[index..]
        }

        fn clear_containers(&mut self) {
            self.containers = &[];
        }

        fn find_container(&self, key: u16) -> Option<usize> {
            match self.containers.binary_search_by_key(&key, |container| container.key) {
                Ok(index) | Err(index) if index < self.containers.len() => Some(index),
                _ => None,
            }
        }

        fn iter_front_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>> {
            &mut self.iter_front
        }

        fn iter_back_mut(&mut self) -> &mut Option<Peekable<Self::InnerIter>> {
            &mut self.iter_back
        }

        fn dec_size_hint(&mut self) {
            self.size_hint = self.size_hint.saturating_sub(1);
        }

        fn decompose(
            self,
        ) -> (
            Option<Peekable<Self::InnerIter>>,
            Option<Peekable<Self::InnerIter>>,
            Self::IntoContainerIterator,
        ) {
            (self.iter_front, self.iter_back, self.containers.iter())
        }
    }
}
