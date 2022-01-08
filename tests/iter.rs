extern crate roaring;

use proptest::arbitrary::any;
use proptest::collection::btree_set;
use proptest::proptest;
use std::iter::FromIterator;

use roaring::RoaringBitmap;

#[test]
fn array() {
    let original = (0..2000).collect::<RoaringBitmap>();
    let clone = RoaringBitmap::from_iter(&original);
    let clone2 = RoaringBitmap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmap() {
    let original = (0..6000).collect::<RoaringBitmap>();
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
    let original = (0..6000)
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
        assert!(values.into_iter().eq(bitmap.into_iter()));
    }
}

#[test]
fn rev_iter() {
    use roaring::RoaringBitmap;

    let input: Vec<u32> = (1..3).chain(1_000_000..1_012_003).chain(2_000_001..2_000_003).collect();

    let expected = input.clone().into_iter().rev().collect::<Vec<_>>();
    let bitmap = input.into_iter().collect::<RoaringBitmap>();

    let iter = bitmap.into_rev_iter().collect::<Vec<_>>();

    assert_eq!(iter, expected);
}
