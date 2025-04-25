use alloc::vec::Vec;
use core::ops::{
    BitAnd, BitAndAssign, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, SubAssign,
};
use core::slice::Iter;
use core::{cmp::Ordering, ops::ControlFlow};

use super::{ArrayStore, BitmapStore};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct IntervalStore(Vec<Interval>);

pub(crate) const RUN_NUM_BYTES: usize = 2;
pub(crate) const RUN_ELEMENT_BYTES: usize = 4;

impl Default for IntervalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl IntervalStore {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn full() -> Self {
        Self(alloc::vec![Interval::new(0, u16::MAX)])
    }

    pub fn byte_size(&self) -> usize {
        RUN_NUM_BYTES + (RUN_ELEMENT_BYTES * self.run_amount() as usize)
    }

    #[cfg(feature = "std")]
    pub fn from_vec_unchecked(vec: Vec<Interval>) -> Self {
        #[cfg(debug_assertions)]
        {
            for win in vec.windows(2) {
                let [cur_interval, next] = [win[0], win[1]];
                assert!(cur_interval.end + 1 < next.start);
                assert!(cur_interval.start <= cur_interval.end);
            }
        }
        Self(vec)
    }

    pub(crate) fn push_interval_unchecked(&mut self, interval: Interval) {
        debug_assert!(self.0.last().map(|f| f.end < interval.start).unwrap_or(true));
        debug_assert!(interval.start <= interval.end);
        self.0.push(interval)
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
                    if Some(index) == self.0[loc_or_last].end.checked_add(1) {
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
                    } else if loc_or_last
                        .checked_sub(1)
                        .map(|f| self.0[f].end == index - 1)
                        .unwrap_or(false)
                    {
                        // We are sandwiched between 2 intervals, but the previous interval is
                        // continuous with the index. If the loc
                        self.0[loc_or_last - 1].end = index;
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

    #[inline]
    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        if range.is_empty() {
            return 0;
        }
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
                        (self.0[to_insert].end, to_insert + 1)
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
                        // keep end interval to overwrite if it exists,
                        // otherwise overwrite begin interval
                        (
                            if to_end != self.0.len() { to_begin + 1 } else { to_begin },
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
        if range.is_empty() {
            return 0;
        }

        let interval = Interval::new(*range.start(), *range.end());
        let first_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.start, *iv).reverse());
        let end_interval =
            self.0.binary_search_by(|iv| cmp_index_interval(interval.end, *iv).reverse());
        match (first_interval, end_interval) {
            // both start and end index are contained in intervals
            (Ok(first), Ok(end)) => {
                if self.0[first].start == interval.start && self.0[end].end == interval.end {
                    let removed = self.0[first..=end].iter().map(|iv| iv.run_len()).sum();
                    self.0.drain(first..=end);
                    removed
                } else if self.0[first].start == interval.start {
                    if first == end {
                        self.0[end].start = interval.end + 1;
                        return interval.run_len();
                    }
                    let removed = self.0[first..end].iter().map(|iv| iv.run_len()).sum::<u64>()
                        + Interval::new(self.0[end].start, interval.end).run_len();
                    self.0[end].start = interval.end + 1;
                    self.0.drain(first..end);
                    removed
                } else if self.0[end].end == interval.end {
                    if first == end {
                        self.0[end].end = interval.start - 1;
                        return interval.run_len();
                    }
                    let removed =
                        self.0[first + 1..=end].iter().map(|iv| iv.run_len()).sum::<u64>()
                            + Interval::new(interval.start, self.0[first].end).run_len();
                    self.0[first].end = interval.start - 1;
                    self.0.drain(first + 1..=end);
                    removed
                } else {
                    if first == end {
                        let old_end = self.0[first].end;
                        self.0[first].end = interval.start - 1;
                        self.0.insert(first + 1, Interval::new(interval.end + 1, old_end));
                        return interval.run_len();
                    }

                    let removed = self.0[first + 1..end].iter().map(|iv| iv.run_len()).sum::<u64>()
                        + Interval::new(interval.start, self.0[first].end).run_len()
                        + Interval::new(self.0[end].start, interval.end).run_len();
                    self.0[first].end = interval.start - 1;
                    self.0[end].start = interval.end + 1;
                    removed
                }
            }
            // start index is in the interval store, end index is not
            (Ok(first), Err(end)) => {
                debug_assert!(first < end);
                if self.0[first].start == interval.start {
                    let removed = self.0[first..end].iter().map(|iv| iv.run_len()).sum();
                    self.0.drain(first..end);
                    removed
                } else {
                    let removed = self.0[first + 1..end].iter().map(|iv| iv.run_len()).sum::<u64>()
                        + Interval::new(interval.start, self.0[first].end).run_len();
                    self.0[first].end = interval.start - 1;
                    self.0.drain(first + 1..end);
                    removed
                }
            }
            // end index is in the interval store, start index is not
            (Err(first), Ok(end)) => {
                if self.0[end].end == interval.end {
                    let removed = self.0[first..=end].iter().map(|iv| iv.run_len()).sum();
                    self.0.drain(first..=end);
                    removed
                } else {
                    let removed = self.0[first..end].iter().map(|iv| iv.run_len()).sum::<u64>()
                        + Interval::new(self.0[end].start, interval.end).run_len();
                    self.0[end].start = interval.end + 1;
                    self.0.drain(first..end);
                    removed
                }
            }
            // both indices are not contained in the interval store
            (Err(first), Err(end)) => {
                if first == end {
                    return 0;
                }
                let removed = self.0[first..end].iter().map(|iv| iv.run_len()).sum();
                self.0.drain(first..end);
                removed
            }
        }
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

    pub fn remove_biggest(&mut self, mut amount: u64) {
        let mut remove_to = 0;
        let mut last_interval = None;
        for (i, interval) in self.0.iter_mut().enumerate().rev() {
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
            if last_interval.run_len() >= amount {
                remove_to += 1;
                last_interval.end -= amount as u16;
            }
        }
        self.0.drain(remove_to..);
    }

    pub fn contains(&self, index: u16) -> bool {
        self.0.binary_search_by(|iv| cmp_index_interval(index, *iv).reverse()).is_ok()
    }

    pub fn contains_range(&self, range: RangeInclusive<u16>) -> bool {
        let interval = Interval::new(*range.start(), *range.end());
        let start = self.0.binary_search_by(|iv| cmp_index_interval(interval.start, *iv).reverse());
        let end = self.0.binary_search_by(|iv| cmp_index_interval(interval.end, *iv).reverse());
        match (start, end) {
            // both start and end are inside an interval,
            // check if this interval is that same interval.
            // If this is not the case then this range is not contained in this store
            (Ok(start_id), Ok(end_id)) => start_id == end_id,
            _ => false,
        }
    }

    fn step_walk<
        'a,
        R,
        C: FnMut(Interval, Interval, R) -> ControlFlow<R, R>,
        E: FnMut(
            (Option<Interval>, Option<Interval>),
            (Iter<'a, Interval>, Iter<'a, Interval>),
            R,
        ) -> R,
    >(
        &'a self,
        other: &'a Self,
        mut calc: C,
        mut else_op: E,
        mut buffer: R,
    ) -> R {
        let (mut i1, mut i2) = (self.0.iter(), other.0.iter());
        let (mut iv1, mut iv2) = (i1.next(), i2.next());
        loop {
            match (iv1, iv2) {
                (Some(v1), Some(v2)) => {
                    match calc(*v1, *v2, buffer) {
                        ControlFlow::Continue(new_buffer) => buffer = new_buffer,
                        ControlFlow::Break(end) => return end,
                    }

                    // We increase the iterator based on which one is furthest behind.
                    // Or both if they are equal to each other.
                    match v1.end.cmp(&v2.end) {
                        Ordering::Less => iv1 = i1.next(),
                        Ordering::Greater => iv2 = i2.next(),
                        Ordering::Equal => {
                            iv1 = i1.next();
                            iv2 = i2.next();
                        }
                    }
                }
                (value1, value2) => {
                    return else_op((value1.copied(), value2.copied()), (i1, i2), buffer)
                }
            }
        }
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.step_walk(
            other,
            |interval1, interval2, _| {
                if interval1.overlaps(&interval2) {
                    ControlFlow::Break(false)
                } else {
                    ControlFlow::Continue(true)
                }
            },
            |_, _, _| true,
            false,
        )
    }

    pub(crate) fn is_disjoint_array(&self, array: &ArrayStore) -> bool {
        array.iter().all(|&i| !self.contains(i))
    }

    pub(crate) fn is_disjoint_bitmap(&self, array: &BitmapStore) -> bool {
        // TODO: make this better
        array.iter().all(|i| !self.contains(i))
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.iter().all(|interval| other.contains_range(interval.start..=interval.end))
    }

    pub(crate) fn is_subset_array(&self, other: &ArrayStore) -> bool {
        self.0.iter().all(|interval| other.contains_range(interval.start..=interval.end))
    }

    pub(crate) fn is_subset_bitmap(&self, other: &BitmapStore) -> bool {
        self.0.iter().all(|interval| other.contains_range(interval.start..=interval.end))
    }

    pub fn intersection_len(&self, other: &Self) -> u64 {
        self.step_walk(
            other,
            |interval1, interval2, buffer| {
                ControlFlow::Continue(
                    interval1.overlapping_interval(&interval2).map(|f| f.run_len()).unwrap_or(0)
                        + buffer,
                )
            },
            |_, _, buffer| buffer,
            0,
        )
    }

    pub(crate) fn intersection_len_bitmap(&self, other: &BitmapStore) -> u64 {
        self.0.iter().map(|f| other.intersection_len_interval(f)).sum()
    }

    pub(crate) fn intersection_len_array(&self, other: &ArrayStore) -> u64 {
        other.iter().map(|&f| self.contains(f) as u64).sum()
    }

    pub fn len(&self) -> u64 {
        self.0.iter().map(|iv| iv.run_len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn min(&self) -> Option<u16> {
        self.0.first().map(|f| f.start)
    }

    pub fn max(&self) -> Option<u16> {
        self.0.last().map(|f| f.end)
    }

    pub fn rank(&self, value: u16) -> u64 {
        let mut rank = 0;
        for iv in self.0.iter() {
            if iv.end <= value {
                rank += iv.run_len();
            } else if iv.start <= value {
                rank += Interval::new(iv.start, value).run_len();
            } else {
                break;
            }
        }
        rank
    }

    pub fn select(&self, mut n: u16) -> Option<u16> {
        for iv in self.0.iter() {
            let run_len = iv.run_len() as u16;
            if run_len <= n {
                n -= iv.run_len() as u16;
            } else {
                return Some(iv.start + n);
            }
        }
        None
    }

    pub fn run_amount(&self) -> u64 {
        self.0.len() as u64
    }

    pub fn to_bitmap(&self) -> BitmapStore {
        let mut bits = BitmapStore::new();
        for iv in self.0.iter() {
            bits.insert_range(iv.start..=iv.end);
        }
        bits
    }

    pub(crate) fn iter(&self) -> RunIterBorrowed {
        self.into_iter()
    }

    pub(crate) fn iter_intervals(&self) -> core::slice::Iter<Interval> {
        self.0.iter()
    }
}

impl BitOrAssign for IntervalStore {
    fn bitor_assign(&mut self, mut rhs: Self) {
        let (add_intervals, take_intervals, self_is_add) =
            if self.len() > rhs.len() { (self, &mut rhs, true) } else { (&mut rhs, self, false) };
        for iv in take_intervals.iter_intervals() {
            add_intervals.insert_range(iv.start..=iv.end);
        }
        if !self_is_add {
            core::mem::swap(add_intervals, take_intervals);
        }
    }
}

impl BitOrAssign<&ArrayStore> for IntervalStore {
    fn bitor_assign(&mut self, rhs: &ArrayStore) {
        for &i in rhs.iter() {
            self.insert(i);
        }
    }
}

impl BitOrAssign<&Self> for IntervalStore {
    fn bitor_assign(&mut self, rhs: &Self) {
        for iv in rhs.iter_intervals() {
            self.insert_range(iv.start..=iv.end);
        }
    }
}

impl BitAnd for &IntervalStore {
    type Output = IntervalStore;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.step_walk(
            rhs,
            |iv1, iv2, mut buf: IntervalStore| {
                if let Some(new_iv) = iv1.overlapping_interval(&iv2) {
                    buf.insert_range(new_iv.start..=new_iv.end);
                }
                ControlFlow::Continue(buf)
            },
            |_, _, buf| buf,
            IntervalStore::new(),
        )
    }
}

impl BitAndAssign<&IntervalStore> for ArrayStore {
    fn bitand_assign(&mut self, rhs: &IntervalStore) {
        self.retain(|f| rhs.contains(f));
    }
}

impl SubAssign<&Self> for IntervalStore {
    fn sub_assign(&mut self, rhs: &Self) {
        for iv in rhs.iter_intervals() {
            self.remove_range(iv.start..=iv.end);
        }
    }
}

impl BitXor for &IntervalStore {
    type Output = IntervalStore;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut union = self.clone();
        union |= rhs;
        let intersection = self & rhs;
        union -= &intersection;
        union
    }
}

impl BitXorAssign<&ArrayStore> for IntervalStore {
    fn bitxor_assign(&mut self, rhs: &ArrayStore) {
        rhs.iter().for_each(|&f| {
            if self.contains(f) {
                self.remove(f);
            } else {
                self.insert(f);
            }
        })
    }
}

pub(crate) type RunIterOwned = RunIter<alloc::vec::IntoIter<Interval>>;
pub(crate) type RunIterBorrowed<'a> = RunIter<core::slice::Iter<'a, Interval>>;

impl IntoIterator for IntervalStore {
    type Item = u16;
    type IntoIter = RunIter<alloc::vec::IntoIter<Interval>>;

