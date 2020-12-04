#![allow(clippy::from_iter_instead_of_collect)]

extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn array() {
    let bitmap1 = RoaringBitmap::from_iter(0..2000);
    let bitmap2 = RoaringBitmap::from_iter(4000..6000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn array_not() {
    let bitmap1 = RoaringBitmap::from_iter(0..4000);
    let bitmap2 = RoaringBitmap::from_iter(2000..6000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmap() {
    let bitmap1 = RoaringBitmap::from_iter(0..6000);
    let bitmap2 = RoaringBitmap::from_iter(10000..16000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmap_not() {
    let bitmap1 = RoaringBitmap::from_iter(0..10000);
    let bitmap2 = RoaringBitmap::from_iter(5000..15000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn arrays() {
    let bitmap1 = RoaringBitmap::from_iter(
        (0..2000)
            .chain(1_000_000..1_002_000)
            .chain(2_000_000..2_002_000),
    );
    let bitmap2 = RoaringBitmap::from_iter((100_000..102_000).chain(1_100_000..1_102_000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn arrays_not() {
    let bitmap1 = RoaringBitmap::from_iter(
        (0..2_000)
            .chain(1_000_000..1_002_000)
            .chain(2_000_000..2_002_000),
    );
    let bitmap2 = RoaringBitmap::from_iter((100_000..102_000).chain(1_001_000..1_003_000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmaps() {
    let bitmap1 = RoaringBitmap::from_iter(
        (0..6000)
            .chain(1_000_000..1_006_000)
            .chain(2_000_000..2_006_000),
    );
    let bitmap2 = RoaringBitmap::from_iter((100_000..106_000).chain(1_100_000..1_106_000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmaps_not() {
    let bitmap1 = RoaringBitmap::from_iter(
        (0..6000)
            .chain(1_000_000..1_006_000)
            .chain(2_000_000..2_006_000),
    );
    let bitmap2 = RoaringBitmap::from_iter((100_000..106_000).chain(1_004_000..1_008_000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}
