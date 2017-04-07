extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = RoaringTreemap::from_iter(0..2000);
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmap() {
    let original = RoaringTreemap::from_iter(0..6000);
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn arrays() {
    let original = RoaringTreemap::from_iter((0..2000).chain(1000000..1002000).chain(2000000..2001000));
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps() {
    let original = RoaringTreemap::from_iter((0..6000).chain(1000000..1012000).chain(2000000..2010000));
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps_iterator() {
    let original = RoaringTreemap::from_iter((0..6000).chain(1000000..1012000).chain(2000000..2010000));
    let clone = RoaringTreemap::from_bitmaps(original.bitmaps().map(|(p, b)| (p, b.clone())));
    let clone2 = RoaringTreemap::from_iter(original.bitmaps().map(|(p, b)| (p, b.clone())));

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}
