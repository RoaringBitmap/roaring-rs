extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array_to_array() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..3000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..3000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_to_bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..4000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(4000..8000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..8000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..8000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..8000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(6000..18000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..18000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(10000..13000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..13000);

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter((0..2000).chain(1000000..1002000).chain(3000000..3001000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((1000..3000).chain(1001000..1003000).chain(2000000..2001000));
    let bitmap3: RoaringBitmap = FromIterator::from_iter((0..3000).chain(1000000..1003000).chain(2000000..2001000).chain(3000000..3001000));

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter((0..6000).chain(1000000..1012000).chain(3000000..3010000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((3000..9000).chain(1006000..1018000).chain(2000000..2010000));
    let bitmap3: RoaringBitmap = FromIterator::from_iter((0..9000).chain(1000000..1018000).chain(2000000..2010000).chain(3000000..3010000));

    bitmap1.union_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
