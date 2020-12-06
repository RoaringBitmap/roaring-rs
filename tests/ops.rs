extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn or() {
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (1..6).collect::<RoaringBitmap>();

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
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (3..4).collect::<RoaringBitmap>();

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
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (1..3).collect::<RoaringBitmap>();

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
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (1..3).chain(4..6).collect::<RoaringBitmap>();
    let rb4 = (0..0).collect::<RoaringBitmap>();

    assert_eq!(rb3, &rb1 ^ &rb2);
    assert_eq!(rb3, &rb1 ^ rb2.clone());
    assert_eq!(rb3, rb1.clone() ^ &rb2);
    assert_eq!(rb3, rb1.clone() ^ rb2.clone());

    rb1 ^= &rb2;

    assert_eq!(rb3, rb1);

    rb1 ^= rb3;

    assert_eq!(rb4, rb1);
}
