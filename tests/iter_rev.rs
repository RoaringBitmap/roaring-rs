extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn mini() {
    let original: RoaringBitmap<u32> = (0..5u32).collect();
    let clone: RoaringBitmap<u32> = original.iter().rev().collect();

    assert_eq!(clone, original);
}

#[test]
fn array() {
    let original: RoaringBitmap<u32> = (0..2000u32).collect();
    let clone: RoaringBitmap<u32> = original.iter().rev().collect();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original: RoaringBitmap<u32> = (0..6000u32).collect();
    let clone: RoaringBitmap<u32> = original.iter().rev().collect();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let clone: RoaringBitmap<u32> = original.iter().rev().collect();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let clone: RoaringBitmap<u32> = original.iter().rev().collect();

    assert_eq!(clone, original);
}

#[test]
fn array_vs_vec() {
    let original: Vec<u32> = (0..2000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = bitmap.iter().rev().collect();
    let reversed: Vec<u32> = original.iter().rev().map(|&i| i).collect();

    assert_eq!(clone, reversed);
}

#[test]
fn bitmap_vs_vec() {
    let original: Vec<u32> = (0..6000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = bitmap.iter().rev().collect();
    let reversed: Vec<u32> = original.iter().rev().map(|&i| i).collect();

    assert_eq!(clone, reversed);
}

#[test]
fn arrays_vs_vec() {
    let original: Vec<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = bitmap.iter().rev().collect();
    let reversed: Vec<u32> = original.iter().rev().map(|&i| i).collect();

    assert_eq!(clone, reversed);
}

#[test]
fn bitmaps_vs_vec() {
    let original: Vec<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = bitmap.iter().rev().collect();
    let reversed: Vec<u32> = original.iter().rev().map(|&i| i).collect();

    assert_eq!(clone, reversed);
}
