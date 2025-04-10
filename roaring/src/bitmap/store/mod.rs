mod array_store;
mod bitmap_store;

use alloc::vec;
use core::cmp::Ordering;
use core::mem;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, Sub, SubAssign,
};
use core::slice;

pub use self::bitmap_store::BITMAP_LENGTH;
use self::Store::{Array, Bitmap, Run};

pub(crate) use self::array_store::ArrayStore;
pub use self::bitmap_store::{BitmapIter, BitmapStore};

use crate::bitmap::container::ARRAY_LIMIT;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Interval {
    pub start: u16,
    pub end: u16,
}

fn cmp_index_interval(index: u16, iv: Interval) -> Ordering {
    if index < iv.start {
        Ordering::Less
    } else if index > iv.end {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

impl Interval {
    pub fn new(start: u16, end: u16) -> Interval {
        Interval { start, end }
    }

    pub fn run_len(&self) -> u64 {
        (self.end - self.start) as u64 + 1
    }
}

#[derive(Clone)]
pub(crate) enum Store {
    Array(ArrayStore),
    Bitmap(BitmapStore),
    Run(Vec<Interval>),
}

#[derive(Clone)]
pub(crate) enum Iter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<&'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<Box<[u64; BITMAP_LENGTH]>>),
    Run(RunIter),
}

#[derive(Clone)]
pub struct RunIter {
    run: usize,
    offset: u64,
    intervals: Vec<Interval>,
}

impl Store {
    pub fn new() -> Store {
        Store::Array(ArrayStore::new())
    }

    #[cfg(feature = "std")]
    pub fn with_capacity(capacity: usize) -> Store {
        if capacity <= ARRAY_LIMIT as usize {
            Store::Array(ArrayStore::with_capacity(capacity))
        } else {
            Store::Bitmap(BitmapStore::new())
        }
    }

    pub fn full() -> Store {
        Store::Bitmap(BitmapStore::full())
    }

    pub fn from_lsb0_bytes(bytes: &[u8], byte_offset: usize) -> Option<Self> {
        assert!(byte_offset + bytes.len() <= BITMAP_LENGTH * mem::size_of::<u64>());

        // It seems to be pretty considerably faster to count the bits
        // using u64s than for each byte
        let bits_set = {
            let mut bits_set = 0;
            let chunks = bytes.chunks_exact(mem::size_of::<u64>());
            let remainder = chunks.remainder();
            for chunk in chunks {
                let chunk = u64::from_ne_bytes(chunk.try_into().unwrap());
                bits_set += u64::from(chunk.count_ones());
            }
            for byte in remainder {
                bits_set += u64::from(byte.count_ones());
            }
            bits_set
        };
        if bits_set == 0 {
            return None;
        }

        Some(if bits_set < ARRAY_LIMIT {
            Array(ArrayStore::from_lsb0_bytes(bytes, byte_offset, bits_set))
        } else {
            Bitmap(BitmapStore::from_lsb0_bytes_unchecked(bytes, byte_offset, bits_set))
        })
    }

    #[inline]
    pub fn insert(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.insert(index),
            Bitmap(bits) => bits.insert(index),
            Run(ref mut vec) => {
                vec.binary_search_by(|iv| cmp_index_interval(index, *iv))
                    .map_err(|loc| {
                        // Value is beyond end of interval
                        if vec[loc].end < index {
                            // If immediately follows this interval
                            if index == vec[loc].end - 1 {
                                if loc < vec.len() && index == vec[loc + 1].start {
                                    // Merge with following interval
                                    vec[loc].end = vec[loc + 1].end;
                                    vec.remove(loc + 1);
                                    return;
                                }
                                // Extend end of this interval by 1
                                vec[loc].end += 1
                            } else {
                                // Otherwise create new standalone interval
                                vec.insert(loc, Interval::new(index, index));
                            }
                        } else if vec[loc].start == index + 1 {
                            // Value immediately precedes interval
                            if loc > 0 && vec[loc - 1].end == &index - 1 {
                                // Merge with preceding interval
                                vec[loc - 1].end = vec[loc].end;
                                vec.remove(loc);
                                return;
                            }
                            vec[loc].start -= 1;
                        } else if loc > 0 && index - 1 == vec[loc - 1].end {
                            // Immediately follows the previous interval
                            vec[loc - 1].end += 1
                        } else {
                            vec.insert(loc, Interval::new(index, index));
                        }
                    })
                    .is_err()
            }
        }
    }

    pub fn insert_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        // A Range is defined as being of size 0 if start >= end.
        if range.is_empty() {
            return 0;
        }

        match self {
            Array(vec) => vec.insert_range(range),
            Bitmap(bits) => bits.insert_range(range),
            Run(..) => todo!(),
        }
    }

    /// Push `index` at the end of the store only if `index` is the new max.
    ///
    /// Returns whether `index` was effectively pushed.
    pub fn push(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.push(index),
            Bitmap(bits) => bits.push(index),
            Run(..) => todo!(),
        }
    }

    ///
    /// Pushes `index` at the end of the store.
    /// It is up to the caller to have validated index > self.max()
    ///
    /// # Panics
    ///
    /// If debug_assertions enabled and index is > self.max()
    pub(crate) fn push_unchecked(&mut self, index: u16) {
        match self {
            Array(vec) => vec.push_unchecked(index),
            Bitmap(bits) => bits.push_unchecked(index),
            Run(..) => todo!(),
        }
    }

    pub fn remove(&mut self, index: u16) -> bool {
        match self {
            Array(vec) => vec.remove(index),
            Bitmap(bits) => bits.remove(index),
            Run(ref mut vec) => vec
                .binary_search_by(|iv| cmp_index_interval(index, *iv))
                .map(|loc| {
                    if index == vec[loc].start && index == vec[loc].end {
                        // Remove entire run if it only contains this value
                        vec.remove(loc);
                    } else if index == vec[loc].end {
                        // Value is last in this interval
                        vec[loc].end = index - 1;
                    } else if index == vec[loc].start {
                        // Value is first in this interval
                        vec[loc].start = index + 1;
                    } else {
                        // Value lies inside the interval, we need to split it
                        // First construct a new interval with the right part
                        let new_interval = Interval::new(index + 1, vec[loc].end);
                        // Then shrink the current interval
                        vec[loc].end = index - 1;
                        // Then insert the new interval leaving gap where value was removed
                        vec.insert(loc + 1, new_interval);
                    }
                })
                .is_ok(),
        }
    }

    pub fn remove_range(&mut self, range: RangeInclusive<u16>) -> u64 {
        if range.is_empty() {
            return 0;
        }

        match self {
            Array(vec) => vec.remove_range(range),
            Bitmap(bits) => bits.remove_range(range),
            // TODO we must test that algorithm
            Run(ref mut intervals) => {
                let start = *range.start();
                let end = *range.end();
                let mut count = 0;
                let mut search_end = false;

                for iv in intervals.iter_mut() {
                    if !search_end && cmp_index_interval(start as u16, *iv) == Ordering::Equal {
                        count += Interval::new(iv.end, start as u16).run_len();
                        iv.end = start as u16;
                        search_end = true;
                    }

                    if search_end {
                        // The end bound is non-inclusive therefore we must search for end - 1.
                        match cmp_index_interval(end, *iv) {
                            Ordering::Less => {
                                // We invalidate the intervals that are contained in
                                // the start and end but doesn't touch the bounds.
                                count += iv.run_len();
                                *iv = Interval::new(u16::max_value(), 0);
                            }
                            Ordering::Equal => {
                                // We shrink this interval by moving the start of it to be
                                // the end bound which is non-inclusive.
                                count += Interval::new(end as u16, iv.start).run_len();
                                iv.start = end as u16;
                            }
                            Ordering::Greater => break,
                        }
                    }
                }

                // We invalidated the intervals to remove,
                // the start is greater than the end.
                intervals.retain(|iv| iv.start <= iv.end);

                count
            }
        }
    }

    pub fn remove_smallest(&mut self, index: u64) {
        match self {
            Array(vec) => vec.remove_smallest(index),
            Bitmap(bits) => bits.remove_smallest(index),
            Run(..) => todo!(),
        }
    }

    pub fn remove_biggest(&mut self, index: u64) {
        match self {
            Array(vec) => vec.remove_biggest(index),
            Bitmap(bits) => bits.remove_biggest(index),
            Run(..) => todo!(),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match self {
            Array(vec) => vec.contains(index),
            Bitmap(bits) => bits.contains(index),
            Run(ref intervals) => intervals
                .binary_search_by(|iv| cmp_index_interval(index, *iv))
                .is_ok(),
        }
    }

    pub fn contains_range(&self, range: RangeInclusive<u16>) -> bool {
        match self {
            Array(vec) => vec.contains_range(range),
            Bitmap(bits) => bits.contains_range(range),
            Run(..) => todo!(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.len() == (1 << 16)
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.is_disjoint(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.is_disjoint(bits2),
            (Array(vec), Bitmap(bits)) | (Bitmap(bits), Array(vec)) => {
                vec.iter().all(|&i| !bits.contains(i))
            }
            // TODO(jpg) is_disjoint
            (&Run(ref intervals1), &Run(ref intervals2)) => {
                let (mut i1, mut i2) = (intervals1.iter(), intervals2.iter());
                let (mut iv1, mut iv2) = (i1.next(), i2.next());
                loop {
                    match (iv1, iv2) {
                        (Some(v1), Some(v2)) => {
                            if v2.start <= v1.end && v1.start <= v2.end {
                                return false;
                            }

                            match v1.end.cmp(&v2.end) {
                                Ordering::Less => iv1 = i1.next(),
                                Ordering::Greater => iv2 = i2.next(),
                                Ordering::Equal => {
                                    iv1 = i1.next();
                                    iv2 = i2.next();
                                }
                            }
                        }
                        (_, _) => return true,
                    }
                }
            }
            (run @ &Run(..), &Array(ref vec)) | (&Array(ref vec), run @ &Run(..)) => {
                vec.iter().all(|&i| !run.contains(i))
            }
            (&Run(ref _intervals), _store @ &Bitmap(..))
            | (_store @ &Bitmap(..), &Run(ref _intervals)) => unimplemented!(),
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.is_subset(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.is_subset(bits2),
            (Array(vec), Bitmap(bits)) => vec.iter().all(|&i| bits.contains(i)),
            (Bitmap(..), &Array(..)) => false,
            (&Array(ref vec), run @ &Run(..)) => vec.iter().all(|&i| run.contains(i)),
            // TODO(jpg) is subset bitmap, run
            (&Bitmap(..), &Run(ref _vec)) => unimplemented!(),

            // TODO(jpg) is_subset run, *
            (&Run(ref _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&Run(ref _intervals), &Array(ref _vec)) => unimplemented!(),
            (&Run(ref _intervals), _store @ &Bitmap(..)) => unimplemented!(),
        }
    }

    pub fn intersection_len(&self, other: &Self) -> u64 {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1.intersection_len(vec2),
            (Bitmap(bits1), Bitmap(bits2)) => bits1.intersection_len_bitmap(bits2),
            (Array(vec), Bitmap(bits)) => bits.intersection_len_array(vec),
            (Bitmap(bits), Array(vec)) => bits.intersection_len_array(vec),
            (Run(..), _) => todo!(),
            (_, Run(..)) => todo!(),
        }
    }

    pub fn len(&self) -> u64 {
        match self {
            Array(vec) => vec.len(),
            Bitmap(bits) => bits.len(),
            Run(ref intervals) => intervals.iter().map(|iv| iv.run_len() as u64).sum(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Array(vec) => vec.is_empty(),
            Bitmap(bits) => bits.is_empty(),
            Run(..) => todo!(),
        }
    }

    pub fn min(&self) -> Option<u16> {
        match self {
            Array(vec) => vec.min(),
            Bitmap(bits) => bits.min(),
            Run(ref intervals) => intervals.first().map(|f| f.start),
        }
    }

    #[inline]
    pub fn max(&self) -> Option<u16> {
        match self {
            Array(vec) => vec.max(),
            Bitmap(bits) => bits.max(),
            Run(ref intervals) => intervals.last().map(|f| f.end),
        }
    }

    pub fn rank(&self, index: u16) -> u64 {
        match self {
            Array(vec) => vec.rank(index),
            Bitmap(bits) => bits.rank(index),
            Run(..) => todo!(),
        }
    }

    pub fn select(&self, n: u16) -> Option<u16> {
        match self {
            Array(vec) => vec.select(n),
            Bitmap(bits) => bits.select(n),
            Run(..) => todo!(),
        }
    }

    pub fn count_runs(&self) -> u64 {
        match *self {
            Array(ref vec) => {
                vec.iter()
                    .fold((-2, 0u64), |(prev, runs), &v| {
                        let new = v as i32;
                        if prev + 1 != new {
                            (new, runs + 1)
                        } else {
                            (new, runs)
                        }
                    })
                    .1
            }
            Bitmap(ref bits) => {
                let mut num_runs = 0u64;

                for i in 0..BITMAP_LENGTH - 1 {
                    let word = bits.as_array()[i];
                    let next_word = bits.as_array()[i + 1];
                    num_runs +=
                        ((word << 1) & !word).count_ones() as u64 + ((word >> 63) & !next_word);
                }

                let last = bits.as_array()[BITMAP_LENGTH - 1];
                num_runs += ((last << 1) & !last).count_ones() as u64 + (last >> 63);
                num_runs
            }
            Run(ref intervals) => intervals.len() as u64,
        }
    }

    pub(crate) fn to_bitmap(&self) -> Store {
        match self {
            Array(arr) => Bitmap(arr.to_bitmap_store()),
            Bitmap(_) => self.clone(),
            Run(ref intervals) => {
                let mut bits = BitmapStore::new();
                for iv in intervals {
                    for index in iv.start..=iv.end {
                        bits.mut_array()[bitmap_store::key(index)] |= 1 << bitmap_store::bit(index);
                    }
                }
                Bitmap(bits)
            }
        }
    }

    pub(crate) fn to_run(&self) -> Self {
        match *self {
            Array(ref vec) => {
                let mut intervals = Vec::new();
                let mut start = *vec.as_slice().first().unwrap();
                for (idx, &v) in vec.as_slice()[1..].iter().enumerate() {
                    if v - vec.as_slice()[idx] > 1 {
                        intervals.push(Interval::new(start, vec.as_slice()[idx]));
                        start = v
                    }
                }
                intervals.push(Interval::new(start, *vec.as_slice().last().unwrap()));
                Run(intervals)
            }
            Bitmap(ref bits) => {
                let mut current = bits.as_array()[0];
                let mut i = 0u16;
                let mut start;
                let mut last;

                let mut intervals = Vec::new();

                loop {
                    // Skip over empty words
                    while current == 0 && i < BITMAP_LENGTH as u16 - 1 {
                        i += 1;
                        current = bits.as_array()[i as usize];
                    }
                    // Reached end of the bitmap without finding anymore bits set
                    if current == 0 {
                        break;
                    }
                    let current_start = current.trailing_zeros() as u16;
                    start = 64 * i + current_start;

                    // Pad LSBs with 1s
                    current |= current - 1;

                    // Find next 0
                    while current == std::u64::MAX && i < BITMAP_LENGTH as u16 - 1 {
                        i += 1;
                        current = bits.as_array()[i as usize];
                    }

                    // Run continues until end of this container
                    if current == std::u64::MAX {
                        intervals.push(Interval::new(start, std::u16::MAX));
                        break;
                    }

                    let current_last = (!current).trailing_zeros() as u16;
                    last = 64 * i + current_last;
                    intervals.push(Interval::new(start, last - 1));

                    // pad LSBs with 0s
                    current &= current + 1;
                }
                Run(intervals)
            }
            Run(ref _intervals) => panic!("Cannot convert run to run"),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}

impl BitOr<&Store> for &Store {
    type Output = Store;

    fn bitor(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitOr::bitor(vec1, vec2)),
            (&Bitmap(..), &Array(..)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Bitmap(..), &Bitmap(..)) => {
                let mut lhs = self.clone();
                BitOrAssign::bitor_assign(&mut lhs, rhs);
                lhs
            }
            (&Array(..), &Bitmap(..)) => {
                let mut rhs = rhs.clone();
                BitOrAssign::bitor_assign(&mut rhs, self);
                rhs
            }
            (Run(..), _) => todo!(),
            (_, Run(..)) => todo!(),
        }
    }
}

impl BitOrAssign<Store> for Store {
    fn bitor_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref vec2)) => {
                *vec1 = BitOr::bitor(&*vec1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Array(ref vec2)) => {
                BitOrAssign::bitor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitOrAssign::bitor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), Run(..)) => {
                let new = rhs.clone();
                BitOrAssign::bitor_assign(this, new);
            }
            (this @ &mut Bitmap(..), Run(..)) => {
                let other = rhs.to_bitmap();
                BitOrAssign::bitor_assign(this, other);
            }
            (&mut Run(ref mut intervals1), Run(ref intervals2)) => {
                let mut merged = Vec::new();

                let (mut i1, mut i2) = (intervals1.iter(), intervals2.iter());
                let (mut iv1, mut iv2) = (i1.next(), i2.next());
                loop {
                    // Iterate over two iterators and return the lowest value at each step.
                    let iv = match (iv1, iv2) {
                        (None, None) => break,
                        (Some(v1), None) => {
                            iv1 = i1.next();
                            v1
                        }
                        (None, Some(v2)) => {
                            iv2 = i2.next();
                            v2
                        }
                        (Some(v1), Some(v2)) => match v1.start.cmp(&v2.start) {
                            Ordering::Equal => {
                                iv1 = i1.next();
                                iv2 = i2.next();
                                v1
                            }
                            Ordering::Less => {
                                iv1 = i1.next();
                                v1
                            }
                            Ordering::Greater => {
                                iv2 = i2.next();
                                v2
                            }
                        },
                    };

                    match merged.last_mut() {
                        // If the list of merged intervals is empty, append the interval.
                        None => merged.push(*iv),
                        Some(last) => {
                            if last.end < iv.start {
                                // If the interval does not overlap with the previous, append it.
                                merged.push(*iv);
                            } else {
                                // If there is overlap, so we merge the current and previous intervals.
                                last.end = core::cmp::max(last.end, iv.end);
                            }
                        }
                    }
                }

                *intervals1 = merged;
            }
            (ref mut this @ &mut Run(..), Array(ref vec)) => {
                for i in vec.iter() {
                    this.insert(*i);
                }
            }
            (this @ &mut Run(..), Bitmap(..)) => {
                *this = this.to_bitmap();
                BitOrAssign::bitor_assign(this, rhs);
            }
            (this @ &mut Array(..), &mut Bitmap(..)) => {
                mem::swap(this, &mut rhs);
                BitOrAssign::bitor_assign(this, rhs);
            }
        }
    }
}

