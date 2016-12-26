extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn or() {
    let mut rb1 = RoaringBitmap::from_iter(1..4u32);
    let rb2 = RoaringBitmap::from_iter(3..6u32);
    let rb3 = RoaringBitmap::from_iter(1..6u32);

    assert_eq!(rb3, &rb1 | &rb2);
    assert_eq!(rb3, &rb1 | rb2.clone());
    assert_eq!(rb3, rb1.clone() | &rb2);
    assert_eq!(rb3, rb1.clone() | rb2.clone());

    rb1 |= &rb2;
    rb1 |= rb2;

    assert_eq!(rb3, rb1);
}

#[test]
fn and() {
    let mut rb1 = RoaringBitmap::from_iter(1..4u32);
    let rb2 = RoaringBitmap::from_iter(3..6u32);
    let rb3 = RoaringBitmap::from_iter(3..4u32);

    assert_eq!(rb3, &rb1 & &rb2);
    assert_eq!(rb3, &rb1 & rb2.clone());
    assert_eq!(rb3, rb1.clone() & &rb2);
    assert_eq!(rb3, rb1.clone() & rb2.clone());

    rb1 &= &rb2;
    rb1 &= rb2;

    assert_eq!(rb3, rb1);
}

#[test]
fn sub() {
    let mut rb1 = RoaringBitmap::from_iter(1..4u32);
    let rb2 = RoaringBitmap::from_iter(3..6u32);
    let rb3 = RoaringBitmap::from_iter(1..3u32);

    assert_eq!(rb3, &rb1 - &rb2);
    assert_eq!(rb3, &rb1 - rb2.clone());
    assert_eq!(rb3, rb1.clone() - &rb2);
    assert_eq!(rb3, rb1.clone() - rb2.clone());

    rb1 -= &rb2;
    rb1 -= rb2;

    assert_eq!(rb3, rb1);
}

#[test]
fn xor() {
    let mut rb1 = RoaringBitmap::from_iter(1..4u32);
    let rb2 = RoaringBitmap::from_iter(3..6u32);
    let rb3 = RoaringBitmap::from_iter((1..3u32).chain(4..6));
    let rb4 = RoaringBitmap::from_iter(0..0u32);

    assert_eq!(rb3, &rb1 ^ &rb2);
    assert_eq!(rb3, &rb1 ^ rb2.clone());
    assert_eq!(rb3, rb1.clone() ^ &rb2);
    assert_eq!(rb3, rb1.clone() ^ rb2.clone());

    rb1 ^= &rb2;

    assert_eq!(rb3, rb1);

    rb1 ^= rb3;

    assert_eq!(rb4, rb1);
}
