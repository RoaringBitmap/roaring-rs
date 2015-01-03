extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..3000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..1000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..8000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..1000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(6000..18000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..6000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(3000..9000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..3000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(9000..12000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..9000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_array() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(3000..6000);
    let bitmap3: RoaringBitmap = FromIterator::from_iter(0..3000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(1000..3000);
    let mut bitmap3: RoaringBitmap = FromIterator::from_iter(0..1000);

    bitmap1.extend(1000000..1002000);
    bitmap2.extend(1001000..1003000);
    bitmap3.extend(1000000..1001000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(3000..9000);
    let mut bitmap3: RoaringBitmap = FromIterator::from_iter(0..3000);

    bitmap1.extend(1000000..1012000);
    bitmap2.extend(1006000..1018000);
    bitmap3.extend(1000000..1006000);

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
