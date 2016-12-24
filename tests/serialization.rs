extern crate roaring;

use std::iter::FromIterator;

use roaring::RoaringBitmap;

// Test data from https://github.com/RoaringBitmap/RoaringFormatSpec/tree/master/testdata
static BITMAP_WITHOUT_RUNS: &'static [u8] = include_bytes!("bitmapwithoutruns.bin");

fn test_data_bitmap() -> RoaringBitmap<u32> {
    RoaringBitmap::from_iter(
        (0..100).map(|i| i * 1000)
            .chain((100000..200000).map(|i| i * 3))
            .chain(700000..800000))
}

fn serialize_and_deserialize(bitmap: &RoaringBitmap<u32>) -> RoaringBitmap<u32> {
    let mut buffer = vec![];
    bitmap.serialize_into(&mut buffer).unwrap();
    RoaringBitmap::deserialize_from(&mut &buffer[..]).unwrap()
}

#[test]
fn test_deserialize_from_provided_data() {
    assert_eq!(
        RoaringBitmap::deserialize_from(&mut &*BITMAP_WITHOUT_RUNS).unwrap(),
        test_data_bitmap());
}

#[test]
fn test_serialize_into_provided_data() {
    let bitmap = test_data_bitmap();
    let mut buffer = vec![];
    bitmap.serialize_into(&mut buffer).unwrap();
    assert!(BITMAP_WITHOUT_RUNS == &buffer[..]);
}

#[test]
fn test_empty() {
    let original = RoaringBitmap::new();
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_one() {
    let original = RoaringBitmap::from_iter(1..2);
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_array() {
    let original = RoaringBitmap::from_iter(1000..3000);
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_bitmap() {
    let original = RoaringBitmap::from_iter(1000..6000);
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_arrays() {
    let original = RoaringBitmap::from_iter((1000..3000).chain(70000..74000));
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_bitmaps() {
    let original = RoaringBitmap::from_iter((1000..6000).chain(70000..77000));
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}

#[test]
fn test_mixed() {
    let original = RoaringBitmap::from_iter((1000..3000).chain(70000..77000));
    let new = serialize_and_deserialize(&original);
    assert_eq!(original, new);
}
