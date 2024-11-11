use alloc::vec;
use core::iter::FusedIterator;
use core::slice;
use core::ops::RangeBounds;

use super::container::Container;
use super::{container, util};
use crate::{NonSortedIntegers, RoaringBitmap};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    front: Option<container::Iter<'a>>,
    containers: slice::Iter<'a, Container>,
    back: Option<container::Iter<'a>>,
}

/// An iterator for `RoaringBitmap`.
pub struct IntoIter {
    front: Option<container::Iter<'static>>,
    containers: vec::IntoIter<Container>,
    back: Option<container::Iter<'static>>,
}

#[inline]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}

fn advance_to_impl<'a, It>(
    n: u32,
    front_iter: &mut Option<container::Iter<'a>>,
    containers: &mut It,
    back_iter: &mut Option<container::Iter<'a>>,
) where
    It: Iterator,
    It: AsRef<[Container]>,
    It::Item: IntoIterator<IntoIter = container::Iter<'a>>,
{
    let (key, index) = util::split(n);
    if let Some(iter) = front_iter {
        match key.cmp(&iter.key) {
            core::cmp::Ordering::Less => return,
            core::cmp::Ordering::Equal => {
                iter.advance_to(index);
                return;
            }
            core::cmp::Ordering::Greater => {
                *front_iter = None;
            }
        }
    }
    let containers_slice = containers.as_ref();
    let containers_len = containers_slice.len();
    let to_skip = match containers_slice.binary_search_by_key(&key, |c| c.key) {
        Ok(n) => {
            let container = containers.nth(n).expect("binary search returned a valid index");
            let mut container_iter = container.into_iter();
            container_iter.advance_to(index);
            *front_iter = Some(container_iter);
            return;
        }
        Err(n) => n,
    };

    if let Some(n) = to_skip.checked_sub(1) {
        containers.nth(n);
    }
    if to_skip != containers_len {
        // There are still containers with keys greater than the key we are looking for,
        // the key we're looking _can't_ be in the back iterator.
        return;
    }
    if let Some(iter) = back_iter {
        match key.cmp(&iter.key) {
            core::cmp::Ordering::Less => {}
            core::cmp::Ordering::Equal => {
                iter.advance_to(index);
            }
            core::cmp::Ordering::Greater => {
                *back_iter = None;
            }
        }
    }
}

fn advance_back_to_impl<'a, It>(
    n: u32,
    front_iter: &mut Option<container::Iter<'a>>,
    containers: &mut It,
    back_iter: &mut Option<container::Iter<'a>>,
) where
    It: DoubleEndedIterator,
    It: AsRef<[Container]>,
    It::Item: IntoIterator<IntoIter = container::Iter<'a>>,
{
    let (key, index) = util::split(n);
    if let Some(iter) = back_iter {
        match key.cmp(&iter.key) {
            core::cmp::Ordering::Greater => return,
            core::cmp::Ordering::Equal => {
                iter.advance_back_to(index);
                return;
            }
            core::cmp::Ordering::Less => {
                *back_iter = None;
            }
        }
    }
    let containers_slice = containers.as_ref();
    let containers_len = containers_slice.len();
    let to_skip = match containers_slice.binary_search_by_key(&key, |c| c.key) {
        Ok(n) => {
            // n must be less than containers_len, so this can never underflow
            let n = containers_len - n - 1;
            let container = containers.nth_back(n).expect("binary search returned a valid index");
            let mut container_iter = container.into_iter();
            container_iter.advance_back_to(index);
            *back_iter = Some(container_iter);
            return;
        }
        Err(n) => containers_len - n,
    };

    if let Some(n) = to_skip.checked_sub(1) {
        containers.nth_back(n);
    }
    if to_skip != containers_len {
        // There are still containers with keys less than the key we are looking for,
        // the key we're looking _can't_ be in the front iterator.
        return;
    }
    if let Some(iter) = front_iter {
        match key.cmp(&iter.key) {
            core::cmp::Ordering::Greater => {}
            core::cmp::Ordering::Equal => {
                iter.advance_back_to(index);
            }
            core::cmp::Ordering::Less => {
                *front_iter = None;
            }
        }
    }
}

impl Iter<'_> {
    fn new(containers: &[Container]) -> Iter {
        Iter { front: None, containers: containers.iter(), back: None }
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
        advance_to_impl(n, &mut self.front, &mut self.containers, &mut self.back);
    }

    /// Advance the back of the iterator to the first position where the item has a value <= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.iter();
    /// iter.advance_back_to(1);
    ///
    /// assert_eq!(iter.next_back(), Some(1));
    /// assert_eq!(iter.next_back(), None);
    /// ```
    pub fn advance_back_to(&mut self, n: u32) {
        advance_back_to_impl(n, &mut self.front, &mut self.containers, &mut self.back);
    }

    pub fn empty() -> Iter<'static> {
        Iter { front: None, containers: [].iter(), back: None }
    }
}

