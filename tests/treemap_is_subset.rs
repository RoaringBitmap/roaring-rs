extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array_not() {
    let sup = RoaringTreemap::from_iter(0..2000);
    let sub = RoaringTreemap::from_iter(1000..3000);
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn array() {
    let sup = RoaringTreemap::from_iter(0..4000);
    let sub = RoaringTreemap::from_iter(2000..3000);
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn array_bitmap_not() {
    let sup = RoaringTreemap::from_iter(0..2000);
    let sub = RoaringTreemap::from_iter(1000..15000);
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap_not() {
    let sup = RoaringTreemap::from_iter(0..6000);
    let sub = RoaringTreemap::from_iter(4000..10000);
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap() {
    let sup = RoaringTreemap::from_iter(0..20000);
    let sub = RoaringTreemap::from_iter(5000..15000);
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn bitmap_array_not() {
    let sup = RoaringTreemap::from_iter(0..20000);
    let sub = RoaringTreemap::from_iter(19000..21000);
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmap_array() {
    let sup = RoaringTreemap::from_iter(0..20000);
    let sub = RoaringTreemap::from_iter(18000..20000);
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn arrays_not() {
    let sup = RoaringTreemap::from_iter((0..2000).chain(1_000_000..1_002_000));
    let sub = RoaringTreemap::from_iter((100_000..102_000).chain(1_100_000..1_102_000));
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn arrays() {
    let sup = RoaringTreemap::from_iter((0..3000).chain(100_000..103_000));
    let sub = RoaringTreemap::from_iter((0..2000).chain(100_000..102_000));
    assert_eq!(sub.is_subset(&sup), true);
}

#[test]
fn bitmaps_not() {
    let sup = RoaringTreemap::from_iter(
        (0..6000)
            .chain(1_000_000..1_006_000)
            .chain(2_000_000..2_010_000),
    );
    let sub = RoaringTreemap::from_iter((100_000..106_000).chain(1_100_000..1_106_000));
    assert_eq!(sub.is_subset(&sup), false);
}

#[test]
fn bitmaps() {
    let sup = RoaringTreemap::from_iter((0..1_000_000).chain(2_000_000..2_010_000));
    let sub = RoaringTreemap::from_iter((0..10_000).chain(500_000..510_000));
    assert_eq!(sub.is_subset(&sup), true);
}
