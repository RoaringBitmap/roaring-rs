use std::borrow::Borrow;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::vec;
use std::{fmt, slice};

use self::Store::{Array, Bitmap, Run};

pub const BITMAP_LENGTH: usize = 1024;

#[derive(PartialEq, Clone, Debug)]
pub struct Interval {
    pub start: u16,
    pub end: u16,
}

impl Interval {
    pub fn new(start: u16, end: u16) -> Interval {
        Interval { start, end }
    }

    pub fn run_len(&self) -> u64 {
        (self.end - self.start) as u64 + 1
    }
}

pub enum Store {
    Array(Vec<u16>),
    Bitmap(Box<[u64; BITMAP_LENGTH]>),
    Run(Vec<Interval>),
}

pub enum Iter<'a> {
    Array(slice::Iter<'a, u16>),
    Vec(vec::IntoIter<u16>),
    BitmapBorrowed(BitmapIter<&'a [u64; BITMAP_LENGTH]>),
    BitmapOwned(BitmapIter<Box<[u64; BITMAP_LENGTH]>>),
    Run(RunIter),
}

pub struct BitmapIter<B: Borrow<[u64; BITMAP_LENGTH]>> {
    key: usize,
    bit: usize,
    bits: B,
}

pub struct RunIter {
    run: usize,
    offset: u64,
    intervals: Vec<Interval>,
}

impl Store {
    pub fn insert(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => vec
                .binary_search(&index)
                .map_err(|loc| vec.insert(loc, index))
                .is_err(),
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) == 0 {
                    bits[key] |= 1 << bit;
                    true
                } else {
                    false
                }
            }
            Run(ref mut vec) => {
                vec.binary_search_by_key(&index, |iv| iv.start)
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

    pub fn remove(&mut self, index: u16) -> bool {
        match *self {
            Array(ref mut vec) => vec.binary_search(&index).map(|loc| vec.remove(loc)).is_ok(),
            Bitmap(ref mut bits) => {
                let (key, bit) = (key(index), bit(index));
                if bits[key] & (1 << bit) != 0 {
                    bits[key] &= !(1 << bit);
                    true
                } else {
                    false
                }
            }
            Run(ref mut vec) => vec
                .binary_search_by_key(&index, |iv| iv.start)
                .map(|loc| {
                    if index == vec[loc].start && index == vec[loc].end {
                        // Remove entire run if it only contains this value
                        vec.remove(loc);
                    } else if index == vec[loc].end {
                        // Value is last in this interval
                        vec[loc].end -= 1;
                    } else if index == vec[loc].start {
                        // Value is first in this interval
                        vec[loc].start += 1;
                    } else {
                        // Value lies inside the interval, we need to split it
                        // First shrink the current interval
                        vec[loc].end = index - 1;
                        // Then insert a new index leaving gap where value was removed
                        vec.insert(loc + 1, Interval::new(index + 1, vec[loc].end));
                    }
                })
                .is_ok(),
        }
    }

    pub fn remove_range(&mut self, start: u32, end: u32) -> u64 {
        debug_assert!(start < end, "caller must ensure start < end");
        match *self {
            Array(ref mut vec) => {
                let a = vec.binary_search(&(start as u16)).unwrap_or_else(|e| e);
                let b = if end > u32::from(u16::max_value()) {
                    vec.len()
                } else {
                    vec.binary_search(&(end as u16)).unwrap_or_else(|e| e)
                };
                vec.drain(a..b);
                (b - a) as u64
            }
            Bitmap(ref mut bits) => {
                let start_key = key(start as u16) as usize;
                let start_bit = bit(start as u16) as u32;
                // end_key is inclusive
                let end_key = key((end - 1) as u16) as usize;
                let end_bit = bit(end as u16) as u32;

                if start_key == end_key {
                    let mask = (!0u64 << start_bit) & (!0u64).wrapping_shr(64 - end_bit);
                    let removed = (bits[start_key] & mask).count_ones();
                    bits[start_key] &= !mask;
                    return u64::from(removed);
                }

                let mut removed = 0;
                // start key bits
                removed += (bits[start_key] & (!0u64 << start_bit)).count_ones();
                bits[start_key] &= !(!0u64 << start_bit);
                // counts bits in between
                for word in &bits[start_key + 1..end_key] {
                    removed += word.count_ones();
                    // When popcnt is available zeroing in this loop is faster,
                    // but we opt to perform reasonably on most cpus by zeroing after.
                    // By doing that the compiler uses simd to count ones.
                }
                // do zeroing outside the loop
                for word in &mut bits[start_key + 1..end_key] {
                    *word = 0;
                }
                // end key bits
                removed += (bits[end_key] & (!0u64).wrapping_shr(64 - end_bit)).count_ones();
                bits[end_key] &= !(!0u64).wrapping_shr(64 - end_bit);
                u64::from(removed)
            }
            // TODO(jpg): Remove range
            Run(ref mut _intervals) => unimplemented!(),
        }
    }

    pub fn contains(&self, index: u16) -> bool {
        match *self {
            Array(ref vec) => vec.binary_search(&index).is_ok(),
            Bitmap(ref bits) => bits[key(index)] & (1 << bit(index)) != 0,
            Run(ref intervals) => intervals
                .binary_search_by_key(&index, |iv| iv.start)
                .is_ok(),
        }
    }

    pub fn is_disjoint<'a>(&'a self, other: &'a Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let (mut i1, mut i2) = (vec1.iter(), vec2.iter());
                let (mut value1, mut value2) = (i1.next(), i2.next());
                loop {
                    match value1.and_then(|v1| value2.map(|v2| v1.cmp(v2))) {
                        None => return true,
                        Some(Equal) => return false,
                        Some(Less) => value1 = i1.next(),
                        Some(Greater) => value2 = i2.next(),
                    }
                }
            }
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => bits1
                .iter()
                .zip(bits2.iter())
                .all(|(&i1, &i2)| (i1 & i2) == 0),
            (&Array(ref vec), store @ &Bitmap(..)) | (store @ &Bitmap(..), &Array(ref vec)) => {
                vec.iter().all(|&i| !store.contains(i))
            }
            // TODO(jpg) is_disjoint
            (&Run(ref _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&Run(ref _intervals), &Array(ref _vec)) | (&Array(ref _vec), &Run(ref _intervals)) => {
                unimplemented!()
            }
            (&Run(ref _intervals), _store @ &Bitmap(..))
            | (_store @ &Bitmap(..), &Run(ref _intervals)) => unimplemented!(),
        }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => {
                let (mut i1, mut i2) = (vec1.iter(), vec2.iter());
                let (mut value1, mut value2) = (i1.next(), i2.next());
                loop {
                    match (value1, value2) {
                        (None, _) => return true,
                        (Some(..), None) => return false,
                        (Some(v1), Some(v2)) => match v1.cmp(v2) {
                            Equal => {
                                value1 = i1.next();
                                value2 = i2.next();
                            }
                            Less => return false,
                            Greater => value2 = i2.next(),
                        },
                    }
                }
            }
            (&Array(ref vec), store @ &Bitmap(..)) => vec.iter().all(|&i| store.contains(i)),
            // TODO(jpg) is_subset array, run
            (&Array(ref _vec), &Run(ref _intervals)) => unimplemented!(),

            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => bits1
                .iter()
                .zip(bits2.iter())
                .all(|(&i1, &i2)| (i1 & i2) == i1),
            (&Bitmap(..), &Array(..)) => false,
            // TODO(jpg) is subset bitmap, run
            (&Bitmap(..), &Run(ref _vec)) => unimplemented!(),

            // TODO(jpg) is_subset run, *
            (&Run(ref _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&Run(ref _intervals), &Array(ref _vec)) => unimplemented!(),
            (&Run(ref _intervals), _store @ &Bitmap(..)) => unimplemented!(),
        }
    }

    pub fn to_array(&self) -> Self {
        match *self {
            Array(..) => panic!("Cannot convert array to array"),
            Bitmap(ref bits) => {
                let mut vec = Vec::new();
                for (key, val) in bits.iter().cloned().enumerate().filter(|&(_, v)| v != 0) {
                    for bit in 0..64 {
                        if (val & (1 << bit)) != 0 {
                            vec.push(key as u16 * 64 + bit as u16);
                        }
                    }
                }
                Array(vec)
            }
            Run(ref intervals) => Array(
                intervals
                    .iter()
                    .map(|iv| iv.start..iv.end)
                    .flatten()
                    .collect(),
            ),
        }
    }

    pub fn to_bitmap(&self) -> Self {
        match *self {
            Array(ref vec) => {
                let mut bits = Box::new([0; BITMAP_LENGTH]);
                for &index in vec {
                    bits[key(index)] |= 1 << bit(index);
                }
                Bitmap(bits)
            }
            Bitmap(..) => panic!("Cannot convert bitmap to bitmap"),
            Run(ref intervals) => {
                let mut bits = Box::new([0; BITMAP_LENGTH]);
                for iv in intervals {
                    for index in iv.start..iv.end {
                        bits[key(index)] |= 1 << bit(index);
                    }
                }
                Bitmap(bits)
            }
        }
    }

    pub fn to_run(&self) -> Self {
        match *self {
            Array(ref vec) => {
                let mut intervals = Vec::new();
                let mut start = *vec.first().unwrap();
                for (idx, &v) in vec[1..].iter().enumerate() {
                    if v - vec[idx] > 1 {
                        intervals.push(Interval::new(start, vec[idx]));
                        start = v
                    }
                }
                intervals.push(Interval::new(start, *vec.last().unwrap()));
                Run(intervals)
            }
            Bitmap(ref bits) => {
                let mut current = bits[0];
                let mut i = 0u16;
                let mut start;
                let mut last;

                let mut intervals = Vec::new();

                loop {
                    // Skip over empty words
                    while current == 0 && i < BITMAP_LENGTH as u16 - 1 {
                        i += 1;
                        current = bits[i as usize];
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
                        current = bits[i as usize];
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

    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0;
                let mut iter2 = vec2.iter();
                'outer: for &index2 in &mut iter2 {
                    while i1 < vec1.len() {
                        match vec1[i1].cmp(&index2) {
                            Less => i1 += 1,
                            Greater => vec1.insert(i1, index2),
                            Equal => continue 'outer,
                        }
                    }
                    vec1.push(index2);
                    break;
                }
                vec1.extend(iter2);
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            }
            // TODO(jpg) union_with array, run
            (&mut Array(ref mut _vec), &Run(ref _intervals)) => {}
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 |= index2;
                }
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec)) => {
                for &index in vec {
                    this.insert(index);
                }
            }
            // TODO(jpg) union_with bitmap, run
            (ref mut _this @ &mut Bitmap(..), &Run(ref _intervals)) => unimplemented!(),
            // TODO(jpg) union_with run, *
            (&mut Run(ref mut _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&mut Run(ref mut _intervals), &Array(ref _vec)) => unimplemented!(),
            (this @ &mut Run(..), &Bitmap(..)) => {
                *this = this.to_bitmap();
                this.union_with(other);
            }
        }
    }

    pub fn intersect_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None | Some(Less) => {
                            vec1.remove(i1);
                        }
                        Some(Greater) => {
                            current2 = iter2.next();
                        }
                        Some(Equal) => {
                            i1 += 1;
                            current2 = iter2.next();
                        }
                    }
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                for i in (0..(vec.len())).rev() {
                    if !store.contains(vec[i]) {
                        vec.remove(i);
                    }
                }
            }
            // TODO(jpg) intersect_with array, run
            (&mut Array(ref mut _intervals1), &Run(ref _intervals2)) => {}
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= index2;
                }
            }
            (this @ &mut Bitmap(..), &Array(..)) => {
                let mut new = other.clone();
                new.intersect_with(this);
                *this = new;
            }
            // TODO(jpg) intersect_with bitmap, run
            (_this @ &mut Bitmap(..), &Run(..)) => unimplemented!(),
            // TODO(jpg) intersect_with run, *
            (&mut Run(ref mut _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&mut Run(ref mut _intervals), &Array(ref _vec)) => unimplemented!(),
            (&mut Run(ref mut _intervals), _store @ &Bitmap(..)) => unimplemented!(),
        }
    }

    pub fn difference_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None => break,
                        Some(Less) => {
                            i1 += 1;
                        }
                        Some(Greater) => {
                            current2 = iter2.next();
                        }
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        }
                    }
                }
            }
            (&mut Array(ref mut vec), store @ &Bitmap(..)) => {
                for i in (0..vec.len()).rev() {
                    if store.contains(vec[i]) {
                        vec.remove(i);
                    }
                }
            }
            // TODO(jpg) difference_with array, run
            (&mut Array(ref mut _vec), &Run(ref _intervals)) => unimplemented!(),

            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    this.remove(*index);
                }
            }
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 &= !*index2;
                }
            }
            // TODO(jpg) difference_with bitmap, run
            (ref mut _this @ &mut Bitmap(..), &Run(ref _intervals)) => unimplemented!(),

            // TODO(jpg) difference_with run, *
            (&mut Run(ref mut _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&mut Run(ref mut _intervals), &Array(ref _vec)) => unimplemented!(),
            (&mut Run(ref mut _vec), _store @ &Bitmap(..)) => unimplemented!(),
        }
    }

    pub fn symmetric_difference_with(&mut self, other: &Self) {
        match (self, other) {
            (&mut Array(ref mut vec1), &Array(ref vec2)) => {
                let mut i1 = 0usize;
                let mut iter2 = vec2.iter();
                let mut current2 = iter2.next();
                while i1 < vec1.len() {
                    match current2.map(|c2| vec1[i1].cmp(c2)) {
                        None => break,
                        Some(Less) => {
                            i1 += 1;
                        }
                        Some(Greater) => {
                            vec1.insert(i1, *current2.unwrap());
                            i1 += 1;
                            current2 = iter2.next();
                        }
                        Some(Equal) => {
                            vec1.remove(i1);
                            current2 = iter2.next();
                        }
                    }
                }
                if let Some(current) = current2 {
                    vec1.push(*current);
                    vec1.extend(iter2.cloned());
                }
            }
            (this @ &mut Array(..), &Bitmap(..)) => {
                let mut new = other.clone();
                new.symmetric_difference_with(this);
                *this = new;
            }
            // TODO(jpg) symmetric_difference_with array, run
            (&mut Array(ref mut _vec), &Run(ref _intervals)) => {}
            (&mut Bitmap(ref mut bits1), &Bitmap(ref bits2)) => {
                for (index1, &index2) in bits1.iter_mut().zip(bits2.iter()) {
                    *index1 ^= index2;
                }
            }
            (ref mut this @ &mut Bitmap(..), &Array(ref vec2)) => {
                for index in vec2.iter() {
                    if this.contains(*index) {
                        this.remove(*index);
                    } else {
                        this.insert(*index);
                    }
                }
            }
            // TODO(jpg) symmetric_difference_with bitmap, run
            (ref mut _this @ &mut Bitmap(..), &Run(ref _vec)) => unimplemented!(),
            // TODO(jpg) symmetric_difference_with run, *
            (&mut Run(ref mut _intervals1), &Run(ref _intervals2)) => unimplemented!(),
            (&mut Run(ref mut _intervals), &Array(ref _vec)) => unimplemented!(),
            (_this @ &mut Run(..), &Bitmap(..)) => unimplemented!(),
        }
    }

    pub fn len(&self) -> u64 {
        match *self {
            Array(ref vec) => vec.len() as u64,
            Bitmap(ref bits) => bits.iter().map(|bit| u64::from(bit.count_ones())).sum(),
            Run(ref intervals) => intervals.iter().map(|iv| iv.run_len() as u64).sum(),
        }
    }

    pub fn min(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.first().unwrap(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| index * 64 + (bit.trailing_zeros() as usize))
                .unwrap() as u16,
            Run(ref intervals) => intervals.first().unwrap().start,
        }
    }

    pub fn max(&self) -> u16 {
        match *self {
            Array(ref vec) => *vec.last().unwrap(),
            Bitmap(ref bits) => bits
                .iter()
                .enumerate()
                .rev()
                .find(|&(_, &bit)| bit != 0)
                .map(|(index, bit)| index * 64 + (63 - bit.leading_zeros() as usize))
                .unwrap() as u16,
            Run(ref intervals) => intervals.last().unwrap().end,
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
                    let word = bits[i];
                    let next_word = bits[i + 1];
                    num_runs +=
                        ((word << 1) & !word).count_ones() as u64 + ((word >> 63) & !next_word);
                }

                let last = bits[BITMAP_LENGTH - 1];
                num_runs += ((last << 1) & !last).count_ones() as u64 + (last >> 63);
                num_runs
            }
            Run(ref intervals) => intervals.len() as u64,
        }
    }
}

