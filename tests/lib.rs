extern crate roaring;

use std::{ u32 };

use roaring::RoaringBitmap;

#[test]
fn smoke() {
    let mut bitmap = RoaringBitmap::new();
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
    let bitmap: RoaringBitmap = FromIterator::from_iter(0..5000);
    assert_eq!(bitmap.len(), 5000);
    for i in 1..5000 {
        assert_eq!(bitmap.contains(i), true);
    }
    assert_eq!(bitmap.contains(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap: RoaringBitmap = FromIterator::from_iter(0..5000);
    for i in 3000..5000 {
        bitmap.remove(i);
    }
    assert_eq!(bitmap.len(), 3000);
    for i in 0..3000 {
        assert_eq!(bitmap.contains(i), true);
    }
    for i in 3000..5000 {
        assert_eq!(bitmap.contains(i), false);
    }
}

#[test]
fn subset_array() {
    let mut bitmap1 = RoaringBitmap::new();
    let mut bitmap2 = RoaringBitmap::new();
    for i in 1..3000 {
        bitmap1.insert(i);
    }
    for i in 1001..2000 {
        bitmap2.insert(i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
    bitmap2.insert(6000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn subset_bitmap() {
    let mut bitmap1 = RoaringBitmap::new();
    let mut bitmap2 = RoaringBitmap::new();
    for i in 1..10000 {
        bitmap1.insert(i);
    }
    for i in 2001..8000 {
        bitmap2.insert(i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
    bitmap2.insert(13000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn subset_array_bitmap() {
    let mut bitmap1 = RoaringBitmap::new();
    let mut bitmap2 = RoaringBitmap::new();
    for i in 1..10000 {
        bitmap1.insert(i);
    }
    for i in 2001..3000 {
        bitmap2.insert(i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
    bitmap2.insert(13000);
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
}

#[test]
fn subset_different_containers() {
    let mut bitmap1 = RoaringBitmap::new();
    let mut bitmap2 = RoaringBitmap::new();
    for i in 1..4000 {
        bitmap1.insert(i);
    }
    for i in 1..4000 {
        bitmap2.insert(u32::MAX - i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
    for i in 2001..3000 {
        bitmap2.insert(i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), false);
    for i in 1..4000 {
        bitmap2.remove(u32::MAX - i);
    }
    assert_eq!(bitmap2.is_subset(&bitmap1), true);
}