    fn into_iter(self) -> Self::IntoIter {
        RunIter::new(self.0.into_iter())
    }
}

impl<'a> IntoIterator for &'a IntervalStore {
    type Item = u16;
    type IntoIter = RunIter<core::slice::Iter<'a, Interval>>;

    fn into_iter(self) -> Self::IntoIter {
        RunIter::new(self.0.iter())
    }
}

pub(crate) trait SliceIterator<I>: Iterator + DoubleEndedIterator {
    fn as_slice(&self) -> &[I];
}

impl<I> SliceIterator<I> for alloc::vec::IntoIter<I> {
    fn as_slice(&self) -> &[I] {
        alloc::vec::IntoIter::as_slice(self)
    }
}

impl<'a, I> SliceIterator<I> for core::slice::Iter<'a, I> {
    fn as_slice(&self) -> &'a [I] {
        core::slice::Iter::as_slice(self)
    }
}

#[derive(Clone)]
pub(crate) struct RunIter<I: SliceIterator<Interval>> {
    forward_offset: u16,
    backward_offset: u16,
    intervals: I,
}

impl<I: SliceIterator<Interval>> RunIter<I> {
    fn new(intervals: I) -> Self {
        Self { forward_offset: 0, backward_offset: 0, intervals }
    }

    fn move_next(&mut self) {
        if let Some(value) = self.forward_offset.checked_add(1) {
            self.forward_offset = value;
        } else {
            return;
        }
        if Some(self.forward_offset as u64)
            >= self.intervals.as_slice().first().map(|f| f.run_len())
        {
            self.intervals.next();
            self.forward_offset = 0;
        }
    }

