use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use std::mem;
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Num(pub u32);

pub const MAX_NUM: u32 = 0x1_0000 * 4;

impl<'a> Arbitrary<'a> for Num {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(u.int_in_range(0..=(MAX_NUM - 1))?))
    }
}

#[derive(Arbitrary, Debug)]
pub enum Operation {
    Binary(BitmapBinaryOperation),
    MutateLhs(MutableBitmapOperation),
    Read(ReadBitmapOperation),
    SwapSides,
}

impl Operation {
    pub fn apply(
        &self,
        lhs_c: &mut croaring::Bitmap,
        rhs_c: &mut croaring::Bitmap,
        lhs_r: &mut roaring::RoaringBitmap,
        rhs_r: &mut roaring::RoaringBitmap,
    ) {
        match self {
            Operation::Binary(op) => op.apply(lhs_c, rhs_c, lhs_r, rhs_r),
            Operation::MutateLhs(op) => op.apply(lhs_c, lhs_r),
            Operation::Read(op) => op.apply(lhs_c, lhs_r),
            Operation::SwapSides => {
                mem::swap(lhs_c, rhs_c);
                mem::swap(lhs_r, rhs_r);
            }
        }
    }
}

#[derive(Arbitrary, Debug)]
pub enum MutableBitmapOperation {
    Insert(Num),
    InsertRange(RangeInclusive<Num>),
    Push(Num),
    Remove(Num),
    RemoveRange(RangeInclusive<Num>),
    Clear,
    Extend(Vec<Num>),
    SwapSerialization,
    Optimize,
    // TODO: not implemented in roaring-rs yet
    // RemoveRunCompression,
    // Probably turn it into a bitmap
    MakeBitmap { key: u16 },
    // Probably turn it into a Range
    MakeRange { key: u16 },
}

#[derive(Arbitrary, Debug, Copy, Clone)]
pub enum RangeOperations {
    Optimized,
    Removed,
}

#[derive(Arbitrary, Debug)]
pub enum ReadBitmapOperation {
    ContainsRange(RangeInclusive<Num>),
    Contains(Num),
    RangeCardinality(RangeInclusive<Num>),
    Cardinality,
    IsEmpty,
    IsFull,
    Minimum,
    Maximum,
    Rank(Num),
    Select(Num),
    Statistics(RangeOperations),
    Clone,
    Debug,
    SerializedSize(RangeOperations),
    Serialize(RangeOperations),
}

#[derive(Arbitrary, Debug)]
pub enum BitmapBinaryOperation {
    Eq,
    IsSubset,
    And,
    Or,
    Xor,
    AndNot,
}