impl BitOrAssign<&Store> for Store {
    fn bitor_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let this = mem::take(vec1);
                *vec1 = BitOr::bitor(&this, vec2);
            }
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                BitOrAssign::bitor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitOrAssign::bitor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), Bitmap(bits2)) => {
                let mut lhs: Store = Bitmap(bits2.clone());
                BitOrAssign::bitor_assign(&mut lhs, &*this);
                *this = lhs;
            }
            (Run(..), _) => todo!(),
            (_, Run(..)) => todo!(),
        }
    }
}

impl BitAnd<&Store> for &Store {
    type Output = Store;

    fn bitand(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitAnd::bitand(vec1, vec2)),
            (&Bitmap(..), &Array(..)) => {
                let mut rhs = rhs.clone();
                BitAndAssign::bitand_assign(&mut rhs, self);
                rhs
            }
            _ => {
                let mut lhs = self.clone();
                BitAndAssign::bitand_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl BitAndAssign<Store> for Store {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn bitand_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref mut vec2)) => {
                if vec2.len() < vec1.len() {
                    mem::swap(vec1, vec2);
                }
                BitAndAssign::bitand_assign(vec1, &*vec2);
            }
            (&mut Array(ref mut vec), run @ Run(..)) => {
                vec.retain(|i| run.contains(i));
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitAndAssign::bitand_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), &mut Bitmap(ref bits2)) => {
                BitAndAssign::bitand_assign(vec1, bits2);
            }
            (this @ &mut Bitmap(..), Run(..)) => {
                let other = rhs.to_bitmap();
                BitAndAssign::bitand_assign(this, other);
            }
            (&mut Run(ref mut intervals1), Run(ref intervals2)) => {
                let mut merged = Vec::new();

                let (mut i1, mut i2) = (intervals1.iter(), intervals2.iter());
                let (mut iv1, mut iv2) = (i1.next(), i2.next());

                // Iterate over both iterators.
                while let (Some(v1), Some(v2)) = (iv1, iv2) {
                    if v2.start <= v1.end && v1.start <= v2.end {
                        let start = core::cmp::max(v1.start, v2.start);
                        let end = core::cmp::min(v1.end, v2.end);
                        let iv = Interval::new(start, end);
                        merged.push(iv);
                    }

                    match v1.end.cmp(&v2.end) {
                        Ordering::Less => iv1 = i1.next(),
                        Ordering::Greater => iv2 = i2.next(),
                        Ordering::Equal => {
                            iv1 = i1.next();
                            iv2 = i2.next();
                        }
                    }
                }

                *intervals1 = merged;
            }
            (this @ &mut Run(..), other @ Array(..)) => {
                let new = other.clone();
                BitAndAssign::bitand_assign(this, new);
            }
            (this @ &mut Run(..), other @ Bitmap(..)) => {
                let new = other.clone();
                BitAndAssign::bitand_assign(this, new);
            }
            (this @ &mut Bitmap(..), &mut Array(..)) => {
                mem::swap(this, &mut rhs);
                BitAndAssign::bitand_assign(this, rhs);
            }
        }
    }
}

