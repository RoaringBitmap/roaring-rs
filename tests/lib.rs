#![feature(slicing_syntax)]

extern crate roaring;

use std::{ u32 };

use roaring::RoaringBitmap;

#[test]
fn smoke() {
    let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
    assert_eq!(bitmap.len(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.remove(0);
    assert_eq!(bitmap.len(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.insert(1);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.len(), 1);
    assert_eq!(bitmap.is_empty(), false);
    bitmap.insert(u32::MAX - 2);
    assert_eq!(bitmap.contains(u32::MAX - 2), true);
    assert_eq!(bitmap.len(), 2);
    bitmap.insert(u32::MAX);
    assert_eq!(bitmap.contains(u32::MAX), true);
    assert_eq!(bitmap.len(), 3);
    bitmap.insert(2);
    assert_eq!(bitmap.contains(2), true);
    assert_eq!(bitmap.len(), 4);
    bitmap.remove(2);
    assert_eq!(bitmap.contains(2), false);
    assert_eq!(bitmap.len(), 3);
    assert_eq!(bitmap.contains(0), false);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.contains(100), false);
    assert_eq!(bitmap.contains(u32::MAX - 2), true);
    assert_eq!(bitmap.contains(u32::MAX - 1), false);
    assert_eq!(bitmap.contains(u32::MAX), true);
}

#[test]
fn to_bitmap() {
    let bitmap: RoaringBitmap<u32> = (0..5000u32).collect();
    assert_eq!(bitmap.len(), 5000);
    for i in 1..5000u32 {
        assert_eq!(bitmap.contains(i), true);
    }
    assert_eq!(bitmap.contains(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap: RoaringBitmap<u32> = (0..5000u32).collect();
    for i in 3000..5000u32 {
        bitmap.remove(i);
    }
    assert_eq!(bitmap.len(), 3000);
    for i in 0..3000u32 {
        assert_eq!(bitmap.contains(i), true);
    }
    for i in 3000..5000u32 {
        assert_eq!(bitmap.contains(i), false);
    }
}
