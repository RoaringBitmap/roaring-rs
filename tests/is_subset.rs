extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array_not_subset() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(1000..3000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn array_subset() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..4000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(2000..3000);
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
}

#[test]
fn bitmap_not_subset() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(4000..10000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn bitmap_subset() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..20000);
    let bitmap2: RoaringBitmap = FromIterator::from_iter(5000..15000);
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
}

#[test]
fn arrays_not_subset() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..2000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(100_000..102_000);
    bitmap1.extend(1_000_000..1_002_000);
    bitmap2.extend(1_100_000..1_102_000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn arrays_subset() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..3000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(0..2000);
    bitmap1.extend(100000..103000);
    bitmap2.extend(100000..102000);
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
}

#[test]
fn bitmaps_not_subset() {
    let mut bitmap1: RoaringBitmap = FromIterator::from_iter(0..6000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(100000..106000);
    bitmap1.extend(0..1006000);
    bitmap2.extend(1100000..1106000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn bitmaps_subset() {
    let bitmap1: RoaringBitmap = FromIterator::from_iter(0..1_000_000);
    let mut bitmap2: RoaringBitmap = FromIterator::from_iter(0..10_000);
    bitmap2.extend(500_000..510_000);
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
}
