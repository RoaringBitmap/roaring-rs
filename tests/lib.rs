extern crate roaring;

use std::{ u32 };

use roaring::RoaringBitmap;

#[test]
fn smoke() {
    let mut bitmap = RoaringBitmap::new();
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.none(), true);
    assert_eq!(bitmap.any(), false);
    bitmap.set(0, false);
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.none(), true);
    assert_eq!(bitmap.any(), false);
    bitmap.set(1, true);
    assert_eq!(bitmap.cardinality(), 1);
    assert_eq!(bitmap.none(), false);
    assert_eq!(bitmap.any(), true);
    bitmap.set(u32::MAX - 2, true);
    assert_eq!(bitmap.cardinality(), 2);
    bitmap.set(u32::MAX, true);
    assert_eq!(bitmap.cardinality(), 3);
    bitmap.set(2, true);
    assert_eq!(bitmap.cardinality(), 4);
    bitmap.set(2, false);
    assert_eq!(bitmap.cardinality(), 3);
    assert_eq!(bitmap.get(0), false);
    assert_eq!(bitmap.get(1), true);
    assert_eq!(bitmap.get(100), false);
    assert_eq!(bitmap.get(u32::MAX - 2), true);
    assert_eq!(bitmap.get(u32::MAX - 1), false);
    assert_eq!(bitmap.get(u32::MAX), true);
}

#[test]
fn to_bitmap() {
    let mut bitmap = RoaringBitmap::new();
    for i in 1..5000 {
        bitmap.set(i, true);
    }
    assert_eq!(bitmap.cardinality(), 4999);
    assert_eq!(bitmap.get(0), false);
    for i in 1..5000 {
        assert_eq!(bitmap.get(i), true);
    }
    assert_eq!(bitmap.get(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap = RoaringBitmap::new();
    for i in 1..5000 {
        bitmap.set(i, true);
    }
    for i in 3000..5000 {
        bitmap.set(i, false);
    }
    assert_eq!(bitmap.cardinality(), 2999);
    assert_eq!(bitmap.get(0), false);
    for i in 1..2999 {
        assert_eq!(bitmap.get(i), true);
    }
    for i in 3000..5001 {
        assert_eq!(bitmap.get(i), false);
    }
}