impl<'a> IntoIterator for &'a Store {
    type Item = u16;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        match *self {
            Array(ref vec) => Iter::Array(vec.iter()),
            Bitmap(ref bits) => Iter::BitmapBorrowed(BitmapIter::new(&**bits)),
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
            Bitmap(bits) => Iter::BitmapOwned(BitmapIter::new(bits)),
            Run(intervals) => Iter::Run(RunIter::new(intervals)),
        }
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Array(ref vec1), &Array(ref vec2)) => vec1 == vec2,
            (&Bitmap(ref bits1), &Bitmap(ref bits2)) => {
                bits1.iter().zip(bits2.iter()).all(|(i1, i2)| i1 == i2)
            }
            (&Run(ref intervals1), &Run(ref intervals2)) => intervals1 == intervals2,
            _ => false,
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        match *self {
            Array(ref vec) => Array(vec.clone()),
            Bitmap(ref bits) => Bitmap(Box::new(**bits)),
            Run(ref intervals) => Run(intervals.clone().to_vec()),
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

impl<B: Borrow<[u64; BITMAP_LENGTH]>> BitmapIter<B> {
    fn new(bits: B) -> BitmapIter<B> {
        BitmapIter {
            key: 0,
            bit: 0,
            bits,
        }
    }

    fn move_next(&mut self) {
        self.bit += 1;
        if self.bit == 64 {
            self.bit = 0;
            self.key += 1;
        }
    }
}

impl<B: Borrow<[u64; BITMAP_LENGTH]>> Iterator for BitmapIter<B> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if self.key == BITMAP_LENGTH {
                return None;
            } else if (unsafe { self.bits.borrow().get_unchecked(self.key) } & (1u64 << self.bit))
                != 0
            {
                let result = Some((self.key * 64 + self.bit) as u16);
                self.move_next();
                return result;
            } else {
                self.move_next();
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match *self {
            Iter::Array(ref mut inner) => inner.next().cloned(),
            Iter::Vec(ref mut inner) => inner.next(),
            Iter::BitmapBorrowed(ref mut inner) => inner.next(),
            Iter::BitmapOwned(ref mut inner) => inner.next(),
            Iter::Run(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        panic!("Should never be called (roaring::Iter caches the size_hint itself)")
    }
}

#[inline]
fn key(index: u16) -> usize {
    index as usize / 64
}

#[inline]
fn bit(index: u16) -> usize {
    index as usize % 64
}

impl fmt::Debug for Store {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Array(_) => format!(
                "Array<{} values from {} to {}>",
                self.len(),
                self.min(),
                self.max()
            )
            .fmt(formatter),
            Bitmap(_) => format!(
                "Bitmap<{} bits set from {} to {}>",
                self.len(),
                self.min(),
                self.max()
            )
            .fmt(formatter),
            Run(intervals) => format!(
                "Run<{} runs totalling {} values from {} to {}>",
                intervals.len(),
                self.len(),
                self.min(),
                self.max()
            )
            .fmt(formatter),
        }
    }
}
