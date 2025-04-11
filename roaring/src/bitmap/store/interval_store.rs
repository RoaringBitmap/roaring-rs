#![allow(unused)]
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::RangeInclusive;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct IntervalStore(Vec<Interval>);

impl IntervalStore {
    pub fn new() -> Self {
        Self(Default::default())
    }

    #[inline]
    pub fn insert(&mut self, index: u16) -> bool {
        self.0
            .binary_search_by(|iv| cmp_index_interval(index, *iv).reverse())
            .map_err(|loc| {
                // loc may be equal to self.0.len()
                let loc_or_last = if loc < self.0.len() {
                    Some(loc)
                } else if !self.0.is_empty() {
                    Some(self.0.len() - 1)
                } else {
                    None
                };
                // There exists an interval at or before the location we should insert
                if let Some(loc_or_last) = loc_or_last {
                    if index == self.0[loc_or_last].end + 1 {
                        // index immediately follows an interval
                        // Checking for sandwiched intervals is not needed because of binary search loc
                        // i.e. when the index is sandwiched between two intervals we always
                        // get the right most interval, which puts us in the different if
                        self.0[loc_or_last].end += 1;
                    } else if index
                        .checked_add(1)
                        .map(|f| f == self.0[loc_or_last].start)
                        .unwrap_or(false)
                    {
                        // checked_add required for if u16::MAX is added
                        // Value immediately precedes interval
                        if loc > 0 && self.0[loc - 1].end == index - 1 {
                            // Merge with preceding interval
                            self.0[loc - 1].end = self.0[loc].end;
                            self.0.remove(loc);
                            return;
                        }
                        self.0[loc].start -= 1;
                    } else {
                        // The value stands alone
                        self.0.insert(loc, Interval::new(index, index));
                    }
                } else {
                    // there does not exist a single interval
                    self.0.insert(loc, Interval::new(index, index));
                }
            })
            .is_err()
    }

    fn drain_overlapping(&mut self, start_index: usize, interval: &Interval) -> u64 {
        let value = self.drain_overlapping_range(start_index, interval);
        if let Some(to_drain) = value.1 {
            self.0.drain(start_index..to_drain);
        }
        value.0
    }

    fn drain_overlapping_range(
        &mut self,
        start_index: usize,
        interval: &Interval,
    ) -> (u64, Option<usize>) {
        let mut drain_loc = None;
        let mut amount = 0;
        let mut intervals = self.0[start_index..].iter().enumerate().peekable();
        while let Some((i, cur_interval)) = intervals.next() {
            if !interval.contains_interval(cur_interval) {
                drain_loc = Some(start_index + i);
                break;
            }
            amount += cur_interval.run_len();
            if intervals.peek().is_none() {
                drain_loc = Some(start_index + i + 1);
            }
        }
        (amount, drain_loc)
    }

