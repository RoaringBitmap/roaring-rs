extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let original = (0..2000).collect::<RoaringBitmap>();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original = (0..6000).collect::<RoaringBitmap>();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original = (0..2000)
        .chain(1_000_000..1_002_000)
        .chain(2_000_000..2_001_000)
        .collect::<RoaringBitmap>();
    let clone = original.clone();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original = (0..6000)
        .chain(1_000_000..1_012_000)
        .chain(2_000_000..2_010_000)
        .collect::<RoaringBitmap>();
    let clone = original.clone();

    assert_eq!(clone, original);
}
