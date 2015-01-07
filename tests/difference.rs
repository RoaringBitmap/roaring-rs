#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap<u32> = (0..2000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000).collect();

    let expected: RoaringBitmap<u32> = (0..1000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn array_and_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..2000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..8000).collect();

    let expected: RoaringBitmap<u32> = (0..1000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..12000).collect();
    let bitmap2: RoaringBitmap<u32> = (6000..18000).collect();

    let expected: RoaringBitmap<u32> = (0..6000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_array() {
    let bitmap1: RoaringBitmap<u32> = (0..6000).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000).collect();

    let expected: RoaringBitmap<u32> = (0..3000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..12000).collect();
    let bitmap2: RoaringBitmap<u32> = (9000..12000).collect();

    let expected: RoaringBitmap<u32> = (0..9000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_array() {
    let bitmap1: RoaringBitmap<u32> = (0..6000).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..6000).collect();

    let expected: RoaringBitmap<u32> = (0..3000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap<u32> = (0..2000).chain(1000000..1002000).chain(2000000..2001000).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000).chain(1001000..1003000).chain(2000000..2001000).collect();

    let expected: RoaringBitmap<u32> = (0..1000).chain(1000000..1001000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap<u32> = (0..6000).chain(1000000..1012000).chain(2000000..2010000).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000).chain(1006000..1018000).chain(2000000..2010000).collect();

    let expected: RoaringBitmap<u32> = (0..3000).chain(1000000..1006000).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}