    #[inline]
    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let interval = Interval { start: *range.start(), end: *range.end() };
        let first_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.start, *iv).reverse());
        let end_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.end, *iv).reverse());
        match (first_interval, end_interval) {
            // both start and end index are contained in intervals
            (Ok(begin), Ok(end)) => {
                if begin == end {
                    return 0;
                }
                let drained_amount: u64 = self.0[begin + 1..end].iter().map(|f| f.run_len()).sum();
                let amount = Interval::new(self.0[begin].end + 1, self.0[end].start - 1).run_len()
                    - drained_amount;
                self.0[begin].end = self.0[end].end;
                self.0.drain(begin + 1..=end);
                amount
            }
            // start index is contained in an interval,
            // end index is not
            (Ok(begin), Err(to_insert)) => {
                let (new_end, drain_id) =
                    // if there is a next interval, check if these intervals are consecutive
                    if to_insert < self.0.len() && self.0[to_insert].start - 1 == interval.end {
                        // The intervals are consecutive! Adjust new end of interval, and how far
                        // we drain
                        (self.0[to_insert].start, to_insert + 1)
                    } else {
                        (interval.end, to_insert)
                    };
                let drained_amount: u64 =
                    self.0[begin + 1..to_insert].iter().map(|f| f.run_len()).sum();
                let amount =
                    Interval::new(self.0[begin].end + 1, interval.end).run_len() - drained_amount;
                self.0[begin].end = new_end;
                self.0.drain(begin + 1..drain_id);
                amount
            }
            // there is no interval that contains the start index,
            // there is an interval that contains the end index,
            (Err(to_begin), Ok(end)) => {
                let consecutive_begin =
                    to_begin > 0 && self.0[to_begin - 1].end + 1 == interval.start;
                let (drain_id, interval_id) =
                    // check if begin interval is consecutive with new interval
                    if consecutive_begin {
                        // The intervals are consecutive! Adjust how much we remove, and how
                        // which interval we end up keeping
                        (end + 1, to_begin - 1)
                    } else {
                        (end, end)
                    };
                let drained_amount: u64 = self.0[to_begin..end].iter().map(|f| f.run_len()).sum();
                let amount =
                    Interval::new(interval.start, self.0[end].start - 1).run_len() - drained_amount;
                if consecutive_begin {
                    self.0[interval_id].end = self.0[end].end;
                } else {
                    self.0[interval_id].start = interval.start;
                }
                self.0.drain(to_begin..drain_id);
                amount
            }
            (Err(to_begin), Err(to_end)) => {
                if self.0.is_empty() {
                    self.0.insert(to_begin, interval);
                    return interval.run_len();
                }
                let consec_begin = to_begin > 0 && self.0[to_begin - 1].end + 1 == interval.start;
                let conces_end = to_end < self.0.len()
                    && self.0[to_end]
                        .start
                        .checked_sub(1)
                        .map(|f| f == interval.end)
                        .unwrap_or(false);
                if !consec_begin && !conces_end && to_begin == to_end {
                    // an arbitrary range with no consecutive intervals, unable to reuse existing interval
                    self.0.insert(to_begin, interval);
                    return interval.run_len();
                }
                let (drain_id_begin, drain_id_end, interval_id) = {
                    if conces_end && consec_begin {
                        // Both intervals are consecutive! Adjust how much we remove, and
                        // which interval we end up keeping
                        //
                        // keep begin interval and remove end
                        // NOTE: to_begin - 1 since the interval we actually care about is one to
                        // the left e.g.:
                        // [3..=5, 9..=20] add 6..=8 ->
                        // to_begin = 1
                        // to_end = 1
                        (to_begin, to_end + 1, to_begin - 1)
                    } else if consec_begin {
                        // Remove end interval, keep begin to overwrite
                        //
                        // NOTE: to_begin - 1 since the interval we actually care about is one to
                        // the left e.g.:
                        // [3..=5] add 6..=8 ->
                        // to_begin = 1
                        // to_end = 1
                        (to_begin, to_end, to_begin - 1)
                    } else if conces_end {
                        // Remove begin interval, keep end to overwrite
                        //
                        // NOTE: no -1 since the interval we actually care about is one to
                        // the left e.g.:
                        // [8..=10] add 6..=7 ->
                        // to_begin = 0
                        // to_end = 1
                        (to_begin, to_end, to_end)
                    } else {
                        // keep end interval to overwrite
                        (
                            to_begin,
                            to_end.min(self.0.len() - 1),
                            if to_end != self.0.len() {
                                to_begin
                            } else {
                                to_end.min(self.0.len() - 1)
                            },
                        )
                    }
                };
                let drained_amount: u64 =
                    self.0[to_begin..to_end].iter().map(|f| f.run_len()).sum();
                let end_amount_interval =
                    if conces_end { self.0[to_end].start - 1 } else { interval.end };
                let amount =
                    Interval::new(interval.start, end_amount_interval).run_len() - drained_amount;
                let end_interval = if conces_end { self.0[to_end].end } else { interval.end };

                self.0[interval_id].end = end_interval;
                if !consec_begin {
                    self.0[interval_id].start = interval.start;
                }
                self.0.drain(drain_id_begin..drain_id_end);
                amount
            }
        }
    }

    pub fn push(&mut self, index: u16) -> bool {
        if let Some(last_interval) = self.0.last_mut() {
            if last_interval.end.checked_add(1).map(|f| f == index).unwrap_or(false) {
                last_interval.end = index;
                true
            } else if last_interval.end < index {
                self.0.push(Interval::new(index, index));
                true
            } else {
                false
            }
        } else {
            self.0.push(Interval::new(index, index));
            true
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        self.0
            .binary_search_by(|iv| cmp_index_interval(index, *iv).reverse())
            .map(|loc| {
                // loc always points to an interval
                let equal_to_start = self.0[loc].start == index;
                let equal_to_end = self.0[loc].end == index;
                if index == self.0[loc].start && index == self.0[loc].end {
                    // Remove entire run if it only contains this value
                    self.0.remove(loc);
                } else if index == self.0[loc].end {
                    // Value is last in this interval
                    self.0[loc].end = index - 1;
                } else if index == self.0[loc].start {
                    // Value is first in this interval
                    self.0[loc].start = index + 1;
                } else {
                    // Value lies inside the interval, we need to split it
                    // First construct a new interval with the right part
                    let new_interval = Interval::new(index + 1, self.0[loc].end);
                    // Then shrink the current interval
                    self.0[loc].end = index - 1;
                    // Then insert the new interval leaving gap where value was removed
                    self.0.insert(loc + 1, new_interval);
                }
            })
            .is_ok()
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        let interval = Interval { start: *range.start(), end: *range.end() };
        let first_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.start, *iv).reverse());
        let end_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.end, *iv).reverse());
        struct IdValue {
            index: usize,
            value: u16,
        }
        struct IntervalRange {
            drain_range: core::ops::Range<usize>,
            begin_value: Option<IdValue>,
            end_value: Option<IdValue>,
            residual_count: u64,
        }
        let todo = match (first_interval, end_interval) {
            // both start and end index are contained in intervals
            (Ok(begin), Ok(end)) => {
                if self.0[begin].start == interval.start && self.0[end].end == interval.end {
                    IntervalRange {
                        drain_range: begin..end + 1,
                        begin_value: None,
                        end_value: None,
                        residual_count: 0,
                    }
                } else if self.0[begin].start == interval.start {
                    IntervalRange {
                        drain_range: begin..end,
                        begin_value: None,
                        end_value: Some(IdValue { index: end, value: interval.end + 1 }),
                        residual_count: Interval::new(self.0[end].start, interval.end).run_len(),
                    }
                } else if self.0[end].end == interval.end {
                    IntervalRange {
                        drain_range: begin + 1..end + 1,
                        begin_value: Some(IdValue { index: begin, value: interval.start - 1 }),
                        end_value: None,
                        residual_count: Interval::new(interval.start, self.0[begin].end).run_len(),
                    }
                } else {
                    IntervalRange {
                        drain_range: begin + 1..end,
                        begin_value: Some(IdValue { index: begin, value: interval.start - 1 }),
                        end_value: Some(IdValue { index: end, value: interval.end + 1 }),
                        residual_count: Interval::new(self.0[end].start, interval.end).run_len()
                            + Interval::new(interval.start, self.0[begin].end).run_len(),
                    }
                }
            }
            // start index is contained in an interval,
            // end index is not
            (Ok(begin), Err(to_insert)) => {
                let end = if to_insert == self.0.len() { self.0.len() - 1 } else { to_insert };
                if self.0[begin].start == interval.start {
                    IntervalRange {
                        drain_range: begin..end,
                        begin_value: None,
                        end_value: None,
                        residual_count: 0,
                    }
                } else {
                    IntervalRange {
                        drain_range: begin + 1..end + 1,
                        begin_value: Some(IdValue { index: begin, value: interval.start - 1 }),
                        end_value: None,
                        residual_count: Interval::new(interval.start, self.0[begin].end).run_len(),
                    }
                }
            }
            // there is no interval that contains the start index,
            // there is an interval that contains the end index,
            (Err(begin), Ok(end)) => {
                if self.0[begin].end == interval.end {
                    IntervalRange {
                        drain_range: begin..end + 1,
                        begin_value: None,
                        end_value: None,
                        residual_count: 0,
                    }
                } else {
                    IntervalRange {
                        drain_range: begin..end,
                        begin_value: None,
                        end_value: Some(IdValue { index: end, value: interval.end + 1 }),
                        residual_count: Interval::new(self.0[end].start, interval.end).run_len(),
                    }
                }
            }
            (Err(begin), Err(to_end)) => {
                let end = if to_end == self.0.len() { self.0.len() - 1 } else { to_end };
                IntervalRange {
                    drain_range: begin..end + 1,
                    begin_value: None,
                    end_value: None,
                    residual_count: 0,
                }
            }
        };
        let count = self.0[todo.drain_range.clone()].iter().map(|f| f.run_len()).sum::<u64>()
            + todo.residual_count;
        if let Some(IdValue { index, value }) = todo.begin_value {
            self.0[index].end = value;
        }
        if let Some(IdValue { index, value }) = todo.end_value {
            self.0[index].start = value;
        }
        self.0.drain(todo.drain_range);
        count
    }

    pub fn remove_smallest(&mut self, mut amount: u64) {
        let mut remove_to = 0;
        let mut last_interval = None;
        for (i, interval) in self.0.iter_mut().enumerate() {
            let too_much = interval.run_len() < amount;
            if too_much {
                amount -= interval.run_len();
            }
            remove_to = i;
            last_interval = Some(interval);
            if !too_much {
                break;
            }
        }
        if let Some(last_interval) = last_interval {
            if last_interval.run_len() < amount {
                remove_to += 1;
            } else {
                last_interval.start += amount as u16;
            }
        }
        self.0.drain(..remove_to);
    }
}