    fn move_next_back(&mut self) {
        if let Some(value) = self.backward_offset.checked_add(1) {
            self.backward_offset = value;
        } else {
            return;
        }
        if Some(self.backward_offset as u64)
            >= self.intervals.as_slice().last().map(|f| f.run_len())
        {
            self.intervals.next_back();
            self.backward_offset = 0;
        }
    }

    fn remaining_size(&self) -> usize {
        (self.intervals.as_slice().iter().map(|f| f.run_len()).sum::<u64>()
            - self.forward_offset as u64
            - self.backward_offset as u64) as usize
    }

    /// Advance the iterator to the first value greater than or equal to `n`.
    pub(crate) fn advance_to(&mut self, n: u16) {
        if n == 0 {
            return;
        }
        if self
            .intervals
            .as_slice()
            .first()
            .map(|f| f.start + self.forward_offset > n)
            .unwrap_or(true)
        {
            return;
        }
        match self.intervals.as_slice().binary_search_by(|iv| cmp_index_interval(n, *iv).reverse())
        {
            Ok(index) => {
                if let Some(value) = index.checked_sub(1) {
                    self.intervals.nth(value);
                }
                self.forward_offset = n - self.intervals.as_slice().first().unwrap().start;
            }
            Err(index) => {
                if index == self.intervals.as_slice().len() {
                    return;
                }
                if let Some(value) = index.checked_sub(1) {
                    self.intervals.nth(value);
                    self.forward_offset = 0;
                }
            }
        }
    }

