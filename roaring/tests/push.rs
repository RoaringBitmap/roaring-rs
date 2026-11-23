extern crate roaring;
use roaring::{RoaringBitmap, RoaringTreemap};

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
    test_from_sorted_iter!((0..1_000_000).map(|x| 13 * x).collect::<Vec<u32>>(), RoaringBitmap);
    test_from_sorted_iter!(vec![1, 2, 4, 5, 7, 8, 9], RoaringBitmap);
}

#[test]
fn append_empty() {
    assert_eq!(RoaringBitmap::new().append(vec![]), Ok(0u64))
}

#[test]
fn append_error() {
    match [100u32].iter().cloned().collect::<RoaringBitmap>().append(vec![10, 20, 0]) {
        Ok(_) => {
            panic!("The 0th element in the iterator was < the max of the bitmap")
        }
        Err(non_sorted_error) => {
            assert_eq!(non_sorted_error.valid_until(), 0)
        }
    }

    match [100u32].iter().cloned().collect::<RoaringBitmap>().append(vec![200, 201, 201]) {
        Ok(_) => {
            panic!("The 3rd element in the iterator was < 2nd")
        }
        Err(non_sorted_error) => {
            assert_eq!(non_sorted_error.valid_until(), 2)
        }
    }
}

#[test]
fn append_tree() {
    test_from_sorted_iter!((0..1_000_000).map(|x| 13 * x).collect::<Vec<u64>>(), RoaringTreemap);
    test_from_sorted_iter!(vec![1, 2, 4, 5, 7, 8, 9], RoaringTreemap);
}

// fills more than 4096 values, so the container is a bitmap store.
fn dense() -> RoaringBitmap {
    (0..5000).collect()
}

#[test]
fn bitmap_store_max_after_changes() {
    let mut rb = dense();
    // `h` rises to word 937
    rb.insert(60_000);
    // `h` stays at 937 because remove doesn't update `h`
    rb.remove(60_000);
    assert_eq!(rb, dense()); // same values, so equal even though `h` differs
    assert_eq!(rb.max(), Some(4999));
    assert!(rb.try_push(6000).is_ok());
    assert_eq!(rb.max(), Some(6000));

    // insert_range inside one word, above the current max
    let mut rb = dense();
    rb.insert_range(64_000..=64_010);
    assert_eq!(rb.max(), Some(64_010));

    // insert_range that ends below the current max
    let mut rb = dense();
    rb.insert(60_000);
    rb.insert_range(100..=700);
    assert_eq!(rb.max(), Some(60_000));

    // union and symmetric difference can add bits above the max
    let high: RoaringBitmap = (60_000..65_000).collect();
    assert_eq!((dense() | &high).max(), Some(64_999));
    assert_eq!((dense() ^ &high).max(), Some(64_999));
}

#[test]
fn insert_increasing_and_out_of_order() {
    // increasing values take the fast path, the rest use binary search
    let mut rb = RoaringBitmap::new();
    for v in [10, 20, 70_000, 5, 15, 70_000, 200_000, 65_536] {
        rb.insert(v);
    }
    assert_eq!(rb.iter().collect::<Vec<_>>(), [5, 10, 15, 20, 65_536, 70_000, 200_000]);

    let expected = RoaringBitmap::from_sorted_iter((0..1_000_000).step_by(64)).unwrap();
    let mut rb = RoaringBitmap::new();
    for v in (0..1_000_000).step_by(64) {
        rb.insert(v);
    }
    assert_eq!(rb, expected);
}
