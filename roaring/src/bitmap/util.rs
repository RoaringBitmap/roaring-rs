use core::ops::{Bound, RangeBounds, RangeInclusive};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Returns the container key and the index
/// in this container for a given integer.
#[inline]
pub fn split(value: u32) -> (u16, u16) {
    ((value >> 16) as u16, value as u16)
}

/// Returns the original integer from the container
/// key and the index of it in the container.
#[inline]
pub fn join(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) + u32::from(low)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConvertRangeError {
    Empty,
    StartGreaterThanEnd,
    StartAndEndEqualExcluded,
}

/// Convert a `RangeBounds<u32>` object to `RangeInclusive<u32>`,
pub fn convert_range_to_inclusive<R>(range: R) -> Result<RangeInclusive<u32>, ConvertRangeError>
where
    R: RangeBounds<u32>,
{
    let start_bound = range.start_bound().cloned();
    let end_bound = range.end_bound().cloned();
    match (start_bound, end_bound) {
        (Bound::Excluded(s), Bound::Excluded(e)) if s == e => {
            Err(ConvertRangeError::StartAndEndEqualExcluded)
        }
        (Bound::Included(s) | Bound::Excluded(s), Bound::Included(e) | Bound::Excluded(e))
            if s > e =>
        {
            Err(ConvertRangeError::StartGreaterThanEnd)
        }
        _ => {
            let start = match start_bound {
                Bound::Included(s) => s,
                Bound::Excluded(s) => s.checked_add(1).ok_or(ConvertRangeError::Empty)?,
                Bound::Unbounded => 0,
            };

            let end = match end_bound {
                Bound::Included(e) => e,
                Bound::Excluded(e) => e.checked_sub(1).ok_or(ConvertRangeError::Empty)?,
                Bound::Unbounded => u32::MAX,
            };

            if start > end {
                // This handles e.g. `x..x`: we've ruled out `start > end` overall, so a value must
                // have been changed via exclusion.
                Err(ConvertRangeError::Empty)
            } else {
                Ok(start..=end)
            }
        }
    }
}

// Thresholds of `SortedInserts`, picked from benchmarks.
const FEW_INSERTS: usize = 8;
const SHORT_SHIFT: usize = 64;

/// Inserts values into a sorted `Vec` at the positions given by failed binary searches.
///
/// Every insertion shifts the values after it, which is quadratic when many values are
/// interleaved with the existing ones. So a value is only inserted directly when that is
/// cheap: when it shifts fewer than `SHORT_SHIFT` values, or for the first `FEW_INSERTS`
/// values that shift more. The other ones are deferred and merged into the vector by
/// `finish` in a single pass.
pub struct SortedInserts<T> {
    long_inserts: usize,
    deferred: Vec<(usize, T)>,
}

impl<T> SortedInserts<T> {
    pub fn new() -> Self {
        SortedInserts { long_inserts: 0, deferred: Vec::new() }
    }

    /// Inserts `value` at `index` in `vec`, or defers it until `finish` is called.
    /// `index` must come from a failed binary search in `vec`, and values must be
    /// given in ascending order.
    pub fn insert(&mut self, vec: &mut Vec<T>, index: usize, value: T) {
        let short = vec.len() - index < SHORT_SHIFT;
        if short || self.long_inserts < FEW_INSERTS {
            // The deferred indexes stay valid: the values deferred so far belong at or
            // before `index`, so before `value`.
            vec.insert(index, value);
            self.long_inserts += usize::from(!short);
        } else {
            self.deferred.push((index, value));
        }
    }

    /// Merges the deferred values into `vec`, moving the values that follow the first
    /// deferred index once.
    pub fn finish(self, vec: &mut Vec<T>) {
        let Some(&(start, _)) = self.deferred.first() else { return };
        vec.reserve(self.deferred.len());
        let mut tail = vec.split_off(start).into_iter();
        let mut prev = start;
        for (index, value) in self.deferred {
            vec.extend(tail.by_ref().take(index - prev));
            vec.push(value);
            prev = index;
        }
        vec.extend(tail);
    }
}

#[cfg(test)]
mod test {
    use super::{convert_range_to_inclusive, join, split, ConvertRangeError, SortedInserts};
    use core::ops::Bound;

    #[cfg(not(feature = "std"))]
    use alloc::vec::Vec;

