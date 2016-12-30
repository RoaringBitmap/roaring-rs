extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn array() {
    let bitmap = RoaringBitmap::from_iter(0..2000);
    assert_eq!((2000, Some(2000)), bitmap.iter().size_hint());
    assert_eq!((1000, Some(1000)), bitmap.iter().skip(1000).size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(2000).size_hint());
}

#[test]
fn bitmap() {
    let bitmap = RoaringBitmap::from_iter(0..6000);
    assert_eq!((6000, Some(6000)), bitmap.iter().size_hint());
    assert_eq!((3000, Some(3000)), bitmap.iter().skip(3000).size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(6000).size_hint());
}

#[test]
fn arrays() {
    let bitmap = RoaringBitmap::from_iter((0..2000).chain(1000000..1002000).chain(2000000..2001000));
    assert_eq!((5000, Some(5000)), bitmap.iter().size_hint());
    assert_eq!((2000, Some(2000)), bitmap.iter().skip(3000).size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(5000).size_hint());
}

#[test]
fn bitmaps() {
    let bitmap = RoaringBitmap::from_iter((0..6000).chain(1000000..1012000).chain(2000000..2010000));
    assert_eq!((28000, Some(28000)), bitmap.iter().size_hint());
    assert_eq!((26000, Some(26000)), bitmap.iter().skip(2000).size_hint());
    assert_eq!((21000, Some(21000)), bitmap.iter().skip(7000).size_hint());
    assert_eq!((1000, Some(1000)), bitmap.iter().skip(27000).size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(28000).size_hint());
}
