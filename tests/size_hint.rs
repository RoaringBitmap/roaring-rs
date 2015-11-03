extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn array() {
    let bitmap: RoaringBitmap<u32> = (0..2000u32).collect();
    assert_eq!((2000, Some(2000)), bitmap.iter().size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(2000).size_hint());
}

#[test]
fn bitmap() {
    let bitmap: RoaringBitmap<u32> = (0..6000u32).collect();
    assert!(bitmap.iter().size_hint().0 > 5000);
    assert_eq!(Some(6000), bitmap.iter().size_hint().1);
    assert_eq!((0, Some(0)), bitmap.iter().skip(6000).size_hint());
}

#[test]
fn arrays() {
    let bitmap: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    assert_eq!((5000, Some(5000)), bitmap.iter().size_hint());
    assert_eq!((0, Some(0)), bitmap.iter().skip(5000).size_hint());
}

#[test]
fn bitmaps() {
    let bitmap: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();

    assert!(bitmap.iter().size_hint().0 > 27000);
    assert_eq!(Some(28000), bitmap.iter().size_hint().1);

    assert!(bitmap.iter().skip(100).size_hint().0 > 27000);
    assert!(bitmap.iter().skip(100).size_hint().1.is_some());
    assert!(bitmap.iter().skip(100).size_hint().1 >= Some(27900));

    assert!(bitmap.iter().skip(2000).size_hint().0 < 27000);
    assert!(bitmap.iter().skip(2000).size_hint().0 > 25000);
    assert!(bitmap.iter().skip(2000).size_hint().1.is_some());
    assert!(bitmap.iter().skip(2000).size_hint().1 >= Some(26000));

    assert!(bitmap.iter().skip(7000).size_hint().0 < 22000);
    assert!(bitmap.iter().skip(7000).size_hint().0 > 20000);
    assert!(bitmap.iter().skip(7000).size_hint().1.is_some());
    assert!(bitmap.iter().skip(7000).size_hint().1 >= Some(21000));

    assert!(bitmap.iter().skip(27000).size_hint().0 < 2000);
    assert!(bitmap.iter().skip(27000).size_hint().0 > 0);
    assert!(bitmap.iter().skip(27000).size_hint().1.is_some());
    assert!(bitmap.iter().skip(27000).size_hint().1 >= Some(1000));

    assert_eq!((0, Some(0)), bitmap.iter().skip(28000).size_hint());
}
