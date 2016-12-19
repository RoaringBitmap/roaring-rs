extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..1000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn no_difference() {
    let mut bitmap1: RoaringBitmap<u32> = (1..3u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1..3u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, RoaringBitmap::new());
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..8000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..1000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..12000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (6000..18000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..6000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1: RoaringBitmap<u32> = (0..6000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..3000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let mut bitmap1: RoaringBitmap<u32> = (0..12000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (9000..12000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..9000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_array() {
    let mut bitmap1: RoaringBitmap<u32> = (0..6000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..6000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..3000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (1000..3000u32).chain(1001000..1003000u32).chain(2000000..2001000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..1000u32).chain(1000000..1001000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays_removing_one_whole_container() {
    let mut bitmap1: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (0..3000u32).chain(1001000..1003000u32).chain(2000000..2001000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (1000000..1001000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let bitmap2: RoaringBitmap<u32> = (3000..9000u32).chain(1006000..1018000u32).chain(2000000..2010000u32).collect();
    let bitmap3: RoaringBitmap<u32> = (0..3000u32).chain(1000000..1006000u32).collect();

    bitmap1.difference_with(&bitmap2);

    assert_eq!(bitmap1, bitmap3);
}
