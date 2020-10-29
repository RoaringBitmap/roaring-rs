extern crate roaring;
use roaring::{RoaringBitmap, RoaringTreemap};
use std::iter::FromIterator;

/// macro created to reduce code duplication
macro_rules! test_from_sorted_iter {
    ($values: expr, $class: ty) => {{
        let rb1 = <$class>::from_iter($values.clone());
        let rb2 = <$class>::from_sorted_iter($values);

        for (x, y) in rb1.iter().zip(rb2.iter()) {
            assert_eq!(x, y);
        }
        assert_eq!(rb1.len(), rb2.len());
        assert_eq!(rb1.min(), rb2.min());
        assert_eq!(rb1.max(), rb2.max());
        assert_eq!(rb1.is_empty(), rb2.is_empty());
        assert_eq!(rb1, rb2);
    }};
}

#[test]
fn append() {
    test_from_sorted_iter!(
        (0..1_000_000).map(|x| 13 * x).collect::<Vec<u32>>(),
        RoaringBitmap
    );
    test_from_sorted_iter!(vec![1, 2, 4, 5, 5, 7, 8, 8, 9], RoaringBitmap);
}

#[test]
fn append_tree() {
    test_from_sorted_iter!(
        (0..1_000_000).map(|x| 13 * x).collect::<Vec<u64>>(),
        RoaringTreemap
    );
    test_from_sorted_iter!(vec![1, 2, 4, 5, 5, 7, 8, 8, 9], RoaringTreemap);
}
