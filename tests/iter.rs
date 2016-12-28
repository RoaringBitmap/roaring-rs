extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = RoaringBitmap::from_iter(0..2000);
    let clone = RoaringBitmap::from_iter(&original);

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original = RoaringBitmap::from_iter(0..6000);
    let clone = RoaringBitmap::from_iter(&original);

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original = RoaringBitmap::from_iter((0..2000).chain(1000000..1002000).chain(2000000..2001000));
    let clone = RoaringBitmap::from_iter(&original);

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original = RoaringBitmap::from_iter((0..6000).chain(1000000..1012000).chain(2000000..2010000));
    let clone = RoaringBitmap::from_iter(&original);

    assert_eq!(clone, original);
}
