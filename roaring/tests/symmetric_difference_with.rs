extern crate roaring;
use std::collections::BTreeSet;

use proptest::collection::vec;
use proptest::prelude::*;
use roaring::{MultiOps, RoaringBitmap};

#[test]
fn array() {
    let mut bitmap1 = (0..2000).collect::<RoaringBitmap>();
    let bitmap2 = (1000..3000).collect::<RoaringBitmap>();
    let bitmap3 = (0..1000).chain(2000..3000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn no_symmetric_difference() {
    let mut bitmap1 = (0..2).collect::<RoaringBitmap>();
    let bitmap2 = (0..2).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, RoaringBitmap::new());
}

#[test]
fn array_and_bitmap() {
    let mut bitmap1 = (0..2000).collect::<RoaringBitmap>();
    let bitmap2 = (1000..8000).collect::<RoaringBitmap>();
    let bitmap3 = (0..1000).chain(2000..8000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_bitmap() {
    let mut bitmap1 = (0..12000).collect::<RoaringBitmap>();
    let bitmap2 = (6000..18000).collect::<RoaringBitmap>();
    let bitmap3 = (0..6000).chain(12000..18000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_to_array() {
    let mut bitmap1 = (0..6000).collect::<RoaringBitmap>();
    let bitmap2 = (2000..7000).collect::<RoaringBitmap>();
    let bitmap3 = (0..2000).chain(6000..7000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_bitmap() {
    let mut bitmap1 = (0..12000).collect::<RoaringBitmap>();
    let bitmap2 = (11000..14000).collect::<RoaringBitmap>();
    let bitmap3 = (0..11000).chain(12000..14000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmap_and_array_to_array() {
    let mut bitmap1 = (0..6000).collect::<RoaringBitmap>();
    let bitmap2 = (3000..7000).collect::<RoaringBitmap>();
    let bitmap3 = (0..3000).chain(6000..7000).collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn arrays() {
    let mut bitmap1 = (0..2000)
        .chain(1_000_000..1_002_000)
        .chain(3_000_000..3_001_000)
        .collect::<RoaringBitmap>();
    let bitmap2 = (1000..3000)
        .chain(1_001_000..1_003_000)
        .chain(2_000_000..2_000_001)
        .collect::<RoaringBitmap>();
    let bitmap3 = (0..1000)
        .chain(1_000_000..1_001_000)
        .chain(2000..3000)
        .chain(1_002_000..1_003_000)
        .chain(2_000_000..2_000_001)
        .chain(3_000_000..3_001_000)
        .collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn bitmaps() {
    let mut bitmap1 = (0..6000)
        .chain(1_000_000..1_012_000)
        .chain(3_000_000..3_010_000)
        .collect::<RoaringBitmap>();
    let bitmap2 = (3000..7000)
        .chain(1_006_000..1_018_000)
        .chain(2_000_000..2_010_000)
        .collect::<RoaringBitmap>();
    let bitmap3 = (0..3000)
        .chain(1_000_000..1_006_000)
        .chain(6000..7000)
        .chain(1_012_000..1_018_000)
        .chain(2_000_000..2_010_000)
        .chain(3_000_000..3_010_000)
        .collect::<RoaringBitmap>();

    bitmap1 ^= bitmap2;

    assert_eq!(bitmap1, bitmap3);
}

#[test]
fn interleaved_containers() {
    // One side has containers at even keys and the other one at odd keys.
    let even = (0..4096).map(|key| key << 17).collect::<RoaringBitmap>();
    let odd = (0..4096).map(|key| (key << 17) | (1 << 16)).collect::<RoaringBitmap>();
    let expected = (0..8192).map(|key| key << 16).collect::<RoaringBitmap>();

    let mut bitmap = even.clone();
    bitmap ^= &odd;
    assert_eq!(bitmap, expected);

    let mut bitmap = odd.clone();
    bitmap ^= even.clone();
    assert_eq!(bitmap, expected);

    assert_eq!([&even, &odd].symmetric_difference(), expected);
    assert_eq!([odd, even].symmetric_difference(), expected);
}

/// A bitmap made of ranges of up to 64 values spread over up to 512 containers,
/// along with the set of its values.
fn bitmap_and_set() -> impl Strategy<Value = (RoaringBitmap, BTreeSet<u32>)> {
    vec((0u32..512, any::<u16>(), 0u32..64), 0..256).prop_map(|ranges| {
        let mut bitmap = RoaringBitmap::new();
        let mut set = BTreeSet::new();
        for (key, low, len) in ranges {
            let start = (key << 16) | u32::from(low);
            let end = (start + len).min(start | 0xFFFF);
            bitmap.insert_range(start..=end);
            set.extend(start..=end);
        }
        (bitmap, set)
    })
}

proptest! {
    #[test]
    fn symmetric_difference_matches_btreeset(
        (a, set_a) in bitmap_and_set(),
        (b, set_b) in bitmap_and_set(),
        (c, set_c) in bitmap_and_set(),
    ) {
        let expected = &set_a ^ &set_b;

        let mut assign_ref = a.clone();
        assign_ref ^= &b;
        let mut assign_own = a.clone();
        assign_own ^= b.clone();

        for bitmap in [
            assign_ref,
            assign_own,
            &a ^ &b,
            &a ^ b.clone(),
            a.clone() ^ &b,
            a.clone() ^ b.clone(),
            [&a, &b].symmetric_difference(),
            [a.clone(), b.clone()].symmetric_difference(),
        ] {
            prop_assert_eq!(bitmap.len(), expected.len() as u64);
            prop_assert!(bitmap.iter().eq(expected.iter().copied()));
        }

        let expected = &expected ^ &set_c;
        for bitmap in [[&a, &b, &c].symmetric_difference(), [a, b, c].symmetric_difference()] {
            prop_assert_eq!(bitmap.len(), expected.len() as u64);
            prop_assert!(bitmap.iter().eq(expected.iter().copied()));
        }
    }
}
