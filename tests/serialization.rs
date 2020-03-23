extern crate roaring;

use std::iter::FromIterator;

use roaring::RoaringBitmap;

// Test data from https://github.com/RoaringBitmap/RoaringFormatSpec/tree/master/testdata
static BITMAP_WITHOUT_RUNS: &[u8] = include_bytes!("bitmapwithoutruns.bin");
static BITMAP_WITH_RUNS: &[u8] = include_bytes!("bitmapwithruns.bin");

fn test_data_bitmap() -> RoaringBitmap {
    RoaringBitmap::from_iter(
        (0..100)
            .map(|i| i * 1000)
            .chain((100_000..200_000).map(|i| i * 3))
            .chain(700_000..800_000),
    )
}

fn serialize_and_deserialize(bitmap: &RoaringBitmap) -> RoaringBitmap {
    let mut buffer = vec![];
    bitmap.serialize_into(&mut buffer).unwrap();
    assert_eq!(buffer.len(), bitmap.serialized_size());
    RoaringBitmap::deserialize_from(&mut &buffer[..]).unwrap()
}

#[test]
fn test_deserialize_without_runs_from_provided_data() {
    assert_eq!(
        RoaringBitmap::deserialize_from(&mut &BITMAP_WITHOUT_RUNS[..]).unwrap(),
        test_data_bitmap()
    );
}

#[test]
fn test_deserialize_with_runs_from_provided_data() {
    let mut expected = test_data_bitmap();
    // Call optimize to create run containers
    expected.optimize();
    assert_eq!(
        RoaringBitmap::deserialize_from(&mut &BITMAP_WITH_RUNS[..]).unwrap(),
        expected
    );
}

#[test]
fn test_serialize_into_provided_data() {
    let bitmap = test_data_bitmap();
    let mut buffer = vec![];
    bitmap.serialize_into(&mut buffer).unwrap();
    assert!(BITMAP_WITHOUT_RUNS == &buffer[..]);
}

#[test]
fn test_serialize_with_runs_into_provided_data() {
    let mut bitmap = test_data_bitmap();
    // Call optimize to create run containers
    bitmap.optimize();
    let mut buffer = vec![];
    bitmap.serialize_into(&mut buffer).unwrap();
    assert!(BITMAP_WITH_RUNS == &buffer[..]);
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

#[test]
fn test_runs() {
    let mut original = RoaringBitmap::from_iter((1000..3000).chain(70000..77000));
    original.optimize();
    let new = serialize_and_deserialize(&original);
    assert_eq!(original.len(), new.len());
    assert_eq!(original.min(), new.min());
    assert_eq!(original.max(), new.max());
}
