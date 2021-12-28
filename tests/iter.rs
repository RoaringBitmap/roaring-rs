extern crate roaring;

use std::iter::FromIterator;
use std::ops::Range;

use quickcheck_macros::quickcheck;

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

#[quickcheck]
fn qc_iter(values: Vec<u32>) {
    let expected = {
        let mut vec = values.clone();
        vec.sort();
        vec.dedup();
        vec
    };

    let b = RoaringBitmap::from_iter(values.into_iter());
    assert_eq!(b.iter().collect::<Vec<u32>>(), expected);
}
