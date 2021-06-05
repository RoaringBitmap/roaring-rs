use std::ops::Range;

use quickcheck_macros::quickcheck;
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

#[test]
fn multi_bitor() {
    use roaring::bitmap::MultiBitOr;

    let a: RoaringBitmap = (1..4).collect();
    let b: RoaringBitmap = (1000..4000).collect();
    let c: RoaringBitmap = (1_000_000..4_000_000).collect();
    let rbs = [a, b, c];

    let res1 = rbs.bitor();
    let res2 = rbs.iter().cloned().reduce(|a, b| a | b).unwrap_or_default();

    assert_eq!(res1, res2);
}

#[quickcheck]
fn qc_multi_bitor(values: Vec<Range<u32>>) {
    use roaring::bitmap::MultiBitOr;

    let bitmaps: Vec<RoaringBitmap> = values.into_iter().map(|ints| ints.collect()).collect();

    // do the multi union by hand
    let mut byhand = RoaringBitmap::default();
    for rb in &bitmaps {
        byhand |= rb;
    }

    // do it by using the MultiBitOr helper
    let helped = bitmaps.as_slice().bitor();

    assert_eq!(byhand, helped);
}

#[test]
fn multi_bitand() {
    use roaring::bitmap::MultiBitAnd;

    let a: RoaringBitmap = (1..1400).collect();
    let b: RoaringBitmap = (1000..4000).collect();
    let c: RoaringBitmap = (1300..4_000_000).collect();
    let rbs = [a, b, c];

    let res1 = rbs.bitand();
    let res2 = rbs.iter().cloned().reduce(|a, b| a & b).unwrap_or_default();

    assert_eq!(res1, res2);
}

#[quickcheck]
fn qc_multi_bitand(values: Vec<Range<u32>>) {
    use roaring::bitmap::MultiBitAnd;

    let bitmaps: Vec<RoaringBitmap> = values.into_iter().map(|ints| ints.collect()).collect();

    // do the multi union by hand
    let byhand = bitmaps.iter().cloned().reduce(|a, b| a & b).unwrap_or_default();

    // do it by using the MultiBitAnd helper
    let helped = bitmaps.bitand();

    assert_eq!(byhand, helped);
}

#[test]
fn multi_bitxor() {
    use roaring::bitmap::MultiBitXor;

    let a: RoaringBitmap = (1..1400).collect();
    let b: RoaringBitmap = (1000..4000).collect();
    let c: RoaringBitmap = (1300..4_000_000).collect();
    let rbs = [a, b, c];

    let res1 = rbs.bitxor();
    let res2 = rbs.iter().cloned().reduce(|a, b| a ^ b).unwrap_or_default();

    assert_eq!(res1, res2);
}

#[quickcheck]
fn qc_multi_bitxor(values: Vec<Range<u32>>) {
    use roaring::bitmap::MultiBitXor;

    let bitmaps: Vec<RoaringBitmap> = values.into_iter().map(|ints| ints.collect()).collect();

    // do the multi union by hand
    let byhand = bitmaps.iter().cloned().reduce(|a, b| a ^ b).unwrap_or_default();

    // do it by using the MultiBitXor helper
    let helped = bitmaps.bitxor();

    assert_eq!(byhand, helped);
}
