#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000u32).collect();

    let expected: RoaringBitmap<u32> = (0..1000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn array_and_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..8000u32).collect();

    let expected: RoaringBitmap<u32> = (0..1000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..12000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (6000..18000u32).collect();

    let expected: RoaringBitmap<u32> = (0..6000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_array() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000u32).collect();

    let expected: RoaringBitmap<u32> = (0..3000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..12000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (9000..12000u32).collect();

    let expected: RoaringBitmap<u32> = (0..9000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_array() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..6000u32).collect();

    let expected: RoaringBitmap<u32> = (0..3000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000u32).chain(1001000..1003000u32).chain(2000000..2001000u32).collect();

    let expected: RoaringBitmap<u32> = (0..1000u32).chain(1000000..1001000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000u32).chain(1006000..1018000u32).chain(2000000..2010000u32).collect();

    let expected: RoaringBitmap<u32> = (0..3000u32).chain(1000000..1006000u32).collect();
    let actual: RoaringBitmap<u32> = bitmap1.difference(&bitmap2).collect();

    assert_eq!(actual, expected);
}
