extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(4000..6000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn array_not() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..4000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(2000..6000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmap() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(10000..16000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmap_not() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..10000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(5000..15000);
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..2000).chain(1000000..1002000).chain(2000000..2002000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((100000..102000).chain(1100000..1102000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn arrays_not() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..2000).chain(0..1002000).chain(2000000..2002000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((100000..102000).chain(1001000..1003000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..6000).chain(1000000..1006000).chain(2000000..2006000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((100000..106000).chain(1100000..1106000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), true);
}

#[test]
fn bitmaps_not() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..6000).chain(0..1006000).chain(2000000..2006000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((100000..106000).chain(1004000..1008000));
    assert_eq!(bitmap1.is_disjoint(&bitmap2), false);
}
