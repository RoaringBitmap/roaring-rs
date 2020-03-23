extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = RoaringTreemap::from_iter(0..2000);
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmap() {
    let original = RoaringTreemap::from_iter(0..6000);
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn arrays() {
    let original = RoaringTreemap::from_iter(
        (0..2000)
            .chain(1_000_000..1_002_000)
            .chain(2_000_000..2_001_000),
    );
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps() {
    let original = RoaringTreemap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(2_000_000..2_010_000),
    );
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps_iterator() {
    let original = RoaringTreemap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(2_000_000..2_010_000),
    );
    let clone = RoaringTreemap::from_bitmaps(original.bitmaps().map(|(p, b)| (p, b.clone())));
    let clone2 = RoaringTreemap::from_iter(original.bitmaps().map(|(p, b)| (p, b.clone())));

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}
