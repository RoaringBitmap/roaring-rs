use roaring::RoaringBitmap;

#[test]
fn iter_basic() {
    let bm = RoaringBitmap::from([1, 2, 3, 4, 11, 12, 13, 14]);
    let mut i = bm.iter().advance_to(10);
    for n in 11..=14 {
        assert_eq!(i.next(), Some(n))
    }
    assert_eq!(i.next(), None);
}

#[test]
fn iter_advance_past_end() {
    let bm = RoaringBitmap::from([1, 2, 3, 4, 11, 12, 13, 14]);
    let mut i = bm.iter().advance_to(15);
    assert_eq!(i.next(), None);
}

#[test]
fn iter_multi_container() {
    let bm = RoaringBitmap::from([1, 2, 3, 100000, 100001]);
    let mut i = bm.iter().advance_to(3);
    assert_eq!(i.next(), Some(3));
    assert_eq!(i.next(), Some(100000));
    assert_eq!(i.next(), Some(100001));
    assert_eq!(i.next(), None);
}

#[test]
fn iter_empty() {
    let bm = RoaringBitmap::new();
    assert_eq!(bm.iter().advance_to(31337).next(), None)
}

#[test]
fn into_iter_basic() {
    let bm = RoaringBitmap::from([1, 2, 3, 4, 11, 12, 13, 14]);
    let mut i = bm.into_iter().advance_to(10);
    for n in 11..=14 {
        assert_eq!(i.next(), Some(n))
    }
    assert_eq!(i.next(), None);
}

#[test]
fn into_iter_multi_container() {
    let bm = RoaringBitmap::from([1, 2, 3, 100000, 100001]);
    let mut i = bm.into_iter().advance_to(3);
    assert_eq!(i.next(), Some(3));
    assert_eq!(i.next(), Some(100000));
    assert_eq!(i.next(), Some(100001));
    assert_eq!(i.next(), None);
}

#[test]
fn into_iter_empty() {
    let bm = RoaringBitmap::new();
    assert_eq!(bm.into_iter().advance_to(31337).next(), None)
}

#[test]
fn iter_from() {
    let bm = RoaringBitmap::from([1, 2, 3, 100000, 100001]);
    let mut i = bm.iter_from(99999);
    assert_eq!(i.next(), Some(100000));
    assert_eq!(i.next(), Some(100001));
    assert_eq!(i.next(), None);
}