    /// Advance the back of iterator to the first value less than or equal to `n`.
    pub(crate) fn advance_back_to(&mut self, n: u16) {
        if n == u16::MAX {
            return;
        }
        if self
            .intervals
            .as_slice()
            .last()
            .map(|f| f.end - self.backward_offset < n)
            .unwrap_or(true)
        {
            return;
        }
        match self.intervals.as_slice().binary_search_by(|iv| cmp_index_interval(n, *iv).reverse())
        {
            Ok(index) => {
                let backward_index = self.intervals.as_slice().len() - index - 1;
                if let Some(value) = backward_index.checked_sub(1) {
                    self.intervals.nth_back(value);
                }
                self.backward_offset = self.intervals.as_slice().last().unwrap().end - n;
            }
            Err(index) => {
                if index == 0 {
                    return;
                }
                let backward_index = self.intervals.as_slice().len() - index;
                if let Some(value) = backward_index.checked_sub(1) {
                    self.intervals.nth_back(value);
                    self.backward_offset = 0;
                }
            }
        }
    }
}

impl<I: SliceIterator<Interval>> Iterator for RunIter<I> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        let result = self.intervals.as_slice().first()?.start + self.forward_offset;
        self.move_next();
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_size = self.remaining_size();
        (remaining_size, Some(remaining_size))
    }

    fn count(self) -> usize {
        self.remaining_size()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if let Some(skip) = n.checked_sub(1) {
            let mut to_skip = skip as u64;
            loop {
                let to_remove = (self.intervals.as_slice().first()?.run_len()
                    - self.forward_offset as u64)
                    .min(to_skip);
                to_skip -= to_remove;
                self.forward_offset += to_remove as u16;
                self.move_next();
                if to_skip == 0 {
                    break;
                }
            }
        }
        self.next()
    }
}

impl<I: SliceIterator<Interval>> DoubleEndedIterator for RunIter<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let result = self.intervals.as_slice().last()?.end - self.backward_offset;
        self.move_next_back();
        Some(result)
    }
}

impl<I: SliceIterator<Interval>> ExactSizeIterator for RunIter<I> {}

/// This interval is inclusive to end.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub(crate) struct Interval {
    pub start: u16,
    pub end: u16,
}

impl From<RangeInclusive<u16>> for Interval {
    fn from(value: RangeInclusive<u16>) -> Self {
        Interval::new(*value.start(), *value.end())
    }
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

    pub fn overlaps(&self, interval: &Interval) -> bool {
        interval.start <= self.end && self.start <= interval.end
    }

