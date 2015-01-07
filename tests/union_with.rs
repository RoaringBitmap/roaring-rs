#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array_to_array() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..3000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_to_bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..4000).collect();
    let bitmap2: RoaringBitmap<u32> = (4000..8000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..8000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..8000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..8000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..12000).collect();
    let bitmap2: RoaringBitmap<u32> = (6000..18000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..18000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array() {
    let mut bitmap1: RoaringBitmap<u32> = (0..12000).collect();
    let bitmap2: RoaringBitmap<u32> = (10000..13000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..13000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000).chain(1000000..1002000).chain(3000000..3001000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000).chain(1001000..1003000).chain(2000000..2001000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..3000).chain(1000000..1003000).chain(2000000..2001000).chain(3000000..3001000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1: RoaringBitmap<u32> = (0..6000).chain(1000000..1012000).chain(3000000..3010000).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000).chain(1006000..1018000).chain(2000000..2010000).collect();
    let bitmap3: RoaringBitmap<u32> = (0..9000).chain(1000000..1018000).chain(2000000..2010000).chain(3000000..3010000).collect();

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
