extern crate roaring;
use roaring::RoaringTreemap;

#[test]
fn smoke() {
    let mut bitmap = RoaringTreemap::new();
    assert_eq!(bitmap.len(), 0);
    assert!(bitmap.is_empty());
    bitmap.remove(0);
    assert_eq!(bitmap.len(), 0);
    assert!(bitmap.is_empty());
    bitmap.insert(1);
    assert!(bitmap.contains(1));
    assert_eq!(bitmap.len(), 1);
    assert!(!bitmap.is_empty());
    bitmap.insert(u64::max_value() - 2);
    assert!(bitmap.contains(u64::max_value() - 2));
    assert_eq!(bitmap.len(), 2);
    bitmap.insert(u64::max_value());
    assert!(bitmap.contains(u64::max_value()));
    assert_eq!(bitmap.len(), 3);
    bitmap.insert(2);
    assert!(bitmap.contains(2));
    assert_eq!(bitmap.len(), 4);
    bitmap.remove(2);
    assert!(!bitmap.contains(2));
    assert_eq!(bitmap.len(), 3);
    assert!(!bitmap.contains(0));
    assert!(bitmap.contains(1));
    assert!(!bitmap.contains(100));
    assert!(bitmap.contains(u64::max_value() - 2));
    assert!(!bitmap.contains(u64::max_value() - 1));
    assert!(bitmap.contains(u64::max_value()));
}

#[test]
fn remove_range() {
    let ranges = [0u64, 1, 63, 64, 65, 100, 4096 - 1, 4096, 4096 + 1, 65536 - 1];
    for (i, &a) in ranges.iter().enumerate() {
        for &b in &ranges[i..] {
            let mut bitmap = (0..=65536).collect::<RoaringTreemap>();
            assert_eq!(bitmap.remove_range(a..b), (b - a));
            assert_eq!(bitmap, ((0..a).chain(b..=65536)).collect::<RoaringTreemap>());
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
    let bitmap = (0..5000).collect::<RoaringTreemap>();
    assert_eq!(bitmap.len(), 5000);
    for i in 1..5000 {
        assert!(bitmap.contains(i));
    }
    assert!(!bitmap.contains(5001));
}

#[test]
fn to_array() {
    let mut bitmap = (0..5000).collect::<RoaringTreemap>();
    for i in 3000..5000 {
        bitmap.remove(i);
    }
    assert_eq!(bitmap.len(), 3000);
    for i in 0..3000 {
        assert!(bitmap.contains(i));
    }
    for i in 3000..5000 {
        assert!(!bitmap.contains(i));
    }
}
