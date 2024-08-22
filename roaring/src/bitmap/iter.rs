use alloc::vec;
use core::slice;

use super::container::Container;
use crate::{NonSortedIntegers, RoaringBitmap};

use crate::bitmap::{container, util};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use std::iter::Peekable;

macro_rules! impl_iter {
    ($t:ident,$c:ty, $c_iter:ty, $iter_lt:lifetime $(,$lt:lifetime)?) => {
        /// An iterator for `RoaringBitmap`.
        pub struct $t$(<$lt>)? {
            containers_iter: $c_iter,
            iter_front: Option<container::Iter<$iter_lt>>,
            iter_back: Option<container::Iter<$iter_lt>>,
            size_hint: u64,
        }

        impl$(<$lt>)? $t$(<$lt>)? {
            fn new(containers: $c) -> Self {
                let size_hint = containers.iter().map(|c| c.len()).sum();
                let containers_iter = containers.into_iter();
                Self {
                    containers_iter,
                    iter_front: None,
                    iter_back: None,
                    size_hint,
                }
            }
        }

        impl$(<$lt>)? Iterator for $t$(<$lt>)? {
            type Item = u32;

            fn next(&mut self) -> Option<u32> {
                self.size_hint = self.size_hint.saturating_sub(1);
                loop {
                    if let Some(iter) = &mut self.iter_front {
                        if let item @ Some(_) = iter.next() {
                            return item
                        }
                    }
                    if let Some(container) = self.containers_iter.next() {
                        self.iter_front = Some(container.into_iter())
                    } else if let Some(iter) = &mut self.iter_back {
                        return iter.next()
                    } else {
                        return None
                    }
                }
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
                match (self.iter_front, self.iter_back) {
                    (Some(iter_front), Some(iter_back)) => {
                        iter_front.chain(self.containers_iter.flatten()).chain(iter_back).fold(init, f)
                    },
                    (Some(iter_front), None) => {
                        iter_front.chain(self.containers_iter.flatten()).fold(init, f)
                    },
                    (None, Some(iter_back)) => {
                        self.containers_iter.flatten().chain(iter_back).fold(init, f)
                    },
                    (None, None) => self.containers_iter.flatten().fold(init, f)
                }
            }
        }

        impl$(<$lt>)? DoubleEndedIterator for $t$(<$lt>)? {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.size_hint = self.size_hint.saturating_sub(1);
                loop {
                    if let Some(iter) = &mut self.iter_back {
                        if let item @ Some(_) = iter.next_back() {
                            return item
                        }
                    }
                    if let Some(container) = self.containers_iter.next_back() {
                        self.iter_back = Some(container.into_iter())
                    } else if let Some(iter) = &mut self.iter_front {
                        return iter.next_back()
                    } else {
                        return None
                    }
                }
            }

            #[inline]
            fn rfold<Acc, Fold>(self, init: Acc, f: Fold) -> Acc
            where
                Fold: FnMut(Acc, Self::Item) -> Acc,
            {
                match (self.iter_front, self.iter_back) {
                    (Some(iter_front), Some(iter_back)) => {
                        iter_front.chain(self.containers_iter.flatten()).chain(iter_back).rfold(init, f)
                    },
                    (Some(iter_front), None) => {
                        iter_front.chain(self.containers_iter.flatten()).rfold(init, f)
                    },
                    (None, Some(iter_back)) => {
                        self.containers_iter.flatten().chain(iter_back).rfold(init, f)
                    },
                    (None, None) => self.containers_iter.flatten().rfold(init, f)
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        impl$(<$lt>)? ExactSizeIterator for $t$(<$lt>)? {
            fn len(&self) -> usize {
                self.size_hint as usize
            }
        }
    };
}
impl_iter!(Iter, &'a [Container], slice::Iter<'a, Container>, 'a, 'a);
impl_iter!(IntoIter, Vec<Container>, vec::IntoIter<Container>, 'static);

pub struct AdvanceToIter<'a, CI> {
    containers_iter: CI,
    iter_front: Peekable<container::Iter<'a>>,
    iter_back: Option<Peekable<container::Iter<'a>>>,
}

impl<'a, CI> AdvanceToIter<'a, CI>
where
    Self: AdvanceIterContainer<'a>,
{
    fn new(
        containers_iter: CI,
        iter_front: Option<container::Iter<'a>>,
        iter_back: Option<container::Iter<'a>>,
        n: u32,
    ) -> Self {
        let mut result = Self {
            containers_iter,
            iter_front: container::Iter::empty().peekable(),
            iter_back: iter_back.map(|o| o.peekable()),
        };
        if let Some(iter_front) = iter_front {
            result.iter_front = iter_front.peekable();
        } else {
            result.advance_container();
        }
        if let Some(peek) = result.iter_front.peek().cloned() {
            if peek < n {
                let (peek_key, _) = util::split(peek);
                let (target_key, _) = util::split(n);
                if target_key > peek_key {
                    while let Some(next_key) = result.advance_container() {
                        if next_key >= target_key {
                            break;
                        }
                    }
                }
                while let Some(peek) = result.iter_front.peek() {
                    if *peek >= n {
                        break;
                    } else {
                        result.iter_front.next();
                    }
                }
            }
        }
        result
    }
}

impl<'a, CI> Iterator for AdvanceToIter<'a, CI>
where
    Self: AdvanceIterContainer<'a>,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let item @ Some(_) = self.iter_front.next() {
                return item;
            }
            self.advance_container()?;
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

    /// Iterator over each value >= `n` stored in the RoaringBitmap, guarantees values are ordered
    /// by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.iter_from(2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_from(&self, n: u32) -> AdvanceToIter<'_, slice::Iter<'_, Container>> {
        let (key, _) = util::split(n);
        match self.containers.binary_search_by_key(&key, |container| container.key) {
            Ok(index) | Err(index) => {
                if index == self.containers.len() {
                    // no container has a key >= key(n)
                    AdvanceToIter::new([].iter(), None, None, n)
                } else {
                    let containers = &self.containers[index..];
                    let iter = (&containers[0]).into_iter();
                    if index + 1 < containers.len() - 1 {
                        AdvanceToIter::new(containers[index + 1..].iter(), Some(iter), None, n)
                    } else {
                        AdvanceToIter::new([].iter(), Some(iter), None, n)
                    }
                }
            }
        }
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
        // Name shadowed to prevent accidentally referencing the param
        let mut iterator = iterator.into_iter();

        let mut prev = match (iterator.next(), self.max()) {
            (None, _) => return Ok(0),
            (Some(first), Some(max)) if first <= max => {
                return Err(NonSortedIntegers { valid_until: 0 })
            }
            (Some(first), _) => first,
        };

        // It is now guaranteed that so long as the values of the iterator are
        // monotonically increasing they must also be the greatest in the set.

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

pub trait AdvanceIterContainer<'a> {
    fn advance_container(&mut self) -> Option<u16>;
}

