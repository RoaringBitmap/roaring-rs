extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::FromIterator;

#[test]
fn append() {
    let values = (0..10u32).map(|x| 13 * x).collect::<Vec<u32>>();
    let mut rb1 = RoaringBitmap::new();
    rb1.append(values.clone());

    for (x, y) in rb1.iter().zip(values.iter()) {
        assert_eq!(x, *y);
    }
}
