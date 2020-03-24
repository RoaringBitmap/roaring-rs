extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = RoaringTreemap::from_iter(0..2000);
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original = RoaringTreemap::from_iter(0..6000);
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original = RoaringTreemap::from_iter(
        (0..2000)
            .chain(1_000_000..1_002_000)
            .chain(2_000_000..2_001_000),
    );
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original = RoaringTreemap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(2_000_000..2_010_000),
    );
    let clone = original.clone();

    assert_eq!(clone, original);
}