/// This interval is inclusive to end.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub(crate) struct Interval {
    pub start: u16,
    pub end: u16,
}

impl IntoIterator for Interval {
    type Item = u16;
    type IntoIter = RangeInclusive<u16>;

    fn into_iter(self) -> Self::IntoIter {
        self.start..=self.end
    }
}

impl IntoIterator for &'_ Interval {
    type Item = u16;
    type IntoIter = RangeInclusive<u16>;

    fn into_iter(self) -> Self::IntoIter {
        self.start..=self.end
    }
}

pub(crate) fn cmp_index_interval(index: u16, iv: Interval) -> Ordering {
    if index < iv.start {
        Ordering::Less
    } else if index > iv.end {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

impl Interval {
    pub fn new(start: u16, end: u16) -> Interval {
        Interval { start, end }
    }

    pub fn contains_index(&self, value: u16) -> bool {
        self.start <= value && value <= self.end
    }

    pub fn contains_interval(&self, interval: &Interval) -> bool {
        self.start <= interval.start && interval.end <= self.end
    }

    pub fn run_len(&self) -> u64 {
        u64::from(self.end - self.start) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_empty() {
        let mut interval_store = IntervalStore(alloc::vec![]);
        assert!(interval_store.insert(1));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 1, end: 1 }]))
    }

    #[test]
    fn insert_consecutive_begin() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 0, end: 0 },]);
        assert!(interval_store.insert(1));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 0, end: 1 }]))
    }

    #[test]
    fn insert_consecutive_end() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 1, end: 1 },]);
        assert!(interval_store.insert(0));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 0, end: 1 }]))
    }

    #[test]
    fn insert_consecutive_begin_end() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 0, end: 0 },
            Interval { start: 2, end: 2 },
        ]);
        interval_store.insert(1);
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 0, end: 2 }]))
    }

    #[test]
    fn insert_arbitrary() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 0, end: 3 },
            Interval { start: 9, end: 10 },
        ]);
        interval_store.insert(5);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 0, end: 3 },
                Interval { start: 5, end: 5 },
                Interval { start: 9, end: 10 },
            ])
        )
    }

    #[test]
    fn insert_u16_max() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 0, end: 3 },]);
        interval_store.insert(u16::MAX);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 0, end: 3 },
                Interval { start: u16::MAX, end: u16::MAX },
            ])
        )
    }

    #[test]
    fn insert_u16_max_consecutive() {
        let mut interval_store =
            IntervalStore(alloc::vec![Interval { start: 0, end: u16::MAX - 1 },]);
        interval_store.insert(u16::MAX);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval { start: 0, end: u16::MAX },])
        )
    }

    #[test]
    fn insert_range_empty() {
        let mut interval_store = IntervalStore(alloc::vec![]);
        assert_eq!(interval_store.insert_range(1..=2), Interval::new(1, 2).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 1, end: 2 },]));
    }

    #[test]
    fn insert_range_overlap_begin() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 1, end: 20 }]);
        assert_eq!(interval_store.insert_range(5..=50), Interval::new(21, 50).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 1, end: 50 },]));
    }

    #[test]
    fn insert_range_overlap_end() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 10, end: 20 }]);
        assert_eq!(interval_store.insert_range(5..=15), Interval::new(5, 9).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 5, end: 20 },]));
    }

    #[test]
    fn insert_range_overlap_begin_end() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 40, end: 60 },
        ]);
        assert_eq!(interval_store.insert_range(15..=50), Interval::new(21, 39).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 10, end: 60 },]));
    }

    #[test]
    fn insert_range_concescutive_begin() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 10, end: 20 },]);
        assert_eq!(interval_store.insert_range(21..=50), Interval::new(21, 50).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 10, end: 50 },]));
    }

    #[test]
    fn insert_range_concescutive_end() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 70 },]);
        assert_eq!(interval_store.insert_range(21..=49), Interval::new(21, 49).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 21, end: 70 },]));
    }

    #[test]
    fn insert_range_concescutive_begin_end() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
        ]);
        assert_eq!(interval_store.insert_range(21..=49), Interval::new(21, 49).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 10, end: 70 },]));
    }

    #[test]
    fn insert_range_no_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
        ]);
        assert_eq!(interval_store.insert_range(25..=30), Interval::new(25, 30).run_len());
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 10, end: 20 },
                Interval { start: 25, end: 30 },
                Interval { start: 50, end: 70 },
            ])
        );
    }

    #[test]
    fn insert_range_u16_max_no_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
        ]);
        assert_eq!(
            interval_store.insert_range(90..=u16::MAX),
            Interval::new(90, u16::MAX).run_len()
        );
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 10, end: 20 },
                Interval { start: 50, end: 70 },
                Interval { start: 90, end: u16::MAX },
            ])
        );
    }

    #[test]
    fn insert_range_u16_max_overlap_begin() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
        ]);
        assert_eq!(
            interval_store.insert_range(70..=u16::MAX),
            Interval::new(71, u16::MAX).run_len()
        );
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 10, end: 20 },
                Interval { start: 50, end: u16::MAX },
            ])
        );
    }

    #[test]
    fn insert_range_u16_max_overlap_all() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
        ]);
        assert_eq!(
            interval_store.insert_range(0..=u16::MAX),
            Interval::new(0, u16::MAX).run_len()
                - Interval::new(10, 20).run_len()
                - Interval::new(50, 70).run_len()
        );
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval { start: 0, end: u16::MAX },])
        );
    }

    #[test]
    fn push_new_max() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 70 },]);
        assert!(interval_store.push(80));
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 50, end: 70 },
                Interval { start: 80, end: 80 },
            ])
        );
    }

    #[test]
    fn push_new_max_consecutive() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 70 },]);
        assert!(interval_store.push(71));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 50, end: 71 },]));
    }

    #[test]
    fn push_existing() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 70 },]);
        assert!(!interval_store.push(60));
        assert_eq!(interval_store, interval_store);
    }

    #[test]
    fn push_non_existing_non_max() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 70 },]);
        assert!(!interval_store.push(10));
        assert_eq!(interval_store, interval_store);
    }

    #[test]
    fn push_existing_u16_max() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: u16::MAX },]);
        assert!(!interval_store.push(u16::MAX));
        assert_eq!(interval_store, interval_store);
    }

    #[test]
    fn push_new_u16_max() {
        let mut interval_store =
            IntervalStore(alloc::vec![Interval { start: 50, end: u16::MAX - 1 },]);
        assert!(interval_store.push(u16::MAX));
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval { start: 50, end: u16::MAX },])
        );
    }

    #[test]
    fn remove_end_of_interval() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 60 },]);
        assert!(interval_store.remove(60));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 50, end: 59 },]));
    }

    #[test]
    fn remove_begin_of_interval() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 60 },]);
        assert!(interval_store.remove(50));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 51, end: 60 },]));
    }

    #[test]
    fn remove_middle() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 1, end: 3 },]);
        assert!(interval_store.remove(2));
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 1, end: 1 },
                Interval { start: 3, end: 3 },
            ])
        );
    }

    #[test]
    fn remove_nothing() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 60 },]);
        assert!(!interval_store.remove(90));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 50, end: 60 },]));
    }

    #[test]
    fn remove_u16_max() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: u16::MAX },]);
        assert!(interval_store.remove(u16::MAX));
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval { start: 50, end: u16::MAX - 1 },])
        );
    }

    #[test]
    fn remove_range_exact_one() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(40..=60), 21);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_exact_many() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 80, end: 90 },
            Interval { start: 100, end: 200 },
        ]);
        assert_eq!(
            interval_store.remove_range(40..=200),
            Interval::new(40, 60).run_len()
                + Interval::new(80, 90).run_len()
                + Interval::new(100, 200).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_begin_exact_overlap_end_one() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 70, end: 90 },
        ]);
        assert_eq!(
            interval_store.remove_range(40..=80),
            Interval::new(40, 60).run_len() + Interval::new(70, 80).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 81, end: 90 },]));
    }

    #[test]
    fn remove_range_begin_overlap_end_exact_one() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 70, end: 90 },
        ]);
        assert_eq!(
            interval_store.remove_range(50..=90),
            Interval::new(70, 90).run_len() + Interval::new(50, 60).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 40, end: 49 },]));
    }

    #[test]
    fn remove_range_both_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 70, end: 90 },
        ]);
        assert_eq!(
            interval_store.remove_range(50..=80),
            Interval::new(70, 80).run_len() + Interval::new(50, 60).run_len()
        );
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 40, end: 49 },
                Interval { start: 81, end: 90 },
            ])
        );
    }

    #[test]
    fn remove_range_begin_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(50..=100), Interval::new(50, 60).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 40, end: 49 },]));
    }

    #[test]
    fn remove_range_begin_overlap_many() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 80, end: 100 },
            Interval { start: 200, end: 500 },
        ]);
        assert_eq!(
            interval_store.remove_range(50..=1000),
            Interval::new(50, 60).run_len()
                + Interval::new(80, 100).run_len()
                + Interval::new(200, 500).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 40, end: 49 },]));
    }

    #[test]
    fn remove_range_end_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(20..=50), Interval::new(40, 50).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 51, end: 60 },]));
    }

    #[test]
    fn remove_range_end_overlap_many() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 100, end: 500 },
            Interval { start: 800, end: 900 },
        ]);
        assert_eq!(
            interval_store.remove_range(20..=850),
            Interval::new(40, 60).run_len()
                + Interval::new(100, 500).run_len()
                + Interval::new(800, 850).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 851, end: 900 },]));
    }

    #[test]
    fn remove_range_no_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(20..=80), Interval::new(40, 60).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_no_overlap_many() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 400, end: 600 },
            Interval { start: 4000, end: 6000 },
        ]);
        assert_eq!(
            interval_store.remove_range(20..=60000),
            Interval::new(40, 60).run_len()
                + Interval::new(400, 600).run_len()
                + Interval::new(4000, 6000).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_smallest_one() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        interval_store.remove_smallest(500);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_smallest_many_1() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 0, end: 99 },
            Interval { start: 400, end: 600 },
            Interval { start: 4000, end: 6000 },
        ]);
        interval_store.remove_smallest(200);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval::new(500, 600), Interval::new(4000, 6000),])
        );
    }

    #[test]
    fn remove_smallest_many_2() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 0, end: 99 },
            Interval { start: 400, end: 599 },
            Interval { start: 4000, end: 6000 },
        ]);
        interval_store.remove_smallest(500);
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval::new(4200, 6000),]));
    }
}
