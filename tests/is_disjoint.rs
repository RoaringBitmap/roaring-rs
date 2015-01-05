#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap = (0..2000).collect();
    let bitmap2: RoaringBitmap = (4000..6000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn array_not() {
    let bitmap1: RoaringBitmap = (0..4000).collect();
    let bitmap2: RoaringBitmap = (2000..6000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmap() {
    let bitmap1: RoaringBitmap = (0..6000).collect();
    let bitmap2: RoaringBitmap = (10000..16000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmap_not() {
    let bitmap1: RoaringBitmap = (0..10000).collect();
    let bitmap2: RoaringBitmap = (5000..15000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap = (0..2000).chain(1000000..1002000).chain(2000000..2002000).collect();
    let bitmap2: RoaringBitmap = (100000..102000).chain(1100000..1102000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn arrays_not() {
    let bitmap1: RoaringBitmap = (0..2000).chain(0..1002000).chain(2000000..2002000).collect();
    let bitmap2: RoaringBitmap = (100000..102000).chain(1001000..1003000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap = (0..6000).chain(1000000..1006000).chain(2000000..2006000).collect();
    let bitmap2: RoaringBitmap = (100000..106000).chain(1100000..1106000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmaps_not() {
    let bitmap1: RoaringBitmap = (0..6000).chain(0..1006000).chain(2000000..2006000).collect();
    let bitmap2: RoaringBitmap = (100000..106000).chain(1004000..1008000).collect();
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}