    pub fn overlapping_interval(&self, other: &Interval) -> Option<Interval> {
        if self.overlaps(other) {
            Some(Interval::new(self.start.max(other.start), self.end.min(other.end)))
        } else {
            None
        }
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
    fn insert_consecutive_end_with_extra() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 65079, end: 65079 },
            Interval { start: 65179, end: 65179 },
        ]);
        assert!(interval_store.insert(65080));
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 65079, end: 65080 },
                Interval { start: 65179, end: 65179 },
            ])
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
    fn insert_range_concescutive_begin_overlap_end() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 40, end: 60 },
        ]);
        assert_eq!(interval_store.insert_range(21..=50), Interval::new(21, 39).run_len());
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 10, end: 60 },]));
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
    fn insert_range_overlap_some() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 10, end: 20 },
            Interval { start: 50, end: 70 },
            Interval { start: 500, end: 700 },
        ]);
        assert_eq!(
            interval_store.insert_range(0..=100),
            Interval::new(0, 100).run_len()
                - Interval::new(10, 20).run_len()
                - Interval::new(50, 70).run_len()
        );
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval { start: 0, end: 100 },
                Interval { start: 500, end: 700 },
            ])
        );
    }

    #[test]
    fn insert_range_begin_overlap_concescutive_end() {
        let mut interval_store =
            IntervalStore(alloc::vec![Interval::new(2, 10), Interval::new(12, 700),]);
        assert_eq!(interval_store.insert_range(2..=11), 1);
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval::new(2, 700)]));
    }

    #[test]
    fn insert_range_pin_1() {
        let mut interval_store = IntervalStore(alloc::vec![Interval::new(65079, 65079)]);
        assert_eq!(interval_store.insert_range(65080..=65080), 1);
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval::new(65079, 65080)]));
    }

    #[test]
    fn push_empty() {
        let mut interval_store = IntervalStore(alloc::vec![]);
        assert!(interval_store.push(80));
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval { start: 80, end: 80 },]));
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
    fn remove_interval() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 50, end: 50 },]);
        assert!(interval_store.remove(50));
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_exact_one() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(40..=60), 21);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_one_with_extra_1() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        assert_eq!(interval_store.remove_range(40..=70), 21);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_one_with_extra_2() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 400, end: 600 },
            Interval { start: 4000, end: 6000 },
        ]);
        assert_eq!(interval_store.remove_range(40..=70), 21);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval::new(400, 600), Interval::new(4000, 6000),])
        );
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
    fn remove_range_begin_no_overlap_end_exact_one_1() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 70, end: 90 },
        ]);
        assert_eq!(
            interval_store.remove_range(30..=90),
            Interval::new(70, 90).run_len() + Interval::new(40, 60).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_begin_no_overlap_end_exact_one_2() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 40, end: 60 },
            Interval { start: 70, end: 90 },
            Interval { start: 700, end: 900 },
        ]);
        assert_eq!(
            interval_store.remove_range(30..=90),
            Interval::new(70, 90).run_len() + Interval::new(40, 60).run_len()
        );
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval::new(700, 900),]));
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
    fn remove_range_complete_overlap() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 51, end: 6000 },]);
        assert_eq!(interval_store.remove_range(500..=600), Interval::new(500, 600).run_len());
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval::new(51, 499), Interval::new(601, 6000),])
        );
    }

    #[test]
    fn remove_range_nothing() {
        let mut interval_store = IntervalStore(alloc::vec![]);
        assert_eq!(interval_store.remove_range(50000..=60000), 0);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_range_with_extra() {
        let mut interval_store =
            IntervalStore(alloc::vec![Interval::new(38161, 38162), Interval::new(40562, 40562),]);
        assert_eq!(interval_store.remove_range(38162..=38163), 1);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval::new(38161, 38161), Interval::new(40562, 40562),])
        );
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

    #[test]
    fn remove_biggest_one() {
        let mut interval_store = IntervalStore(alloc::vec![Interval { start: 40, end: 60 },]);
        interval_store.remove_biggest(500);
        assert_eq!(interval_store, IntervalStore(alloc::vec![]));
    }

    #[test]
    fn remove_biggest_many_1() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 0, end: 99 },
            Interval { start: 400, end: 600 },
            Interval { start: 5901, end: 6000 },
        ]);
        interval_store.remove_biggest(200);
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![Interval::new(0, 99), Interval::new(400, 500),])
        );
    }

    #[test]
    fn remove_biggest_many_2() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 6000 },
            Interval { start: 8401, end: 8600 },
            Interval { start: 9901, end: 10000 },
        ]);
        interval_store.remove_biggest(500);
        assert_eq!(interval_store, IntervalStore(alloc::vec![Interval::new(1, 5800),]));
    }

    #[test]
    fn contains_index_1() {
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(interval_store.contains(5));
        assert!(interval_store.contains(16000));
    }

    #[test]
    fn contains_index_2() {
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(!interval_store.contains(0));
    }

    #[test]
    fn contains_range_1() {
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(interval_store.contains_range(1..=500));
    }

    #[test]
    fn contains_range_2() {
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(!interval_store.contains_range(1..=1500));
    }

    #[test]
    fn contains_range_3() {
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(interval_store.contains_range(1..=1));
    }

    #[test]
    fn is_disjoint_1() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval { start: 601, end: 1200 },]);
        assert!(!interval_store_1.is_disjoint(&interval_store_1));
        assert!(!interval_store_2.is_disjoint(&interval_store_2));
        assert!(interval_store_1.is_disjoint(&interval_store_2));
        assert!(interval_store_2.is_disjoint(&interval_store_1));
    }

    #[test]
    fn is_disjoint_2() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval { start: 600, end: 1200 },]);
        assert!(!interval_store_1.is_disjoint(&interval_store_1));
        assert!(!interval_store_2.is_disjoint(&interval_store_2));
        assert!(!interval_store_1.is_disjoint(&interval_store_2));
        assert!(!interval_store_2.is_disjoint(&interval_store_1));
    }

    #[test]
    fn is_disjoint_3() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval { start: 15800, end: 15905 },]);
        assert!(!interval_store_1.is_disjoint(&interval_store_1));
        assert!(!interval_store_2.is_disjoint(&interval_store_2));
        assert!(!interval_store_1.is_disjoint(&interval_store_2));
        assert!(!interval_store_2.is_disjoint(&interval_store_1));
    }

    #[test]
    fn is_disjoint_array_store_1() {
        let array_store = ArrayStore::from_vec_unchecked(alloc::vec![0, 60, 200, 500,]);
        let interval_store = IntervalStore(alloc::vec![Interval { start: 70, end: 199 },]);
        assert!(interval_store.is_disjoint_array(&array_store));
    }

    #[test]
    fn is_disjoint_array_store_2() {
        let array_store = ArrayStore::from_vec_unchecked(alloc::vec![0, 60, 200, 500,]);
        let interval_store = IntervalStore(alloc::vec![Interval { start: 1, end: 400 },]);
        assert!(!interval_store.is_disjoint_array(&array_store));
    }

    #[test]
    fn is_disjoint_bitmap_store_1() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in [500, 5001, 20, 40] {
            bitmap_store.insert(to_set);
        }
        let interval_store = IntervalStore(alloc::vec![
            Interval { start: 1000, end: 4000 },
            Interval { start: 8000, end: 10000 },
        ]);
        assert!(interval_store.is_disjoint_bitmap(&bitmap_store));
    }

    #[test]
    fn is_disjoint_bitmap_store_2() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in [500, 5001, 20, 40] {
            bitmap_store.insert(to_set);
        }
        let interval_store = IntervalStore(alloc::vec![Interval { start: 1, end: 400 },]);
        assert!(!interval_store.is_disjoint_bitmap(&bitmap_store));
    }

    #[test]
    fn is_subset_1() {
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 1500, end: 1600 },]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(interval_store_1.is_subset(&interval_store_1));
        assert!(interval_store_2.is_subset(&interval_store_2));
        assert!(interval_store_1.is_subset(&interval_store_2));
        assert!(!interval_store_2.is_subset(&interval_store_1));
    }

    #[test]
    fn is_subset_2() {
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 50, end: 700 },]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 600 },
            Interval { start: 1401, end: 1600 },
            Interval { start: 15901, end: 16000 },
        ]);
        assert!(interval_store_1.is_subset(&interval_store_1));
        assert!(interval_store_2.is_subset(&interval_store_2));
        assert!(!interval_store_1.is_subset(&interval_store_2));
        assert!(!interval_store_2.is_subset(&interval_store_1));
    }

    #[test]
    fn overlapping_interval_1() {
        let interval1 = Interval::new(0, 100);
        let interval2 = Interval::new(50, 300);

        assert_eq!(interval1.overlapping_interval(&interval2), Some(Interval::new(50, 100)))
    }

    #[test]
    fn overlapping_interval_2() {
        let interval1 = Interval::new(50, 300);
        let interval2 = Interval::new(0, 100);

        assert_eq!(interval1.overlapping_interval(&interval2), Some(Interval::new(50, 100)))
    }

    #[test]
    fn overlapping_interval_3() {
        let interval1 = Interval::new(0, 100);
        let interval2 = Interval::new(500, 700);

        assert_eq!(interval1.overlapping_interval(&interval2), None)
    }

    #[test]
    fn intersection_len_1() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 11, end: 20 },
            Interval { start: 51, end: 80 },
            Interval { start: 111, end: 120 },
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 20 },
            Interval { start: 41, end: 80 },
            Interval { start: 101, end: 120 },
        ]);
        assert_eq!(
            interval_store_1.intersection_len(&interval_store_2),
            Interval::new(11, 20).run_len()
                + Interval::new(51, 80).run_len()
                + Interval::new(111, 120).run_len()
        )
    }

    #[test]
    fn intersection_len_2() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 11, end: 20 },
            Interval { start: 51, end: 80 },
            Interval { start: 111, end: 120 },
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval { start: 1, end: 80 },
            Interval { start: 101, end: 120 },
        ]);
        let intersect_len = Interval::new(11, 20).run_len()
            + Interval::new(51, 80).run_len()
            + Interval::new(111, 120).run_len();
        assert_eq!(interval_store_1.intersection_len(&interval_store_2), intersect_len);
        assert_eq!(interval_store_2.intersection_len(&interval_store_1), intersect_len);
    }

    #[test]
    fn intersection_len_3() {
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 1, end: 2000 },]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval { start: 1001, end: 3000 },]);
        let intersect_len = Interval::new(1001, 2000).run_len();
        assert_eq!(interval_store_1.intersection_len(&interval_store_2), intersect_len);
        assert_eq!(interval_store_2.intersection_len(&interval_store_1), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_1() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in [500, 5001, 20, 40, 60] {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 20, end: 600 },]);
        let intersect_len = 4;
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_2() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in 0..=200 {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 20, end: 600 },]);
        let intersect_len = Interval::new(20, 200).run_len();
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_3() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in 0..=20000 {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 20, end: 6000 },
            Interval { start: 5000, end: 33333 },
        ]);
        let intersect_len =
            Interval::new(20, 6000).run_len() + Interval::new(5000, 20000).run_len();
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_4() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in 0..=20000 {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 64, end: 6400 },
            Interval { start: 7680, end: 64000 },
        ]);
        let intersect_len =
            Interval::new(64, 6400).run_len() + Interval::new(7680, 20000).run_len();
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_5() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in 0..=20005 {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 64, end: 6400 },
            Interval { start: 7680, end: 64000 },
        ]);
        let intersect_len =
            Interval::new(64, 6400).run_len() + Interval::new(7680, 20005).run_len();
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_bitmap_6() {
        let mut bitmap_store = BitmapStore::new();
        for to_set in 0..=20005 {
            bitmap_store.insert(to_set);
        }
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 64, end: 64 },]);
        let intersect_len = Interval::new(64, 64).run_len();
        assert_eq!(interval_store_1.intersection_len_bitmap(&bitmap_store), intersect_len);
    }

    #[test]
    fn intersection_len_array_1() {
        let array_store = ArrayStore::from_vec_unchecked(alloc::vec![20, 40, 60, 500, 5001]);
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 20, end: 600 },]);
        let intersect_len = 4;
        assert_eq!(interval_store_1.intersection_len_array(&array_store), intersect_len);
    }

    #[test]
    fn intersection_len_array_2() {
        let array_store = ArrayStore::from_vec_unchecked(Vec::from_iter(0..200));
        let interval_store_1 = IntervalStore(alloc::vec![Interval { start: 20, end: 600 },]);
        let intersect_len = 200 - 20;
        assert_eq!(interval_store_1.intersection_len_array(&array_store), intersect_len);
    }

    #[test]
    fn len_1() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval { start: 20, end: 600 },
            Interval { start: 5000, end: 8000 },
        ]);
        assert_eq!(
            interval_store_1.len(),
            Interval::new(20, 600).run_len() + Interval::new(5000, 8000).run_len()
        );
    }

    #[test]
    fn is_empty() {
        let mut interval_store = IntervalStore(alloc::vec![
            Interval { start: 20, end: 600 },
            Interval { start: 5000, end: 8000 },
        ]);
        assert!(!interval_store.is_empty());
        interval_store.remove_range(0..=u16::MAX);
        assert!(interval_store.is_empty());
    }

    #[test]
    fn min_0() {
        let interval_store = IntervalStore(alloc::vec![Interval::new(20, u16::MAX)]);
        assert_eq!(interval_store.min(), Some(20));
    }

    #[test]
    fn min_1() {
        let interval_store = IntervalStore(alloc::vec![]);
        assert_eq!(interval_store.min(), None);
    }

    #[test]
    fn max_0() {
        let interval_store = IntervalStore(alloc::vec![Interval::new(20, u16::MAX)]);
        assert_eq!(interval_store.max(), Some(u16::MAX));
    }

    #[test]
    fn max_1() {
        let interval_store = IntervalStore(alloc::vec![]);
        assert_eq!(interval_store.max(), None);
    }

    #[test]
    fn rank() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 200),
            Interval::new(5000, 7000),
            Interval::new(8000, 10000),
        ]);
        assert_eq!(
            interval_store.rank(5020),
            Interval::new(0, 200).run_len() + Interval::new(5000, 5020).run_len()
        );
        assert_eq!(interval_store.rank(u16::MAX), interval_store.len());
    }

    #[test]
    fn select() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 10000),
        ]);
        assert_eq!(interval_store.select(0), Some(0));
        assert_eq!(interval_store.select(1), Some(2));
        assert_eq!(interval_store.select(10), Some(11));
        assert_eq!(interval_store.select(11), Some(5000));
        assert_eq!(interval_store.select(11 + 3), Some(5003));
        assert_eq!(interval_store.select(11 + 2001), Some(8000));
    }

    #[test]
    fn union_1() {
        let mut interval_store_1 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 10000),
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 10),
            Interval::new(12, 7000),
            Interval::new(65000, 65050),
        ]);
        interval_store_1 |= interval_store_2;
        assert_eq!(
            interval_store_1,
            IntervalStore(alloc::vec![
                Interval::new(0, 0),
                Interval::new(2, 7000),
                Interval::new(8000, 10000),
                Interval::new(65000, 65050),
            ])
        )
    }

    #[test]
    fn union_array() {
        let mut values = alloc::vec![0, 1, 2, 3, 4, 2000, 5000, u16::MAX];
        values.sort();
        let array = ArrayStore::from_vec_unchecked(values);
        let mut interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 10000),
        ]);
        interval_store |= &array;
        assert_eq!(
            interval_store,
            IntervalStore(alloc::vec![
                Interval::new(0, 11),
                Interval::new(2000, 2000),
                Interval::new(5000, 7000),
                Interval::new(8000, 10000),
                Interval::new(u16::MAX, u16::MAX),
            ])
        )
    }

    #[test]
    fn intersection() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 10000),
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(5, 50),
            Interval::new(4000, 10000),
        ]);
        assert_eq!(
            &interval_store_1 & &interval_store_2,
            IntervalStore(alloc::vec![
                Interval::new(0, 0),
                Interval::new(5, 11),
                Interval::new(5000, 7000),
                Interval::new(8000, 10000),
            ])
        );
        assert_eq!(&interval_store_1 & &interval_store_1, interval_store_1);
    }

    #[test]
    fn difference() {
        let mut interval_store_1 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 11000),
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(5, 50),
            Interval::new(4000, 10000),
        ]);
        interval_store_1 -= &interval_store_2;
        assert_eq!(
            interval_store_1,
            IntervalStore(alloc::vec![Interval::new(2, 4), Interval::new(10001, 11000),])
        )
    }

    #[test]
    fn symmetric_difference_0() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(2, 11),
            Interval::new(5000, 7000),
            Interval::new(8000, 11000),
            Interval::new(40000, 50000),
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![
            Interval::new(0, 0),
            Interval::new(5, 50),
            Interval::new(4000, 10000),
        ]);
        assert_eq!(
            &interval_store_1 ^ &interval_store_2,
            IntervalStore(alloc::vec![
                Interval::new(2, 4),
                Interval::new(12, 50),
                Interval::new(4000, 4999),
                Interval::new(7001, 7999),
                Interval::new(10001, 11000),
                Interval::new(40000, 50000),
            ])
        );
    }

    #[test]
    fn symmetric_difference_1() {
        let interval_store_1 = IntervalStore(alloc::vec![Interval::new(0, 50),]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval::new(100, 200),]);
        assert_eq!(
            &interval_store_1 ^ &interval_store_2,
            IntervalStore(alloc::vec![Interval::new(0, 50), Interval::new(100, 200),])
        );
    }

    #[test]
    fn symmetric_difference_2() {
        let interval_store_1 = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let interval_store_2 = IntervalStore(alloc::vec![Interval::new(0, 6000),]);
        assert_eq!(
            &interval_store_1 ^ &interval_store_2,
            IntervalStore(alloc::vec![
                Interval::new(51, 499),
                Interval::new(601, 799),
                Interval::new(1001, 6000),
            ])
        );
    }

    #[test]
    fn iter_next() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.into_iter();

        let size = (Interval::new(0, 50).run_len()
            + Interval::new(500, 600).run_len()
            + Interval::new(800, 1000).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next() {
            assert_eq!(i, value as usize);
            i += 1;
            if i >= 51 {
                break;
            }
            let size = (Interval::new(i as u16, 50).run_len()
                + Interval::new(500, 600).run_len()
                + Interval::new(800, 1000).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let size =
            (Interval::new(500, 600).run_len() + Interval::new(800, 1000).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next() {
            assert_eq!(i + 500, value as usize);
            i += 1;
            if i >= 101 {
                break;
            }
            let size = (Interval::new((i + 500) as u16, 600).run_len()
                + Interval::new(800, 1000).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let size = Interval::new(800, 1000).run_len() as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next() {
            if i >= 201 {
                break;
            }
            assert_eq!(i + 800, value as usize);
            i += 1;
            if i >= 201 {
                break;
            }
            let size = (Interval::new((i + 800) as u16, 1000).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));

        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn iter_next_back() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.into_iter();

        let size = (Interval::new(0, 50).run_len()
            + Interval::new(500, 600).run_len()
            + Interval::new(800, 1000).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(1000 - i, value as usize);
            i += 1;
            if i >= 201 {
                break;
            }
            let size = (Interval::new(0, 50).run_len()
                + Interval::new(500, 600).run_len()
                + Interval::new(800, (1000 - i) as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(600 - i, value as usize);
            i += 1;
            if i >= 101 {
                break;
            }
            let size = (Interval::new(0, 50).run_len()
                + Interval::new(500, (600 - i) as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(50 - i, value as usize);
            i += 1;
            if i >= 51 {
                break;
            }
            let size = (Interval::new(0, (50 - i) as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn iter_next_and_next_back() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.into_iter();

        let size = (Interval::new(0, 50).run_len()
            + Interval::new(500, 600).run_len()
            + Interval::new(800, 1000).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(1000 - i, value as usize);
            i += 1;
            if i >= 201 {
                break;
            }
            let size = (Interval::new(0, 50).run_len()
                + Interval::new(500, 600).run_len()
                + Interval::new(800, (1000 - i) as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let size = (Interval::new(0, 50).run_len() + Interval::new(500, 600).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(600 - i, value as usize);
            i += 1;
            if i >= 101 {
                break;
            }
            let size = (Interval::new(0, 50).run_len()
                + Interval::new(500, (600 - i) as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let size = (Interval::new(0, 50).run_len()) as usize;
        assert_eq!(iter.size_hint(), (size, Some(size)));

        let mut i = 0;
        while let Some(value) = iter.next() {
            assert_eq!(i, value as usize);
            i += 1;
            if i >= 51 {
                break;
            }
            let size = (Interval::new(i as u16, 50).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn iter_u16_max() {
        let interval_store = IntervalStore(alloc::vec![Interval::new(0, u16::MAX),]);
        let mut iter = interval_store.iter();

        let mut i = 0;
        while let Some(value) = iter.next() {
            assert_eq!(i, value as usize);
            i += 1;
            if i >= u16::MAX as usize {
                break;
            }
            let size = (Interval::new(i as u16, u16::MAX).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }

        let mut iter = interval_store.iter();

        let mut i = 0;
        while let Some(value) = iter.next_back() {
            assert_eq!(u16::MAX as usize - i, value as usize);
            i += 1;
            if i >= u16::MAX as usize {
                break;
            }
            let size = (Interval::new(0, u16::MAX - i as u16).run_len()) as usize;
            assert_eq!(iter.size_hint(), (size, Some(size)));
        }
        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(u16::MAX as usize), Some(u16::MAX));
    }

    #[test]
    fn iter_nth() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(50), Some(50));

        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(51), Some(500));

        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(100), Some(549));

        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(152), Some(800));

        let mut iter = interval_store.iter();
        assert_eq!(
            iter.nth(
                (Interval::new(0, 50).run_len()
                    + Interval::new(500, 600).run_len()
                    + Interval::new(800, 1000).run_len()
                    - 1) as usize
            ),
            Some(1000)
        );

        let mut iter = interval_store.iter();
        iter.next();
        iter.next();
        iter.next();
        assert_eq!(iter.nth(152), Some(803));

        let mut iter = interval_store.iter();
        assert_eq!(iter.nth(u16::MAX as usize), None);
    }

    #[test]
    fn iter_advance_to() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.iter();
        iter.advance_to(20);
        assert_eq!(iter.next(), Some(20));
        iter.advance_to(800);
        assert_eq!(iter.next(), Some(800));
        iter.advance_to(u16::MAX);
        assert_eq!(iter.next(), Some(801));

        let mut iter = interval_store.iter();
        iter.advance_to(100);
        assert_eq!(iter.next(), Some(500));
        iter.advance_to(800);
        assert_eq!(iter.next(), Some(800));
        iter.advance_to(900);
        assert_eq!(iter.next(), Some(900));
        iter.advance_to(800);
        assert_eq!(iter.next(), Some(901));
        let mut iter = interval_store.iter();
        iter.next();
        iter.next();
        iter.next();
        iter.advance_to(499);
        assert_eq!(iter.next(), Some(500));

        let mut iter = interval_store.iter();
        iter.advance_to(100);
        assert_eq!(iter.next(), Some(500));
    }

    #[test]
    fn iter_advance_back_to() {
        let interval_store = IntervalStore(alloc::vec![
            Interval::new(0, 50),
            Interval::new(500, 600),
            Interval::new(800, 1000),
        ]);
        let mut iter = interval_store.iter();
        iter.advance_back_to(u16::MAX);
        assert_eq!(iter.next_back(), Some(1000));
        iter.advance_back_to(800);
        assert_eq!(iter.next_back(), Some(800));
        iter.advance_back_to(20);
        assert_eq!(iter.next_back(), Some(20));

        let mut iter = interval_store.iter();
        iter.advance_back_to(800);
        assert_eq!(iter.next_back(), Some(800));
        iter.advance_back_to(900);
        assert_eq!(iter.next_back(), Some(600));
        iter.advance_back_to(550);
        assert_eq!(iter.next_back(), Some(550));
        iter.advance_back_to(20);
        assert_eq!(iter.next_back(), Some(20));
        let mut iter = interval_store.iter();
        iter.next_back();
        iter.next_back();
        iter.next_back();
        iter.advance_back_to(700);
        assert_eq!(iter.next_back(), Some(600));
        let mut iter = interval_store.iter();
        iter.advance_back_to(400);
        assert_eq!(iter.next_back(), Some(50));
    }
}
