extern crate roaring;
use roaring::RoaringTreemap;

use std::iter::FromIterator;

#[test]
fn array() {
    let original = (0..2000).collect::<RoaringTreemap>();
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmap() {
    let original = (0..6000).collect::<RoaringTreemap>();
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn arrays() {
    let original = ((0..2000).chain(1_000_000..1_002_000).chain(2_000_000..2_001_000))
        .collect::<RoaringTreemap>();
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps() {
    let original = ((0..6000).chain(1_000_000..1_012_000).chain(2_000_000..2_010_000))
        .collect::<RoaringTreemap>();
    let clone = RoaringTreemap::from_iter(&original);
    let clone2 = RoaringTreemap::from_iter(original.clone());

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn bitmaps_iterator() {
    let original = ((0..6000).chain(1_000_000..1_012_000).chain(2_000_000..2_010_000))
        .collect::<RoaringTreemap>();
    let clone = RoaringTreemap::from_bitmaps(original.bitmaps().map(|(p, b)| (p, b.clone())));
    let clone2 = original.bitmaps().map(|(p, b)| (p, b.clone())).collect::<RoaringTreemap>();

    assert_eq!(clone, original);
    assert_eq!(clone2, original);
}

#[test]
fn rev() {
    let original = (1..3)
        .chain(1_000_000..1_012_003)
        .chain(2_000_001..2_000_003)
        .chain(2_000_000_000_001..2_000_000_000_003)
        .rev()
        .collect::<Vec<_>>();
    let clone = RoaringTreemap::from_iter(original.clone()).iter().rev().collect::<Vec<_>>();

    assert_eq!(clone, original);
}

#[test]
fn double_ended() {
    let mut original_iter = (1..3)
        .chain(1_000_000..1_012_003)
        .chain(2_000_001..2_000_003)
        .chain(2_000_000_000_001..2_000_000_000_003);
    let mut clone_iter = RoaringTreemap::from_iter(original_iter.clone()).into_iter();

    let mut flip = true;
    loop {
        let (original, clone) = if flip {
            (original_iter.next(), clone_iter.next())
        } else {
            (original_iter.next_back(), clone_iter.next_back())
        };
        assert_eq!(clone, original);
        if original.is_none() {
            break;
        }
        flip = !flip;
    }

    // Check again with one more element so we end with the other direction
    let mut original_iter = (1..3)
        .chain(1_000_000..1_012_003)
        .chain(2_000_001..2_000_003)
        .chain(2_000_000_000_001..2_000_000_000_004);
    let mut clone_iter = RoaringTreemap::from_iter(original_iter.clone()).into_iter();

    let mut flip = true;
    loop {
        let (original, clone) = if flip {
            (original_iter.next(), clone_iter.next())
        } else {
            (original_iter.next_back(), clone_iter.next_back())
        };
        assert_eq!(clone, original);
        if original.is_none() {
            break;
        }
        flip = !flip;
    }
}
