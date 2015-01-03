extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..3000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..1000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn array_and_bitmap() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..8000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..1000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_bitmap() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(6000..18000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..6000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_to_array() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(3000..9000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..3000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..12000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(9000..12000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..9000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmap_and_array_to_array() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(3000..6000);

    let expected: RoaringBitmap = FromIterator::from_iter(0..3000);
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn arrays() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..2000).chain(1000000..1002000).chain(2000000..2001000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((1000..3000).chain(1001000..1003000).chain(2000000..2001000));

    let expected: RoaringBitmap = FromIterator::from_iter((0..1000).chain(1000000..1001000));
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}

#[test]
fn bitmaps() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter((0..6000).chain(1000000..1012000).chain(2000000..2010000));
    let bitmap2: RoaringBitmap = FromIterator::from_iter((3000..9000).chain(1006000..1018000).chain(2000000..2010000));

    let expected: RoaringBitmap = FromIterator::from_iter((0..3000).chain(1000000..1006000));
    let actual: RoaringBitmap = FromIterator::from_iter(bitmap1.difference(&bitmap2));

    assert_eq!(actual, expected);
}
