#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let mut bitmap1: RoaringBitmap = (0..2000).collect();
    let bitmap2: RoaringBitmap = (1000..3000).collect();
    let bitmap3: RoaringBitmap = (1000..2000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1: RoaringBitmap = (0..2000).collect();
    let bitmap2: RoaringBitmap = (1000..8000).collect();
    let bitmap3: RoaringBitmap = (1000..2000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1: RoaringBitmap = (0..12000).collect();
    let bitmap2: RoaringBitmap = (6000..18000).collect();
    let bitmap3: RoaringBitmap = (6000..12000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1: RoaringBitmap = (0..6000).collect();
    let bitmap2: RoaringBitmap = (3000..9000).collect();
    let bitmap3: RoaringBitmap = (3000..6000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array() {
    let mut bitmap1: RoaringBitmap = (0..12000).collect();
    let bitmap2: RoaringBitmap = (7000..9000).collect();
    let bitmap3: RoaringBitmap = (7000..9000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1: RoaringBitmap = (0..2000).chain(1000000..1002000).chain(3000000..3001000).collect();
    let bitmap2: RoaringBitmap = (1000..3000).chain(1001000..1003000).chain(2000000..2001000).collect();
    let bitmap3: RoaringBitmap = (1000..2000).chain(1001000..1002000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1: RoaringBitmap = (0..6000).chain(1000000..1012000).chain(3000000..3010000).collect();
    let bitmap2: RoaringBitmap = (3000..9000).chain(1006000..1018000).chain(2000000..2010000).collect();
    let bitmap3: RoaringBitmap = (3000..6000).chain(1006000..1012000).collect();

    bitmap1.intersect_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
