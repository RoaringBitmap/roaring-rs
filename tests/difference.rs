extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn mini() {
    let bitmap1 = RoaringBitmap::from_iter(0..20u32);
    let bitmap2 = RoaringBitmap::from_iter(10..30u32);

    let expected = RoaringBitmap::from_iter(0..10u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn array() {
    let bitmap1 = RoaringBitmap::from_iter(0..2000u32);
    let bitmap2 = RoaringBitmap::from_iter(1000..3000u32);

    let expected = RoaringBitmap::from_iter(0..1000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn array_and_bitmap() {
    let bitmap1 = RoaringBitmap::from_iter(0..2000u32);
    let bitmap2 = RoaringBitmap::from_iter(1000..8000u32);

    let expected = RoaringBitmap::from_iter(0..1000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_bitmap() {
    let bitmap1 = RoaringBitmap::from_iter(0..12000u32);
    let bitmap2 = RoaringBitmap::from_iter(6000..18000u32);

    let expected = RoaringBitmap::from_iter(0..6000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_array() {
    let bitmap1 = RoaringBitmap::from_iter(0..6000u32);
    let bitmap2 = RoaringBitmap::from_iter(3000..9000u32);

    let expected = RoaringBitmap::from_iter(0..3000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let bitmap1 = RoaringBitmap::from_iter(0..12000u32);
    let bitmap2 = RoaringBitmap::from_iter(9000..12000u32);

    let expected = RoaringBitmap::from_iter(0..9000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_array() {
    let bitmap1 = RoaringBitmap::from_iter(0..6000u32);
    let bitmap2 = RoaringBitmap::from_iter(3000..6000u32);

    let expected = RoaringBitmap::from_iter(0..3000u32);
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn mini_arrays() {
    let bitmap1 = RoaringBitmap::from_iter((0..20u32).chain(1000000..1000020).chain(2000000..2000010));
    let bitmap2 = RoaringBitmap::from_iter((10..30u32).chain(1000010..1000030).chain(2000000..2000010));

    let expected = RoaringBitmap::from_iter((0..10u32).chain(1000000..1000010));
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn arrays() {
    let bitmap1 = RoaringBitmap::from_iter((0..2000u32).chain(1000000..1002000).chain(2000000..2001000));
    let bitmap2 = RoaringBitmap::from_iter((1000..3000u32).chain(1001000..1003000).chain(2000000..2001000));

    let expected = RoaringBitmap::from_iter((0..1000u32).chain(1000000..1001000));
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmaps() {
    let bitmap1 = RoaringBitmap::from_iter((0..6000u32).chain(1000000..1012000).chain(2000000..2010000));
    let bitmap2 = RoaringBitmap::from_iter((3000..9000u32).chain(1006000..1018000).chain(2000000..2010000));

    let expected = RoaringBitmap::from_iter((0..3000u32).chain(1000000..1006000));
    let actual = RoaringBitmap::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}
