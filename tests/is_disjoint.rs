#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (4000..6000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn array_not() {
    let bitmap1: RoaringBitmap<u32> = (0..4000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (2000..6000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmap() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (10000..16000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmap_not() {
    let bitmap1: RoaringBitmap<u32> = (0..10000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (5000..15000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2002000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (100000..102000u32).chain(1100000..1102000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn arrays_not() {
    let bitmap1: RoaringBitmap<u32> = (0..2000u32).chain(0..1002000u32).chain(2000000..2002000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (100000..102000u32).chain(1001000..1003000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1006000u32).chain(2000000..2006000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (100000..106000u32).chain(1100000..1106000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmaps_not() {
    let bitmap1: RoaringBitmap<u32> = (0..6000u32).chain(0..1006000u32).chain(2000000..2006000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (100000..106000u32).chain(1004000..1008000u32).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}
