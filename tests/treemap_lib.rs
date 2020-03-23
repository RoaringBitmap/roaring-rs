extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn smoke() {
    let mut bitmap = RoaringTreemap::new();
    assert_eq!(bitmap.len(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.remove(0);
    assert_eq!(bitmap.len(), 0);
    assert_eq!(bitmap.is_empty(), true);
    bitmap.insert(1);
    assert_eq!(bitmap.contains(1), true);
    assert_eq!(bitmap.len(), 1);
    assert_eq!(bitmap.is_empty(), false);
    bitmap.insert(u64::max_value() - 2);
    assert_eq!(bitmap.contains(u64::max_value() - 2), true);
    assert_eq!(bitmap.len(), 2);
    bitmap.insert(u64::max_value());
    assert_eq!(bitmap.contains(u64::max_value()), true);
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
    assert_eq!(bitmap.contains(u64::max_value() - 2), true);
    assert_eq!(bitmap.contains(u64::max_value() - 1), false);
    assert_eq!(bitmap.contains(u64::max_value()), true);
}

#[test]
fn remove_range() {
    let ranges = [
        0u64,
        1,
        63,
        64,
        65,
        100,
        4096 - 1,
        4096,
        4096 + 1,
        65536 - 1,
    ];
    for (i, &a) in ranges.iter().enumerate() {
        for &b in &ranges[i..] {
            let mut bitmap = RoaringTreemap::from_iter(0..=65536);
            assert_eq!(bitmap.remove_range(a..b), (b - a));
            assert_eq!(bitmap, RoaringTreemap::from_iter((0..a).chain(b..=65536)));
        }
    }
}

#[test]
fn test_max() {
    let mut bitmap = RoaringTreemap::new();
    assert_eq!(bitmap.max(), None);
    bitmap.insert(0);
    assert_eq!(bitmap.max(), Some(0));
    bitmap.insert(1);
    assert_eq!(bitmap.max(), Some(1));
    bitmap.insert(u64::max_value());
    assert_eq!(bitmap.max(), Some(u64::max_value()));
}

#[test]
fn test_min() {
    let mut bitmap = RoaringTreemap::new();
    assert_eq!(bitmap.min(), None);
    bitmap.insert(u64::max_value());
    assert_eq!(bitmap.min(), Some(u64::max_value()));
    bitmap.insert(1);
    assert_eq!(bitmap.min(), Some(1));
    bitmap.insert(0);
    assert_eq!(bitmap.min(), Some(0));
}

#[test]
fn to_bitmap() {
    let bitmap = RoaringTreemap::from_iter(0..5000);
    assert_eq!(bitmap.len(), 5000);
    for i in 1..5000 {
        assert_eq!(bitmap.contains(i), true);
    }
    assert_eq!(bitmap.contains(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap = RoaringTreemap::from_iter(0..5000);
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
