#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array_not() {
    let sup: RoaringBitmap<u32> = (0..2000).collect();
    let sub: RoaringBitmap<u32> = (1000..3000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn array() {
    let sup: RoaringBitmap<u32> = (0..4000).collect();
    let sub: RoaringBitmap<u32> = (2000..3000).collect();
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn array_bitmap_not() {
    let sup: RoaringBitmap<u32> = (0..2000).collect();
    let sub: RoaringBitmap<u32> = (1000..15000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap_not() {
    let sup: RoaringBitmap<u32> = (0..6000).collect();
    let sub: RoaringBitmap<u32> = (4000..10000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap() {
    let sup: RoaringBitmap<u32> = (0..20000).collect();
    let sub: RoaringBitmap<u32> = (5000..15000).collect();
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn bitmap_array_not() {
    let sup: RoaringBitmap<u32> = (0..20000).collect();
    let sub: RoaringBitmap<u32> = (19000..21000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap_array() {
    let sup: RoaringBitmap<u32> = (0..20000).collect();
    let sub: RoaringBitmap<u32> = (18000..20000).collect();
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn arrays_not() {
    let sup: RoaringBitmap<u32> = (0..2000).chain(1_000_000..1_002_000).collect();
    let sub: RoaringBitmap<u32> = (100_000..102_000).chain(1_100_000..1_102_000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn arrays() {
    let sup: RoaringBitmap<u32> = (0..3000).chain(100000..103000).collect();
    let sub: RoaringBitmap<u32> = (0..2000).chain(100000..102000).collect();
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn bitmaps_not() {
    let sup: RoaringBitmap<u32> = (0..6000).chain(1000000..1006000).chain(2000000..2010000).collect();
    let sub: RoaringBitmap<u32> = (100000..106000).chain(1100000..1106000).collect();
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmaps() {
    let sup: RoaringBitmap<u32> = (0..1_000_000).chain(2000000..2010000).collect();
    let sub: RoaringBitmap<u32> = (0..10_000).chain(500_000..510_000).collect();
    assert_eq!(sub.is_subset(&sup), true);
}
