extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = RoaringBitmap::from_iter(0..2000);
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original = RoaringBitmap::from_iter(0..6000);
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original = RoaringBitmap::from_iter(
        (0..2000)
            .chain(1_000_000..1_002_000)
            .chain(2_000_000..2_001_000),
    );
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original = RoaringBitmap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(2_000_000..2_010_000),
    );
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn runs() {
    let mut original = RoaringBitmap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(2_000_000..2_010_000),
    );
    original.optimize();
    let clone = original.clone();

    assert_eq!(clone, original);
}
