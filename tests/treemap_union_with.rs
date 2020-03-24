extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array_to_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..3000);
    let bitmap3 = RoaringTreemap::from_iter(0..3000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_to_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..4000);
    let bitmap2 = RoaringTreemap::from_iter(4000..8000);
    let bitmap3 = RoaringTreemap::from_iter(0..8000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..8000);
    let bitmap3 = RoaringTreemap::from_iter(0..8000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(6000..18000);
    let bitmap3 = RoaringTreemap::from_iter(0..18000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(10000..13000);
    let bitmap3 = RoaringTreemap::from_iter(0..13000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1 = RoaringTreemap::from_iter(
        (0..2000)
            .chain(1_000_000..1_002_000)
            .chain(3_000_000..3_001_000),
    );
    let bitmap2 = RoaringTreemap::from_iter(
        (1000..3000)
            .chain(1_001_000..1_003_000)
            .chain(2_000_000..2_001_000),
    );
    let bitmap3 = RoaringTreemap::from_iter(
        (0..3000)
            .chain(1_000_000..1_003_000)
            .chain(2_000_000..2_001_000)
            .chain(3_000_000..3_001_000),
    );

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1 = RoaringTreemap::from_iter(
        (0..6000)
            .chain(1_000_000..1_012_000)
            .chain(3_000_000..3_010_000),
    );
    let bitmap2 = RoaringTreemap::from_iter(
        (3000..9000)
            .chain(1_006_000..1_018_000)
            .chain(2_000_000..2_010_000),
    );
    let bitmap3 = RoaringTreemap::from_iter(
        (0..9000)
            .chain(1_000_000..1_018_000)
            .chain(2_000_000..2_010_000)
            .chain(3_000_000..3_010_000),
    );

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
