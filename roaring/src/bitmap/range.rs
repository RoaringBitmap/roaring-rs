use core::ops::RangeBounds;
use core::ops::RangeInclusive;

use super::container::Container;
use super::iter;
use super::store;
use super::util;
use crate::RoaringBitmap;

/// Iterator over a consecutive subsequence of a bitmap.
/// Efficient; O( log[n] + k ),
/// where n is the bitmap's length
/// and k is the subsequence's length.
pub struct RangeIter<'a> {
    first: store::StorePartIter<'a>,
    between: iter::Iter<'a>,
    last: store::StorePartIter<'a>,
    // size_hint: u64,
}

impl<'a> RangeIter<'a> {
    pub fn new<R>(containers: &'a [Container], range: R) -> RangeIter<'a>
    where
        R: RangeBounds<u32>,
    {
        let (start, end) = match util::convert_range_to_inclusive(range) {
            Some(range) => (*range.start(), *range.end()),
            None => return RangeIter::empty(),
        };

        let (start_key, start_low) = util::split(start);
        let (end_key, end_low) = util::split(end);

        let s = containers.binary_search_by_key(&start_key, |c| c.key);
        let e = containers.binary_search_by_key(&end_key, |c| c.key);

        if s == e {
            // single container
            return match s {
                Ok(i) => RangeIter {
                    first: Self::container_part(&containers[i], start_low..=end_low, start_key),
                    between: iter::Iter::empty(),
                    last: store::StorePartIter::empty(),
                },
                Err(_) => RangeIter::empty(), // nothing to iterate over
            };
        }

        // multiple containers
        let (first, inner_start) = match s {
            Ok(i) => (Self::container_part(&containers[i], start_low..=u16::MAX, start_key), i + 1),
            Err(i) => (store::StorePartIter::empty(), i),
        };
        let (last, inner_stop) = match e {
            Ok(i) => (Self::container_part(&containers[i], u16::MIN..=end_low, end_key), i),
            Err(i) => (store::StorePartIter::empty(), i),
        };
        let between = iter::Iter::new(&containers[inner_start..inner_stop]);

        RangeIter { first, between, last }
    }
    fn container_part(
        container: &Container,
        range: RangeInclusive<u16>,
        key: u16,
    ) -> store::StorePartIter {
        store::StorePartIter::new(key, &container.store, range)
    }
    fn empty() -> RangeIter<'a> {
        RangeIter {
            first: store::StorePartIter::empty(),
            between: iter::Iter::empty(),
            last: store::StorePartIter::empty(),
        }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if let f @ Some(_) = self.first.next() {
            return f;
        }
        if let b @ Some(_) = self.between.next() {
            return b;
        }
        self.last.next()
    }
}

impl RoaringBitmap {
    /// Efficiently obtains an iterator over the specified range.
    ///
    /// # Examples
    ///
    /// ```
    /// use roaring::RoaringBitmap;
    ///
    /// // let mut rb = RoaringBitmap::new();
    /// // rb.insert(0);
    /// // rb.insert(1);
    /// // rb.insert(10);
    /// // rb.insert(999_999);
    /// // rb.insert(1_000_000);
    /// //
    /// // let expected = vec![1,10,999_999];
    /// // let actual: Vec<u32> = rb.range(1..=999_999).collect();
    /// // assert_eq!(expected, actual);
    ///
    /// let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();
    ///
    /// let expected = vec![10,11,12];
    /// let actual: Vec<u32> = rb.range(0..13).collect();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn range<R>(&self, range: R) -> RangeIter
    where
        R: RangeBounds<u32>,
    {
        RangeIter::new(&self.containers, range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_bitmap() {
        let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();

        let expected = vec![10, 11, 12];
        let actual: Vec<u32> = rb.range(0..13).collect();
        assert_eq!(expected, actual);
    }
}