macro_rules! impl_advance_iter_container {
    ($lt:lifetime,$ty:ty) => {
        impl<$lt> AdvanceIterContainer<$lt> for AdvanceToIter<$lt, $ty> {
            fn advance_container(&mut self) -> Option<u16> {
                if let Some(container) = self.containers_iter.next() {
                    let result = container.key;
                    self.iter_front = container.into_iter().peekable();
                    Some(result)
                } else if let Some(iter_back) = &mut self.iter_back {
                    std::mem::swap(iter_back, &mut self.iter_front);
                    self.iter_back = None;
                    if let Some(v) = self.iter_front.peek().cloned() {
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
    };
}
impl_advance_iter_container!('a, slice::Iter<'a, Container>);
impl_advance_iter_container!('a ,vec::IntoIter<Container>);

macro_rules! impl_advance_to {
    ($ty:ty, $ret:ty $(,$lt:lifetime)? ) => {
             impl$(<$lt>)? $ty {
                /// Advance the iterator to the first position where the item has a value >= `n`
                pub fn advance_to(mut self, n: u32) -> $ret {
                    if let Some(iter_front) = self.iter_front {
                        AdvanceToIter::new(self.containers_iter, Some(iter_front), self.iter_back, n)
                    } else {
                        if let Some(container) = self.containers_iter.next() {
                            AdvanceToIter::new(self.containers_iter, Some(container.into_iter()), self.iter_back, n)
                        } else {
                            AdvanceToIter::new(self.containers_iter, Some(container::Iter::empty()), self.iter_back, n)
                        }
                    }
                }
            }
    };
}
impl_advance_to!(Iter<'a>, AdvanceToIter<'a, slice::Iter<'a, Container>>, 'a);
impl_advance_to!(IntoIter, AdvanceToIter<'static, vec::IntoIter<Container>>);