    #[test]
    fn test_split_u32() {
        assert_eq!((0x0000u16, 0x0000u16), split(0x0000_0000u32));
        assert_eq!((0x0000u16, 0x0001u16), split(0x0000_0001u32));
        assert_eq!((0x0000u16, 0xFFFEu16), split(0x0000_FFFEu32));
        assert_eq!((0x0000u16, 0xFFFFu16), split(0x0000_FFFFu32));
        assert_eq!((0x0001u16, 0x0000u16), split(0x0001_0000u32));
        assert_eq!((0x0001u16, 0x0001u16), split(0x0001_0001u32));
        assert_eq!((0xFFFFu16, 0xFFFEu16), split(0xFFFF_FFFEu32));
        assert_eq!((0xFFFFu16, 0xFFFFu16), split(0xFFFF_FFFFu32));
    }

    #[test]
    fn test_join_u32() {
        assert_eq!(0x0000_0000u32, join(0x0000u16, 0x0000u16));
        assert_eq!(0x0000_0001u32, join(0x0000u16, 0x0001u16));
        assert_eq!(0x0000_FFFEu32, join(0x0000u16, 0xFFFEu16));
        assert_eq!(0x0000_FFFFu32, join(0x0000u16, 0xFFFFu16));
        assert_eq!(0x0001_0000u32, join(0x0001u16, 0x0000u16));
        assert_eq!(0x0001_0001u32, join(0x0001u16, 0x0001u16));
        assert_eq!(0xFFFF_FFFEu32, join(0xFFFFu16, 0xFFFEu16));
        assert_eq!(0xFFFF_FFFFu32, join(0xFFFFu16, 0xFFFFu16));
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_convert_range_to_inclusive() {
        assert_eq!(Ok(1..=5), convert_range_to_inclusive(1..6));
        assert_eq!(Ok(1..=u32::MAX), convert_range_to_inclusive(1..));
        assert_eq!(Ok(0..=u32::MAX), convert_range_to_inclusive(..));
        assert_eq!(Ok(16..=16), convert_range_to_inclusive(16..=16));
        assert_eq!(
            Ok(11..=19),
            convert_range_to_inclusive((Bound::Excluded(10), Bound::Excluded(20)))
        );

        assert_eq!(Err(ConvertRangeError::Empty), convert_range_to_inclusive(0..0));
        assert_eq!(Err(ConvertRangeError::Empty), convert_range_to_inclusive(5..5));
        assert_eq!(Err(ConvertRangeError::StartGreaterThanEnd), convert_range_to_inclusive(1..0));
        assert_eq!(Err(ConvertRangeError::StartGreaterThanEnd), convert_range_to_inclusive(10..5));
        assert_eq!(
            Err(ConvertRangeError::Empty),
            convert_range_to_inclusive((Bound::Excluded(u32::MAX), Bound::Included(u32::MAX)))
        );
        assert_eq!(
            Err(ConvertRangeError::StartAndEndEqualExcluded),
            convert_range_to_inclusive((Bound::Excluded(u32::MAX), Bound::Excluded(u32::MAX)))
        );
        assert_eq!(
            Err(ConvertRangeError::Empty),
            convert_range_to_inclusive((Bound::Excluded(0), Bound::Included(0)))
        );
    }

    #[test]
    fn test_sorted_inserts() {
        fn check(mut vec: Vec<u32>, values: Vec<u32>) {
            let mut expected = [vec.as_slice(), values.as_slice()].concat();
            expected.sort_unstable();

            let mut inserts = SortedInserts::new();
            for value in values {
                let index = vec.binary_search(&value).unwrap_err();
                inserts.insert(&mut vec, index, value);
            }
            inserts.finish(&mut vec);
            assert_eq!(vec, expected);
        }

        let even: Vec<u32> = (0..1000).map(|i| 2 * i).collect();
        let odd: Vec<u32> = (0..1000).map(|i| 2 * i + 1).collect();
        check(even.clone(), odd.clone());
        check(odd.clone(), even.clone());
        check(even.clone(), (0..5).map(|i| 400 * i + 1).collect());
        check(even.clone(), (2000..3000).collect());
        check(even[500..600].to_vec(), odd[..100].iter().chain(&odd[900..]).copied().collect());
        check(even[..10].to_vec(), odd[..100].to_vec());
        check(Vec::new(), odd.clone());
        check(even, Vec::new());
    }
}