impl BitAndAssign<&Store> for Store {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn bitand_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let (mut lhs, rhs) = if vec2.len() < vec1.len() {
                    (vec2.clone(), &*vec1)
                } else {
                    (mem::take(vec1), vec2)
                };

                BitAndAssign::bitand_assign(&mut lhs, rhs);
                *vec1 = lhs;
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitAndAssign::bitand_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), Bitmap(bits2)) => {
                BitAndAssign::bitand_assign(vec1, bits2);
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = rhs.clone();
                BitAndAssign::bitand_assign(&mut new, &*this);
                *this = new;
            }
            (Run(..), _) => todo!(),
            (_, Run(..)) => todo!(),
        }
    }
}

impl Sub<&Store> for &Store {
    type Output = Store;

    fn sub(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(Sub::sub(vec1, vec2)),
            _ => {
                let mut lhs = self.clone();
                SubAssign::sub_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl SubAssign<&Store> for Store {
    fn sub_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                SubAssign::sub_assign(vec1, vec2);
            }
            (&mut Array(ref mut vec), run @ &Run(..)) => {
                vec.retain(|i| !run.contains(i));
            }
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                SubAssign::sub_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                SubAssign::sub_assign(bits1, bits2);
            }
            (&mut Array(ref mut vec1), Bitmap(bits2)) => {
                SubAssign::sub_assign(vec1, bits2);
            }
            (ref mut this @ &mut Bitmap(..), &Run(ref intervals)) => {
                for iv in intervals {
                    this.remove_range(iv.start..=iv.end);
                }
            }
            (ref mut this @ &mut Run(..), &Run(ref intervals2)) => {
                for iv in intervals2 {
                    this.remove_range(iv.start..=iv.end);
                }
            }
            (ref mut this @ &mut Run(..), &Array(ref vec)) => {
                for i in vec.iter() {
                    this.remove(*i);
                }
            }
            // TODO(jpg) difference_with run bitmap
            (&mut Run(ref mut _vec), _store @ &Bitmap(..)) => unimplemented!(),
        }
    }
}

