#![feature(slicing_syntax)]

extern crate roaring;

use roaring::RoaringBitmap;

#[test]
fn or() {
    let rb1: RoaringBitmap<u32> = (1..4).collect();
    let rb2: RoaringBitmap<u32> = (3..6).collect();
    let rb3: RoaringBitmap<u32> = (1..6).collect();

    assert_eq!(rb3, &rb1 | &rb2);
    assert_eq!(rb3, rb1 | rb2 | &rb3);
}

#[test]
fn and() {
    let rb1: RoaringBitmap<u32> = (1..4).collect();
    let rb2: RoaringBitmap<u32> = (3..6).collect();
    let rb3: RoaringBitmap<u32> = (3..4).collect();

    assert_eq!(rb3, &rb1 & &rb2);
    assert_eq!(rb3, rb1 & rb2 & &rb3);
}

#[test]
fn sub() {
    let rb1: RoaringBitmap<u32> = (1..4).collect();
    let rb2: RoaringBitmap<u32> = (3..6).collect();
    let rb3: RoaringBitmap<u32> = (1..3).collect();
    let rb4: RoaringBitmap<u32> = (0..0).collect();

    assert_eq!(rb3, &rb1 - &rb2);
    assert_eq!(rb4, rb1 - rb2 - rb3);
}

#[test]
fn xor() {
    let rb1: RoaringBitmap<u32> = (1..4).collect();
    let rb2: RoaringBitmap<u32> = (3..6).collect();
    let rb3: RoaringBitmap<u32> = (1..3).chain(4..6).collect();
    let rb4: RoaringBitmap<u32> = (0..0).collect();

    assert_eq!(rb3, &rb1 ^ &rb2);
    assert_eq!(rb4, rb1 ^ rb2 ^ rb3);
}