impl IntoIter {
    fn new(containers: Vec<Container>) -> IntoIter {
        IntoIter { front: None, containers: containers.into_iter(), back: None }
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
        advance_to_impl(n, &mut self.front, &mut self.containers, &mut self.back);
    }

    /// Advance the back of the iterator to the first position where the item has a value <= `n`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    /// use core::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<RoaringBitmap>();
    /// let mut iter = bitmap.into_iter();
    /// iter.advance_back_to(1);
    ///
    /// assert_eq!(iter.next_back(), Some(1));
    /// assert_eq!(iter.next_back(), None);
    /// ```
    pub fn advance_back_to(&mut self, n: u32) {
        advance_back_to_impl(n, &mut self.front, &mut self.containers, &mut self.back);
    }
}

fn size_hint_impl(
    front: &Option<container::Iter<'_>>,
    containers: &impl AsRef<[Container]>,
    back: &Option<container::Iter<'_>>,
) -> (usize, Option<usize>) {
    let first_size = front.as_ref().map_or(0, |it| it.len());
    let last_size = back.as_ref().map_or(0, |it| it.len());
    let mut size = first_size + last_size;
    for container in containers.as_ref() {
        match size.checked_add(container.len() as usize) {
            Some(new_size) => size = new_size,
            None => return (usize::MAX, None),
        }
    }
    (size, Some(size))
}

impl Iterator for Iter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            if let Some(x) = and_then_or_clear(&mut self.front, Iterator::next) {
                return Some(x);
            }
            self.front = match self.containers.next() {
                Some(inner) => Some(inner.into_iter()),
                None => return and_then_or_clear(&mut self.back, Iterator::next),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint_impl(&self.front, &self.containers, &self.back)
    }

    #[inline]
    fn fold<B, F>(mut self, mut init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        if let Some(iter) = &mut self.front {
            init = iter.fold(init, &mut f);
        }
        init = self.containers.fold(init, |acc, container| {
            let iter = <&Container>::into_iter(container);
            iter.fold(acc, &mut f)
        });
        if let Some(iter) = &mut self.back {
            init = iter.fold(init, &mut f);
        };
        init
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let mut count = self.front.map_or(0, Iterator::count);
        count += self.containers.map(|container| container.len() as usize).sum::<usize>();
        count += self.back.map_or(0, Iterator::count);
        count
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut n = n;
        let nth_advance = |it: &mut container::Iter| {
            let len = it.len();
            if n < len {
                it.nth(n)
            } else {
                n -= len;
                None
            }
        };
        if let Some(x) = and_then_or_clear(&mut self.front, nth_advance) {
            return Some(x);
        }
        for container in self.containers.by_ref() {
            let len = container.len() as usize;
            if n < len {
                let mut front_iter = container.into_iter();
                let result = front_iter.nth(n);
                self.front = Some(front_iter);
                return result;
            }
            n -= len;
        }
        and_then_or_clear(&mut self.back, |it| it.nth(n))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(x) = and_then_or_clear(&mut self.back, DoubleEndedIterator::next_back) {
                return Some(x);
            }
            self.back = match self.containers.next_back() {
                Some(inner) => Some(inner.into_iter()),
                None => return and_then_or_clear(&mut self.front, DoubleEndedIterator::next_back),
            }
        }
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, mut init: Acc, mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        if let Some(iter) = &mut self.back {
            init = iter.rfold(init, &mut fold);
        }
        init = self.containers.rfold(init, |acc, container| {
            let iter = container.into_iter();
            iter.rfold(acc, &mut fold)
        });
        if let Some(iter) = &mut self.front {
            init = iter.rfold(init, &mut fold);
        };
        init
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let mut n = n;
        let nth_advance = |it: &mut container::Iter| {
            let len = it.len();
            if n < len {
                it.nth_back(n)
            } else {
                n -= len;
                None
            }
        };
        if let Some(x) = and_then_or_clear(&mut self.back, nth_advance) {
            return Some(x);
        }
        for container in self.containers.by_ref().rev() {
            let len = container.len() as usize;
            if n < len {
                let mut front_iter = container.into_iter();
                let result = front_iter.nth_back(n);
                self.back = Some(front_iter);
                return result;
            }
            n -= len;
        }
        and_then_or_clear(&mut self.front, |it| it.nth_back(n))
    }
}