impl BitXor<&Store> for &Store {
    type Output = Store;

    fn bitxor(self, rhs: &Store) -> Store {
        match (self, rhs) {
            (Array(vec1), Array(vec2)) => Array(BitXor::bitxor(vec1, vec2)),
            (&Array(..), &Bitmap(..)) => {
                let mut lhs = rhs.clone();
                BitXorAssign::bitxor_assign(&mut lhs, self);
                lhs
            }
            _ => {
                let mut lhs = self.clone();
                BitXorAssign::bitxor_assign(&mut lhs, rhs);
                lhs
            }
        }
    }
}

impl BitXorAssign<Store> for Store {
    fn bitxor_assign(&mut self, mut rhs: Store) {
        match (self, &mut rhs) {
            (&mut Array(ref mut vec1), &mut Array(ref vec2)) => {
                *vec1 = BitXor::bitxor(&*vec1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Array(ref vec2)) => {
                BitXorAssign::bitxor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), &mut Bitmap(ref bits2)) => {
                BitXorAssign::bitxor_assign(bits1, bits2);
            }
            (this @ &mut Array(..), &mut Bitmap(..)) => {
                mem::swap(this, &mut rhs);
                BitXorAssign::bitxor_assign(this, rhs);
            }
            (Run(..), _) => todo!(),
            (_, Run(..)) => todo!(),
        }
    }
}

