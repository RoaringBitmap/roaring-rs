use super::container::Container;
use crate::{NonSortedIntegers, RoaringBitmap, Value};
use std::{
    iter::{self, FromIterator},
    slice, vec,
};

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, V: Value> {
    inner: iter::Flatten<slice::Iter<'a, Container<V>>>,
    size_hint: u64,
}

/// An iterator for `RoaringBitmap`.
pub struct IntoIter<V: Value> {
    inner: iter::Flatten<vec::IntoIter<Container<V>>>,
    size_hint: u64,
}

impl<'a, V: Value> Iter<'a, V> {
    fn new(containers: &'a [Container<V>]) -> Self {
        let size_hint = containers.iter().map(|c| c.len()).sum();
        Iter { inner: containers.iter().flatten(), size_hint }
    }
}

impl<V: Value> IntoIter<V> {
    fn new(containers: Vec<Container<V>>) -> Self {
        let size_hint = containers.iter().map(|c| c.len()).sum();
        IntoIter { inner: containers.into_iter().flatten(), size_hint }
    }
}

impl<V: Value> Iterator for Iter<'_, V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
}

impl<V: Value> DoubleEndedIterator for Iter<'_, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next_back()
    }
}

#[cfg(target_pointer_width = "64")]
impl<V: Value> ExactSizeIterator for Iter<'_, V> {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}

impl<V: Value> Iterator for IntoIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.size_hint < usize::MAX as u64 {
            (self.size_hint as usize, Some(self.size_hint as usize))
        } else {
            (usize::MAX, None)
        }
    }
}

impl<V: Value> DoubleEndedIterator for IntoIter<V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.size_hint = self.size_hint.saturating_sub(1);
        self.inner.next_back()
    }
}

#[cfg(target_pointer_width = "64")]
impl<V: Value> ExactSizeIterator for IntoIter<V> {
    fn len(&self) -> usize {
        self.size_hint as usize
    }
}

impl<V: Value> RoaringBitmap<V> {
    /// Iterator over each value stored in the RoaringBitmap, guarantees values are ordered by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::Roaring32;
    /// use std::iter::FromIterator;
    ///
    /// let bitmap = (1..3).collect::<Roaring32>();
    /// let mut iter = bitmap.iter();
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<V> {
        Iter::new(&self.containers)
    }
}

impl<'a, V: Value> IntoIterator for &'a RoaringBitmap<V> {
    type Item = V;
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Iter<'a, V> {
        self.iter()
    }
}

impl<V: Value> IntoIterator for RoaringBitmap<V> {
    type Item = V;
    type IntoIter = IntoIter<V>;

    fn into_iter(self) -> IntoIter<V> {
        IntoIter::new(self.containers)
    }
}

impl<const N: usize, V: Value> From<[V; N]> for RoaringBitmap<V> {
    fn from(arr: [V; N]) -> Self {
        RoaringBitmap::from_iter(arr)
    }
}

impl<V: Value> FromIterator<V> for RoaringBitmap<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iterator: I) -> RoaringBitmap<V> {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl<'a, V: Value> FromIterator<&'a V> for RoaringBitmap<V> {
    fn from_iter<I: IntoIterator<Item = &'a V>>(iterator: I) -> RoaringBitmap<V> {
        let mut rb = RoaringBitmap::new();
        rb.extend(iterator);
        rb
    }
}

impl<V: Value> Extend<V> for RoaringBitmap<V> {
    fn extend<I: IntoIterator<Item = V>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl<'a, V: Value> Extend<&'a V> for RoaringBitmap<V> {
    fn extend<I: IntoIterator<Item = &'a V>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(*value);
        }
    }
}

impl<V: Value> RoaringBitmap<V> {
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
    /// use roaring::Roaring32;
    ///
    /// let mut rb = Roaring32::from_sorted_iter(0..10).unwrap();
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    ///
    /// # Example: Try to create a set from a non-ordered list of integers.
    ///
    /// ```rust
    /// use roaring::Roaring32;
    ///
    /// let integers = 0..10u32;
    /// let error = Roaring32::from_sorted_iter(integers.rev()).unwrap_err();
    ///
    /// assert_eq!(error.valid_until(), 1);
    /// ```
    pub fn from_sorted_iter<I: IntoIterator<Item = V>>(
        iterator: I,
    ) -> Result<RoaringBitmap<V>, NonSortedIntegers> {
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
    /// use roaring::Roaring32;
    ///
    /// let mut rb = Roaring32::new();
    /// assert_eq!(rb.append(0..10), Ok(10));
    ///
    /// assert!(rb.iter().eq(0..10));
    /// ```
    pub fn append<I: IntoIterator<Item = V>>(
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
