use roaring::{RoaringBitmap, SkipTo};

#[test]
fn basic() {
    let bm = RoaringBitmap::from([1,2,3,4,11,12,13,14]);
    let mut i = bm.iter().skip_to(10);
    for n in 11..=14 {
        assert_eq!(i.next(), Some(n))
    }
    assert_eq!(i.next(), None);

}