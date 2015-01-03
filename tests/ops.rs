extern crate roaring;

use roaring::RoaringBitmap;

#[test]
fn or() {
    let rb1: RoaringBitmap = FromIterator::from_iter(1..4);
    let rb2: RoaringBitmap = FromIterator::from_iter(3..6);
    let rb3: RoaringBitmap = FromIterator::from_iter(1..6);

    assert_eq!(rb3, &rb1 | &rb2);
    assert_eq!(rb3, rb1 | rb2 | &rb3);
}

#[test]
fn and() {
    let rb1: RoaringBitmap = FromIterator::from_iter(1..4);
    let rb2: RoaringBitmap = FromIterator::from_iter(3..6);
    let rb3: RoaringBitmap = FromIterator::from_iter(3..4);

    assert_eq!(rb3, &rb1 & &rb2);
    assert_eq!(rb3, rb1 & rb2 & &rb3);
}

#[test]
fn sub() {
    let rb1: RoaringBitmap = FromIterator::from_iter(1..4);
    let rb2: RoaringBitmap = FromIterator::from_iter(3..6);
    let rb3: RoaringBitmap = FromIterator::from_iter(1..3);
    let rb4: RoaringBitmap = FromIterator::from_iter(0..0);

    assert_eq!(rb3, &rb1 - &rb2);
    assert_eq!(rb4, rb1 - rb2 - rb3);
}

#[test]
fn xor() {
    let rb1: RoaringBitmap = FromIterator::from_iter(1..4);
    let rb2: RoaringBitmap = FromIterator::from_iter(3..6);
    let rb3: RoaringBitmap = FromIterator::from_iter((1..3).chain(4..6));
    let rb4: RoaringBitmap = FromIterator::from_iter(0..0);

    assert_eq!(rb3, &rb1 ^ &rb2);
    assert_eq!(rb4, rb1 ^ rb2 ^ rb3);
}

