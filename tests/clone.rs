extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let original: RoaringBitmap<u32> = (0..2000u32).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original: RoaringBitmap<u32> = (0..6000u32).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let clone = original.clone();

    assert_eq!(clone, original);
}
