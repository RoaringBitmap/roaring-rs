extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..3000);
    let bitmap3 = RoaringTreemap::from_iter(1000..2000);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn no_intersection() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2);
    let bitmap2 = RoaringTreemap::from_iter(3..4);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, RoaringTreemap::new());
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..2000);
    let bitmap2 = RoaringTreemap::from_iter(1000..8000);
    let bitmap3 = RoaringTreemap::from_iter(1000..2000);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(6000..18000);
    let bitmap3 = RoaringTreemap::from_iter(6000..12000);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..6000);
    let bitmap2 = RoaringTreemap::from_iter(3000..9000);
    let bitmap3 = RoaringTreemap::from_iter(3000..6000);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array() {
    let mut bitmap1 = RoaringTreemap::from_iter(0..12000);
    let bitmap2 = RoaringTreemap::from_iter(7000..9000);
    let bitmap3 = RoaringTreemap::from_iter(7000..9000);

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1 = RoaringTreemap::from_iter((0..2000).chain(1000000..1002000).chain(3000000..3001000));
    let bitmap2 = RoaringTreemap::from_iter((1000..3000).chain(1001000..1003000).chain(2000000..2001000));
    let bitmap3 = RoaringTreemap::from_iter((1000..2000).chain(1001000..1002000));

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1 = RoaringTreemap::from_iter((0..6000).chain(1000000..1012000).chain(3000000..3010000));
    let bitmap2 = RoaringTreemap::from_iter((3000..9000).chain(1006000..1018000).chain(2000000..2010000));
    let bitmap3 = RoaringTreemap::from_iter((3000..6000).chain(1006000..1012000));

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
