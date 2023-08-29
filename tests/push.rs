use roaring::{Roaring32, Roaring64};
use std::iter::FromIterator;

/// macro created to reduce code duplication
macro_rules! test_from_sorted_iter {
    ($values: expr, $class: ty) => {{
        let rb1 = <$class>::from_iter($values.clone());
        let rb2 = <$class>::from_sorted_iter($values).unwrap();

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
    test_from_sorted_iter!((0..1_000_000).map(|x| 13 * x).collect::<Vec<u32>>(), Roaring32);
    test_from_sorted_iter!(vec![1, 2, 4, 5, 7, 8, 9], Roaring32);
}

#[test]
fn append_empty() {
    assert_eq!(Roaring32::new().append(vec![]), Ok(0u64))
}

#[test]
fn append_error() {
    match [100u32].iter().cloned().collect::<Roaring32>().append(vec![10, 20, 0]) {
        Ok(_) => {
            panic!("The 0th element in the iterator was < the max of the bitmap")
        }
        Err(non_sorted_error) => {
            assert_eq!(non_sorted_error.valid_until(), 0)
        }
    }

    match [100u32].iter().cloned().collect::<Roaring32>().append(vec![200, 201, 201]) {
        Ok(_) => {
            panic!("The 3rd element in the iterator was < 2nd")
        }
        Err(non_sorted_error) => {
            assert_eq!(non_sorted_error.valid_until(), 2)
        }
    }
}

#[test]
fn append_roaring64() {
    test_from_sorted_iter!((0..1_000_000).map(|x| 13 * x).collect::<Vec<u64>>(), Roaring64);
    test_from_sorted_iter!(vec![1, 2, 4, 5, 7, 8, 9], Roaring64);
}
