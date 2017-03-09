extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..3000);
    let bitmap3 = RoaringTreemap::from_iter((0..1000).chain(2000..3000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn no_symmetric_difference() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2);
    let bitmap2 = RoaringTreemap::from_iter(0..2);

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, RoaringTreemap::new());
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..8000);
    let bitmap3 = RoaringTreemap::from_iter((0..1000).chain(2000..8000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(6000..18000);
    let bitmap3 = RoaringTreemap::from_iter((0..6000).chain(12000..18000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..6000);
    let bitmap2 = RoaringTreemap::from_iter(2000..7000);
    let bitmap3 = RoaringTreemap::from_iter((0..2000).chain(6000..7000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(11000..14000);
    let bitmap3 = RoaringTreemap::from_iter((0..11000).chain(12000..14000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..6000);
    let bitmap2 = RoaringTreemap::from_iter(3000..7000);
    let bitmap3 = RoaringTreemap::from_iter((0..3000).chain(6000..7000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1 = RoaringTreemap::from_iter((0..2000).chain(1000000..1002000).chain(3000000..3001000));
    let bitmap2 = RoaringTreemap::from_iter((1000..3000).chain(1001000..1003000).chain(2000000..2000001));
    let bitmap3 = RoaringTreemap::from_iter((0..1000).chain(1000000..1001000).chain(2000..3000).chain(1002000..1003000).chain(2000000..2000001).chain(3000000..3001000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1 = RoaringTreemap::from_iter((0..6000).chain(1000000..1012000).chain(3000000..3010000));
    let bitmap2 = RoaringTreemap::from_iter((3000..7000).chain(1006000..1018000).chain(2000000..2010000));
    let bitmap3 = RoaringTreemap::from_iter((0..3000).chain(1000000..1006000).chain(6000..7000).chain(1012000..1018000).chain(2000000..2010000).chain(3000000..3010000));

    bitmap1.symmetric_difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
