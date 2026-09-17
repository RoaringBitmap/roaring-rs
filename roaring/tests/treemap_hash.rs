extern crate roaring;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use roaring::RoaringTreemap;

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn equal_treemaps_have_equal_hashes() {
    let a = (0..2000u64).collect::<RoaringTreemap>();
    let b = (0..2000u64).collect::<RoaringTreemap>();

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn spanning_multiple_bitmaps_hash_equal() {
    let values = [0u64, 1, u32::MAX as u64, u32::MAX as u64 + 1, u64::MAX];
    let a = values.iter().copied().collect::<RoaringTreemap>();
    let b = values.iter().rev().copied().collect::<RoaringTreemap>();

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn same_values_different_representation_hash_equal() {
    let plain = (0..6000u64).collect::<RoaringTreemap>();
    let mut optimized = plain.clone();
    assert!(optimized.optimize());

    assert_eq!(plain, optimized);
    assert_eq!(hash_of(&plain), hash_of(&optimized));
}

#[test]
fn usable_as_hashset_key() {
    let mut set = HashSet::new();
    set.insert((0..10u64).collect::<RoaringTreemap>());
    set.insert((0..10u64).collect::<RoaringTreemap>());
    set.insert((5..15u64).collect::<RoaringTreemap>());

    assert_eq!(set.len(), 2);
    assert!(set.contains(&(0..10u64).collect::<RoaringTreemap>()));
}