impl BitXorAssign<&Store> for Store {
    fn bitxor_assign(&mut self, rhs: &Store) {
        match (self, rhs) {
            (&mut Array(ref mut vec1), Array(vec2)) => {
                let this = mem::take(vec1);
                *vec1 = BitXor::bitxor(&this, vec2);
            }
            // TODO(jpg) symmetric_difference_with array, run
            (&mut Array(ref mut _vec), &Run(ref _intervals)) => unimplemented!(),
            (&mut Bitmap(ref mut bits1), Array(vec2)) => {
                BitXorAssign::bitxor_assign(bits1, vec2);
            }
            (&mut Bitmap(ref mut bits1), Bitmap(bits2)) => {
                BitXorAssign::bitxor_assign(bits1, bits2);
            }
            // TODO(jpg) symmetric_difference_with bitmap, run
            (ref mut _this @ &mut Bitmap(..), &Run(ref _vec)) => unimplemented!(),
            // TODO(jpg) symmetric_difference_with run, *
            (&mut Run(ref mut _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&mut Run(ref mut _intervals), &Array(ref _vec)) => unimplemented!(),
            (_this @ &mut Run(..), &Bitmap(..)) => unimplemented!(),
            (this @ &mut Array(..), Bitmap(bits2)) => {
                let mut lhs: Store = Bitmap(bits2.clone());
                BitXorAssign::bitxor_assign(&mut lhs, &*this);
                *this = lhs;
            }
        }
    }
}

impl<'a> IntoIterator for &'a Store {
    type Item = u16;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        match self {
            Array(vec) => Iter::Array(vec.iter()),
            Bitmap(bits) => Iter::BitmapBorrowed(bits.iter()),
            Run(ref intervals) => Iter::Run(RunIter::new(intervals.to_vec())),
        }
    }
}

