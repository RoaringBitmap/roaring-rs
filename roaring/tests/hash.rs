extern crate roaring;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use roaring::RoaringBitmap;

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn equal_bitmaps_have_equal_hashes() {
    let a = (0..2000).collect::<RoaringBitmap>();
    let b = (0..2000).collect::<RoaringBitmap>();

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn empty_bitmaps_hash_equal() {
    assert_eq!(hash_of(&RoaringBitmap::new()), hash_of(&RoaringBitmap::new()));
}

#[test]
fn array_and_run_hash_equal() {
    // A short contiguous range is stored as an array, but `optimize` rewrites it
    // into a run. The two share the same values so they must hash the same.
    let array = (0..100).collect::<RoaringBitmap>();
    let mut run = array.clone();
    assert!(run.optimize());

    assert_eq!(array, run);
    assert_eq!(hash_of(&array), hash_of(&run));
}

#[test]
fn bitmap_and_run_hash_equal() {
    let plain = (0..6000).chain(1_000_000..1_012_000).collect::<RoaringBitmap>();
    let mut optimized = plain.clone();
    assert!(optimized.optimize());

    assert_eq!(plain, optimized);
    assert_eq!(hash_of(&plain), hash_of(&optimized));
}

#[test]
fn usable_as_hashset_key() {
    let mut set = HashSet::new();
    set.insert((0..10).collect::<RoaringBitmap>());
    set.insert((0..10).collect::<RoaringBitmap>());
    set.insert((5..15).collect::<RoaringBitmap>());

    assert_eq!(set.len(), 2);
    assert!(set.contains(&(0..10).collect::<RoaringBitmap>()));
    assert!(!set.contains(&(100..110).collect::<RoaringBitmap>()));
}
