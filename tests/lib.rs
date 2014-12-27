extern crate roaring;

use std::{ u32 };

use roaring::RoaringBitmap;

#[test]
fn smoke() {
    let mut bitmap = RoaringBitmap::new();
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.remove(0);
    assert_eq!(bitmap.cardinality(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.insert(1);
    assert_eq!(bitmap.cardinality(), 1);
    assert_eq!(bitmap.is_empty(), false);
    bitmap.insert(u32::MAX - 2);
    assert_eq!(bitmap.cardinality(), 2);
    bitmap.insert(u32::MAX);
    assert_eq!(bitmap.cardinality(), 3);
    bitmap.insert(2);
    assert_eq!(bitmap.cardinality(), 4);
    bitmap.remove(2);
    assert_eq!(bitmap.cardinality(), 3);
    assert_eq!(bitmap.contains(0), false);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.contains(100), false);
    assert_eq!(bitmap.contains(u32::MAX - 2), true);
    assert_eq!(bitmap.contains(u32::MAX - 1), false);
    assert_eq!(bitmap.contains(u32::MAX), true);
}

#[test]
fn to_bitmap() {
    let mut bitmap = RoaringBitmap::new();
    for i in 1..5000 {
        bitmap.insert(i);
    }
    assert_eq!(bitmap.cardinality(), 4999);
    assert_eq!(bitmap.contains(0), false);
    for i in 1..5000 {
        assert_eq!(bitmap.contains(i), true);
    }
    assert_eq!(bitmap.contains(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap = RoaringBitmap::new();
    for i in 1..5000 {
        bitmap.insert(i);
    }
    for i in 3000..5000 {
        bitmap.remove(i);
    }
    assert_eq!(bitmap.cardinality(), 2999);
    assert_eq!(bitmap.contains(0), false);
    for i in 1..2999 {
        assert_eq!(bitmap.contains(i), true);
    }
    for i in 3000..5001 {
        assert_eq!(bitmap.contains(i), false);
    }
}