impl IntoIterator for Store {
    type Item = u16;
    type IntoIter = Iter<'static>;
    fn into_iter(self) -> Iter<'static> {
        match self {
            Array(vec) => Iter::Vec(vec.into_iter()),
            Bitmap(bits) => Iter::BitmapOwned(bits.into_iter()),
            Run(intervals) => Iter::Run(RunIter::new(intervals)),
        }
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Array(vec1), Array(vec2)) => vec1 == vec2,
            (Bitmap(bits1), Bitmap(bits2)) => {
                bits1.len() == bits2.len()
                    && bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            (&Run(ref intervals1), &Run(ref intervals2)) => intervals1 == intervals2,
            _ => false,
        }
    }
}

impl RunIter {
    fn new(intervals: Vec<Interval>) -> RunIter {
        RunIter {
            run: 0,
            offset: 0,
            intervals,
        }
    }

    fn move_next(&mut self) {
        self.offset += 1;
        if self.offset == self.intervals[self.run].run_len() {
            self.offset = 0;
            self.run += 1;
        }
    }
}

impl Iterator for RunIter {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        if self.run == self.intervals.len() {
            return None;
        }
        let result = self.intervals[self.run].start + self.offset as u16;
        self.move_next();
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

impl DoubleEndedIterator for RunIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Iter<'_> {
    /// Advance the iterator to the first value greater than or equal to `n`.
    pub(crate) fn advance_to(&mut self, n: u16) {
        match self {
            Iter::Array(inner) => {
                let skip = inner.as_slice().partition_point(|&i| i < n);
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth(nth);
                }
            }
            Iter::Vec(inner) => {
                let skip = inner.as_slice().partition_point(|&i| i < n);
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth(nth);
                }
            }
            Iter::BitmapBorrowed(inner) => inner.advance_to(n),
            Iter::BitmapOwned(inner) => inner.advance_to(n),
            Iter::Run(..) => todo!(),
        }
    }

    pub(crate) fn advance_back_to(&mut self, n: u16) {
        match self {
            Iter::Array(inner) => {
                let slice = inner.as_slice();
                let from_front = slice.partition_point(|&i| i <= n);
                let skip = slice.len() - from_front;
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth_back(nth);
                }
            }
            Iter::Vec(inner) => {
                let slice = inner.as_slice();
                let from_front = slice.partition_point(|&i| i <= n);
                let skip = slice.len() - from_front;
                if let Some(nth) = skip.checked_sub(1) {
                    inner.nth_back(nth);
                }
            }
            Iter::BitmapBorrowed(inner) => inner.advance_back_to(n),
            Iter::BitmapOwned(inner) => inner.advance_back_to(n),
            Iter::Run(..) => todo!(),
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match self {
            Iter::Array(inner) => inner.next().cloned(),
            Iter::Vec(inner) => inner.next(),
            Iter::BitmapBorrowed(inner) => inner.next(),
            Iter::BitmapOwned(inner) => inner.next(),
            Iter::Run(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Array(inner) => inner.size_hint(),
            Iter::Vec(inner) => inner.size_hint(),
            Iter::BitmapBorrowed(inner) => inner.size_hint(),
            Iter::BitmapOwned(inner) => inner.size_hint(),
            Iter::Run(inner) => inner.size_hint(),
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Iter::Array(inner) => inner.count(),
            Iter::Vec(inner) => inner.count(),
            Iter::BitmapBorrowed(inner) => inner.count(),
            Iter::BitmapOwned(inner) => inner.count(),
            Iter::Run(inner) => inner.count(),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Iter::Array(inner) => inner.nth(n).copied(),
            Iter::Vec(inner) => inner.nth(n),
            Iter::BitmapBorrowed(inner) => inner.nth(n),
            Iter::BitmapOwned(inner) => inner.nth(n),
            Iter::Run(inner) => inner.nth(n),
        }
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Array(inner) => inner.next_back().cloned(),
            Iter::Vec(inner) => inner.next_back(),
            Iter::BitmapBorrowed(inner) => inner.next_back(),
            Iter::BitmapOwned(inner) => inner.next_back(),
            Iter::Run(inner) => inner.next_back(),
        }
    }
}

impl ExactSizeIterator for Iter<'_> {}
