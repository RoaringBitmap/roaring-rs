extern crate roaring;
use roaring::{RoaringBitmap, RoaringTreemap};

#[test]
fn append() {
    let values = (0..10u32).map(|x| 13 * x).collect::<Vec<u32>>();
    let mut rb1 = RoaringBitmap::new();
    rb1.append(values.clone());

    for (x, y) in rb1.iter().zip(values.iter()) {
        assert_eq!(x, *y);
    }
}

#[test]
fn append_tree() {
    let values = (0..10u64).map(|x| 13 * x).collect::<Vec<u64>>();
    let mut rb1 = RoaringTreemap::new();
    rb1.append(values.clone());

    for (x, y) in rb1.iter().zip(values.iter()) {
        assert_eq!(x, *y);
    }
}
