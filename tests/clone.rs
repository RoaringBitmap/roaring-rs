#![feature(slicing_syntax)]

extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let original: RoaringBitmap<u32> = (0..2000).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original: RoaringBitmap<u32> = (0..6000).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original: RoaringBitmap<u32> = (0..2000).chain(1000000..1002000).chain(2000000..2001000).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original: RoaringBitmap<u32> = (0..6000).chain(1000000..1012000).chain(2000000..2010000).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}