impl ReadBitmapOperation {
    pub fn apply(&self, x: &mut croaring::Bitmap, y: &mut roaring::RoaringBitmap) {
        match *self {
            ReadBitmapOperation::ContainsRange(ref range) => {
                let range = range.start().0..=range.end().0;
                let expected = x.contains_range(range.clone());
                let actual = y.contains_range(range);
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Contains(Num(n)) => {
                let expected = x.contains(n);
                let actual = y.contains(n);
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::RangeCardinality(ref range) => {
                let range = range.start().0..=range.end().0;
                let expected = x.range_cardinality(range.clone());
                let actual = y.range_cardinality(range);
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Cardinality => {
                let expected = x.cardinality();
                let actual = y.len();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::IsEmpty => {
                let expected = x.is_empty();
                let actual = y.is_empty();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::IsFull => {
                let expected = x.contains_range(..);
                let actual = y.is_full();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Minimum => {
                let expected = x.minimum();
                let actual = y.min();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Maximum => {
                let expected = x.maximum();
                let actual = y.max();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Rank(Num(n)) => {
                let expected = x.rank(n);
                let actual = y.rank(n);
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Select(Num(n)) => {
                let expected = x.select(n);
                let actual = y.select(n);
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Statistics(ranges) => {
                match ranges {
                    RangeOperations::Optimized => {
                        x.run_optimize();
                        y.optimize();

                    }
                    RangeOperations::Removed => {
                        // TODO: Not implemented in roaring-rs yet
                        return;
                    }
                }
                let expected = x.statistics();
                let actual = y.statistics();
                // Convert to the same statistics struct
                let expected = {
                    let mut v = actual;
                    v.n_containers = expected.n_containers;
                    v.n_array_containers = expected.n_array_containers;
                    v.n_run_containers = expected.n_run_containers;
                    v.n_bitset_containers = expected.n_bitset_containers;
                    v.n_values_array_containers = expected.n_values_array_containers;
                    v.n_values_run_containers = expected.n_values_run_containers;
                    v.n_values_bitset_containers = expected.n_values_bitset_containers.into();
                    // The n_bytes_* fields are not directly comparable:
                    // they are based on the number of bytes of _capacity_ of the
                    // containers, which depends on the allocation strategy.
                    // v.n_bytes_array_containers = expected.n_bytes_array_containers.into();
                    // v.n_bytes_run_containers = expected.n_bytes_run_containers.into();
                    // v.n_bytes_bitset_containers = expected.n_bytes_bitset_containers.into();
                    v.max_value = x.maximum();
                    v.min_value = x.minimum();
                    v.cardinality = x.cardinality();
                    v
                };
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Clone => {
                assert_eq!(*y, y.clone());
            }
            ReadBitmapOperation::Debug => {
                use std::io::Write;
                write!(std::io::sink(), "{:?}", y).unwrap();
            }
            ReadBitmapOperation::SerializedSize(ranges) => {
                match ranges {
                    RangeOperations::Optimized => {
                        x.run_optimize();
                        y.optimize();

                    }
                    RangeOperations::Removed => {
                        // TODO: Not implemented in roaring-rs yet
                        return;
                    }
                }
                let expected = x.get_serialized_size_in_bytes::<croaring::Portable>();
                let actual = y.serialized_size();
                assert_eq!(expected, actual);
            }
            ReadBitmapOperation::Serialize(ranges) => {
                match ranges {
                    RangeOperations::Optimized => {
                        x.run_optimize();
                        y.optimize();

                    }
                    RangeOperations::Removed => {
                        // TODO: Not implemented in roaring-rs yet
                        return;
                    }
                }
                let expected = x.serialize::<croaring::Portable>();
                let mut actual = Vec::new();
                y.serialize_into(&mut actual).unwrap();
                assert_eq!(expected, actual);
            }
        }
    }
}

impl MutableBitmapOperation {
    pub fn apply(&self, x: &mut croaring::Bitmap, y: &mut roaring::RoaringBitmap) {
        match *self {
            MutableBitmapOperation::Insert(Num(n)) => {
                let expected = x.add_checked(n);
                let actual = y.insert(n);
                assert_eq!(expected, actual);
            }
            MutableBitmapOperation::InsertRange(ref range) => {
                let range = range.start().0..=range.end().0;
                let expected_added = u64::try_from(range.clone().count()).unwrap()
                    - x.range_cardinality(range.clone());
                x.add_range(range.clone());
                assert_eq!(expected_added, y.insert_range(range));
            }
            MutableBitmapOperation::Push(Num(n)) => {
                let should_push = y.max().is_none_or(|max| n > max);
                if should_push {
                    x.add(n);
                }
                assert_eq!(should_push, y.push(n));
            }
            MutableBitmapOperation::Remove(Num(n)) => {
                let expected = x.remove_checked(n);
                let actual = y.remove(n);
                assert_eq!(expected, actual);
            }
            MutableBitmapOperation::RemoveRange(ref range) => {
                let range = range.start().0..=range.end().0;
                let expected_removed = x.range_cardinality(range.clone());
                x.remove_range(range.clone());
                assert_eq!(expected_removed, y.remove_range(range));
            }
            MutableBitmapOperation::Clear => {
                x.clear();
                y.clear();
            }
            MutableBitmapOperation::Optimize => {
                x.run_optimize();
                y.optimize();
                let stat_x = x.statistics();
                let stat_y = y.statistics();
                assert_eq!(stat_x.n_run_containers, stat_y.n_run_containers);
                assert_eq!(stat_x.n_bitset_containers, stat_y.n_bitset_containers);
                assert_eq!(stat_x.n_array_containers, stat_y.n_array_containers);
            }
            MutableBitmapOperation::Extend(ref items) => {
                // Safety - Num is repr(transparent) over u32
                let items: &[u32] = unsafe { mem::transmute(&items[..]) };
                x.add_many(items);
                y.extend(items);
            }
            MutableBitmapOperation::SwapSerialization => {
                let x_serialized = x.serialize::<croaring::Portable>();
                let mut y_serialized = Vec::new();
                y.serialize_into(&mut y_serialized).unwrap();

                let new_x = croaring::Bitmap::try_deserialize::<croaring::Portable>(&y_serialized).unwrap();
                let new_y = roaring::RoaringBitmap::deserialize_from(&x_serialized[..]).unwrap();
                assert_eq!(new_x, *x);
                assert_eq!(new_y, *y);
                *x = new_x;
                *y = new_y;
            }
            MutableBitmapOperation::MakeBitmap { key } => {
                let key = u32::from(key);
                let start = key * 0x1_0000;
                let end = start + 9 * 1024;
                for i in (start..end).step_by(2) {
                    x.add(i);
                    y.insert(i);
                }
            }
            MutableBitmapOperation::MakeRange { key } => {
                let key = u32::from(key);
                let start = key * 0x1_0000;
                let end = start + 9 * 1024;
                x.add_range(start..=end);
                y.insert_range(start..=end);
            }
        }
    }
}

impl BitmapBinaryOperation {
    pub fn apply(
        &self,
        lhs_c: &mut croaring::Bitmap,
        rhs_c: &croaring::Bitmap,
        lhs_r: &mut roaring::RoaringBitmap,
        rhs_r: &roaring::RoaringBitmap,
    ) {
        match *self {
            BitmapBinaryOperation::Eq => {
                let expected = lhs_c == rhs_c;
                let actual = lhs_r == rhs_r;
                assert_eq!(expected, actual);
            }
            BitmapBinaryOperation::IsSubset => {
                let expected = lhs_c.is_subset(rhs_c);
                let actual = lhs_r.is_subset(rhs_r);
                assert_eq!(expected, actual);
            }
            BitmapBinaryOperation::And => {
                let expected_len = lhs_r.intersection_len(rhs_r);
                let actual_len = lhs_c.and_cardinality(rhs_c);
                assert_eq!(expected_len, actual_len);

                *lhs_r &= rhs_r;
                *lhs_c &= rhs_c;
                assert_eq!(lhs_r.len(), expected_len);
            }
            BitmapBinaryOperation::Or => {
                let expected_len = lhs_r.union_len(rhs_r);
                let actual_len = lhs_c.or_cardinality(rhs_c);
                assert_eq!(expected_len, actual_len);

                *lhs_r |= rhs_r;
                *lhs_c |= rhs_c;
                assert_eq!(lhs_r.len(), expected_len);
            }
            BitmapBinaryOperation::Xor => {
                let expected_len = lhs_r.symmetric_difference_len(rhs_r);
                let actual_len = lhs_c.xor_cardinality(rhs_c);
                assert_eq!(expected_len, actual_len);

                *lhs_r ^= rhs_r;
                *lhs_c ^= rhs_c;
                assert_eq!(lhs_r.len(), expected_len);
            }
            BitmapBinaryOperation::AndNot => {
                let expected_len = lhs_r.difference_len(rhs_r);
                let actual_len = lhs_c.andnot_cardinality(rhs_c);
                assert_eq!(expected_len, actual_len);

                *lhs_r -= rhs_r;
                *lhs_c -= rhs_c;
                assert_eq!(lhs_r.len(), expected_len);
            }
        }
    }
}

pub(crate) fn check_equal(c: &croaring::Bitmap, r: &roaring::RoaringBitmap) {
    let mut lhs = c.iter();
    let mut rhs = r.iter();

    loop {
        match (lhs.next(), rhs.next()) {
            (Some(l), Some(r)) => {
                assert_eq!(l, r);
            }
            (None, None) => break,
            (Some(n), None) => panic!("croaring has more elements: {n}"),
            (None, Some(n)) => panic!("roaring has more elements: {n}"),
        }
    }
}