#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for Iter<'_> {}
impl FusedIterator for Iter<'_> {}

impl Iterator for IntoIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            if let Some(x) = and_then_or_clear(&mut self.front, Iterator::next) {
                return Some(x);
            }
            match self.containers.next() {
                Some(inner) => self.front = Some(inner.into_iter()),
                None => return and_then_or_clear(&mut self.back, Iterator::next),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        size_hint_impl(&self.front, &self.containers, &self.back)
    }

    #[inline]
    fn fold<B, F>(mut self, mut init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        if let Some(iter) = &mut self.front {
            init = iter.fold(init, &mut f);
        }
        init = self.containers.fold(init, |acc, container| {
            let iter = <Container>::into_iter(container);
            iter.fold(acc, &mut f)
        });
        if let Some(iter) = &mut self.back {
            init = iter.fold(init, &mut f);
        };
        init
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let mut count = self.front.map_or(0, Iterator::count);
        count += self.containers.map(|container| container.len() as usize).sum::<usize>();
        count += self.back.map_or(0, Iterator::count);
        count
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut n = n;
        let nth_advance = |it: &mut container::Iter| {
            let len = it.len();
            if n < len {
                it.nth(n)
            } else {
                n -= len;
                None
            }
        };
        if let Some(x) = and_then_or_clear(&mut self.front, nth_advance) {
            return Some(x);
        }
        for container in self.containers.by_ref() {
            let len = container.len() as usize;
            if n < len {
                let mut front_iter = container.into_iter();
                let result = front_iter.nth(n);
                self.front = Some(front_iter);
                return result;
            }
            n -= len;
        }
        and_then_or_clear(&mut self.back, |it| it.nth(n))
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(x) = and_then_or_clear(&mut self.back, DoubleEndedIterator::next_back) {
                return Some(x);
            }
            match self.containers.next_back() {
                Some(inner) => self.back = Some(inner.into_iter()),
                None => return and_then_or_clear(&mut self.front, DoubleEndedIterator::next_back),
            }
        }
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, mut init: Acc, mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        if let Some(iter) = &mut self.back {
            init = iter.rfold(init, &mut fold);
        }
        init = self.containers.rfold(init, |acc, container| {
            let iter = container.into_iter();
            iter.rfold(acc, &mut fold)
        });
        if let Some(iter) = &mut self.front {
            init = iter.rfold(init, &mut fold);
        };
        init
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let mut n = n;
        let nth_advance = |it: &mut container::Iter| {
            let len = it.len();
            if n < len {
                it.nth_back(n)
            } else {
                n -= len;
                None
            }
        };
        if let Some(x) = and_then_or_clear(&mut self.back, nth_advance) {
            return Some(x);
        }
        for container in self.containers.by_ref().rev() {
            let len = container.len() as usize;
            if n < len {
                let mut front_iter = container.into_iter();
                let result = front_iter.nth_back(n);
                self.back = Some(front_iter);
                return result;
            }
            n -= len;
        }
        and_then_or_clear(&mut self.front, |it| it.nth_back(n))
    }
}

#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for IntoIter {}
impl FusedIterator for IntoIter {}

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

    /// Efficiently obtains an iterator over the specified range.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb = RoaringBitmap::new();
    /// rb.insert(0);
    /// rb.insert(1);
    /// rb.insert(10);
    /// rb.insert(999_999);
    /// rb.insert(1_000_000);
    /// let expected = vec![1,10,999_999];
    /// let actual: Vec<u32> = rb.iter_range(1..=999_999).collect();
    /// assert_eq!(expected, actual);
    ///
    /// let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();
    /// let expected = vec![10,11,12];
    /// let actual: Vec<u32> = rb.iter_range(0..13).collect();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn iter_range<R>(&self, range: R) -> Iter
    where R: RangeBounds <u32> {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return Iter::empty(),
        };
        let mut iter = self.iter();
        iter.advance_to(start);
        iter.advance_back_to(end);
        iter
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_range_array() {
        let mut rb = RoaringBitmap::new();
        rb.insert(0);
        rb.insert(1);
        rb.insert(10);
        rb.insert(999_999);
        rb.insert(1_000_000);
        let expected = vec![1,10,999_999];
        let actual: Vec<u32> = rb.iter_range(1..=999_999).collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_iter_range_bitmap() {
        let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();

        let expected = vec![10, 11, 12];
        let actual: Vec<u32> = rb.iter_range(0..13).collect();
        assert_eq!(expected, actual);
    }
}
