use proptest::arbitrary::any;
use proptest::collection::btree_set;
use proptest::proptest;
use std::iter::FromIterator;

use roaring::RoaringBitmap;

#[test]
fn range() {
    let original = (0..2000).collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from_iter(&original);
    let clone2 = RoaringBitmap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn array() {
    let original = (0..5).collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from([0, 1, 2, 3, 4]);

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original = (0..100_000).collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from_iter(&original);
    let clone2 = RoaringBitmap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn arrays() {
    let original = (0..2000)
        .chain(1_000_000..1_002_000)
        .chain(2_000_000..2_001_000)
        .collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from_iter(&original);
    let clone2 = RoaringBitmap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps() {
    let original = (0..100_000)
        .chain(1_000_000..1_012_000)
        .chain(2_000_000..2_010_000)
        .collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from_iter(&original);
    let clone2 = RoaringBitmap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

proptest! {
    #[test]
    fn iter(values in btree_set(any::<u32>(), ..=10_000)) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();
        // Iterator::eq != PartialEq::eq - cannot use assert_eq macro
        assert!(values.into_iter().eq(bitmap));
    }
}

#[test]
fn rev_array() {
    let values = 0..100;
    let bitmap = values.clone().collect::<RoaringBitmap>();

    assert!(values.into_iter().rev().eq(bitmap.iter().rev()));
}

#[test]
fn rev_bitmap() {
    let values = 0..=100_000;
    let bitmap = values.clone().collect::<RoaringBitmap>();

    assert!(values.into_iter().rev().eq(bitmap.iter().rev()));
}

proptest! {
    #[test]
    fn rev_iter(values in btree_set(any::<u32>(), ..=10_000)) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();

        assert!(values.into_iter().rev().eq(bitmap.iter().rev()));
    }
}

#[test]
fn from_iter() {
    // This test verifies that the public API allows conversion from iterators
    // with u32 as well as &u32 elements.
    let vals = vec![1, 5, 10000];
    let a = RoaringBitmap::from_iter(vals.iter());
    let b = RoaringBitmap::from_iter(vals);
    assert_eq!(a, b);
}

#[derive(Clone, Debug)]
pub struct OutsideInIter<T>(bool, T);

impl<T, I> Iterator for OutsideInIter<I>
where
    I: DoubleEndedIterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let res = if self.0 { self.1.next() } else { self.1.next_back() };
        self.0 = !self.0;
        res
    }
}

pub fn outside_in<U, I>(into_iter: I) -> OutsideInIter<U>
where
    U: DoubleEndedIterator,
    I: IntoIterator<IntoIter = U>,
{
    OutsideInIter(true, into_iter.into_iter())
}

// Sanity check that outside_in does what we expect
#[test]
fn outside_in_iterator() {
    let values = 0..10;
    assert!(outside_in(values).eq(vec![0, 9, 1, 8, 2, 7, 3, 6, 4, 5]));
}

#[test]
fn interleaved_array() {
    let values = 0..100;
    let bitmap = values.clone().collect::<RoaringBitmap>();

    assert!(outside_in(values).eq(outside_in(bitmap)));
}

#[test]
fn interleaved_bitmap() {
    let values = 0..=4097;
    let bitmap = values.clone().collect::<RoaringBitmap>();

    assert!(outside_in(values).eq(outside_in(bitmap)));
}

proptest! {
    #[test]
    fn interleaved_iter(values in btree_set(any::<u32>(), 50_000..=100_000)) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();

        assert!(outside_in(values).eq(outside_in(bitmap)));
    }
}
