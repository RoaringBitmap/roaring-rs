extern crate roaring;

use proptest::collection::btree_set;
use proptest::prelude::*;
use roaring::RoaringBitmap;

#[test]
fn range_array() {
    let mut rb = RoaringBitmap::new();
    rb.insert(0);
    rb.insert(1);
    rb.insert(10);
    rb.insert(100_000);
    rb.insert(999_999);
    rb.insert(1_000_000);

    let expected = vec![1, 10, 100_000, 999_999];
    let actual: Vec<u32> = rb.range(1..=999_999).collect();
    assert_eq!(expected, actual);
}

#[test]
fn range_bitmap() {
    let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();

    let expected = vec![10, 11, 12];
    let actual: Vec<u32> = rb.range(0..13).collect();
    assert_eq!(expected, actual);
}

#[test]
fn range_none() {
    let rb = RoaringBitmap::from_sorted_iter(10..5000).unwrap();

    let expected: Vec<u32> = vec![];
    let actual: Vec<u32> = rb.range(13..0).collect();
    assert_eq!(expected, actual);
}

proptest! {
    #[test]
    fn proptest_range(
        values in btree_set(..=262_143_u32, ..=1000),
        range_a in 0u32..262_143,
        range_b in 0u32..262_143,
    ){
        let range = if range_a <= range_b {
            range_a..=range_b
        } else {
            range_b..=range_a
        };

        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();
        let expected: Vec<u32> = values.iter().cloned()
                                       .filter(|&x| range.contains(&x))
                                       .collect();
        let actual: Vec<u32> = bitmap.range(range.clone()).collect();

        assert_eq!(expected, actual);
    }
}
