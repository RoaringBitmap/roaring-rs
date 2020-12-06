extern crate roaring;
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
    bitmap.insert(u32::max_value() - 2);
    assert_eq!(bitmap.contains(u32::max_value() - 2), true);
    assert_eq!(bitmap.len(), 2);
    bitmap.insert(u32::max_value());
    assert_eq!(bitmap.contains(u32::max_value()), true);
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
    assert_eq!(bitmap.contains(u32::max_value() - 2), true);
    assert_eq!(bitmap.contains(u32::max_value() - 1), false);
    assert_eq!(bitmap.contains(u32::max_value()), true);
}

#[test]
fn remove_range() {
    let ranges = [
        0u32,
        1,
        63,
        64,
        65,
        100,
        4096 - 1,
        4096,
        4096 + 1,
        65536 - 1,
        65536,
        65536 + 1,
    ];
    for (i, &a) in ranges.iter().enumerate() {
        for &b in &ranges[i..] {
            let mut bitmap = (0..=65536).collect::<RoaringBitmap>();
            assert_eq!(
                bitmap.remove_range(u64::from(a)..u64::from(b)),
                u64::from(b - a)
            );
            assert_eq!(bitmap, (0..a).chain(b..=65536).collect::<RoaringBitmap>());
        }
    }
}

#[test]
#[allow(clippy::range_plus_one)] // remove_range needs an exclusive range
fn remove_range_array() {
    let mut bitmap = (0..1000).collect::<RoaringBitmap>();
    for i in 0..1000 {
        assert_eq!(bitmap.remove_range(i..i), 0);
        assert_eq!(bitmap.remove_range(i..i + 1), 1);
    }

    // insert 0, 2, 4, ..
    // remove [0, 2), [2, 4), ..
    let mut bitmap = (0..1000).map(|x| x * 2).collect::<RoaringBitmap>();
    for i in 0..1000 {
        assert_eq!(bitmap.remove_range(i * 2..(i + 1) * 2), 1);
    }

    // remove [0, 2), [2, 4), ..
    let mut bitmap = (0..1000).collect::<RoaringBitmap>();
    for i in 0..1000 / 2 {
        assert_eq!(bitmap.remove_range(i * 2..(i + 1) * 2), 2);
    }
}

#[test]
#[allow(clippy::range_plus_one)] // remove_range needs an exclusive range
fn remove_range_bitmap() {
    let mut bitmap = (0..4096 + 1000).collect::<RoaringBitmap>();
    for i in 0..1000 {
        assert_eq!(bitmap.remove_range(i..i), 0);
        assert_eq!(bitmap.remove_range(i..i + 1), 1);
    }

    // insert 0, 2, 4, ..
    // remove [0, 2), [2, 4), ..
    let mut bitmap = ((0..4096 + 1000).map(|x| x * 2)).collect::<RoaringBitmap>();
    for i in 0..1000 {
        assert_eq!(bitmap.remove_range(i * 2..(i + 1) * 2), 1);
    }

    // remove [0, 2), [2, 4), ..
    let mut bitmap = (0..4096 + 1000).collect::<RoaringBitmap>();
    for i in 0..1000 / 2 {
        assert_eq!(bitmap.remove_range(i * 2..(i + 1) * 2), 2);
    }

    // remove [1, 3), [3, 5), ..
    let mut bitmap = (0..4096 + 1000).collect::<RoaringBitmap>();
    for i in 0..1000 / 2 {
        assert_eq!(bitmap.remove_range(i * 2 + 1..(i + 1) * 2 + 1), 2);
    }
}

#[test]
fn to_bitmap() {
    let bitmap = (0..5000).collect::<RoaringBitmap>();
    assert_eq!(bitmap.len(), 5000);
    for i in 1..5000 {
        assert_eq!(bitmap.contains(i), true);
    }
    assert_eq!(bitmap.contains(5001), false);
}

#[test]
fn to_array() {
    let mut bitmap = (0..5000).collect::<RoaringBitmap>();
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
