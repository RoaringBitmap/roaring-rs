use proptest::arbitrary::any;
use proptest::collection::btree_set;
use proptest::proptest;
use roaring::RoaringBitmap;

/// Test basic next_many functionality with a simple range
#[test]
fn next_many_simple() {
    let bitmap: RoaringBitmap = (0..100).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 32];

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 32);
    assert_eq!(&buf[..n], &(0..32).collect::<Vec<_>>()[..]);

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 32);
    assert_eq!(&buf[..n], &(32..64).collect::<Vec<_>>()[..]);

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 32);
    assert_eq!(&buf[..n], &(64..96).collect::<Vec<_>>()[..]);

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 4);
    assert_eq!(&buf[..n], &[96, 97, 98, 99]);

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 0);
}

/// Test next_many with IntoIter (owned iterator)
#[test]
fn next_many_into_iter() {
    let bitmap: RoaringBitmap = (0..100).collect();
    let mut iter = bitmap.into_iter();
    let mut buf = [0u32; 32];
    let mut all_values = Vec::new();

    loop {
        let n = iter.next_many(&mut buf);
        if n == 0 {
            break;
        }
        all_values.extend_from_slice(&buf[..n]);
    }

    let expected: Vec<u32> = (0..100).collect();
    assert_eq!(all_values, expected);
}

/// Test next_many with empty buffer
#[test]
fn next_many_empty_buffer() {
    let bitmap: RoaringBitmap = (0..10).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 0];

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 0);
    // Iterator should not be advanced
    assert_eq!(iter.next(), Some(0));
}

/// Test next_many with empty bitmap
#[test]
fn next_many_empty_bitmap() {
    let bitmap = RoaringBitmap::new();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 32];

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 0);
}

/// Test next_many across multiple containers
#[test]
fn next_many_multiple_containers() {
    // Container boundary is at 65536
    let bitmap: RoaringBitmap = (65530..65545).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 32];

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 15);
    let expected: Vec<u32> = (65530..65545).collect();
    assert_eq!(&buf[..n], &expected[..]);
}

/// Test next_many with large buffer
#[test]
fn next_many_large_buffer() {
    let bitmap: RoaringBitmap = (0..50).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 1000];

    let n = iter.next_many(&mut buf);
    assert_eq!(n, 50);
    let expected: Vec<u32> = (0..50).collect();
    assert_eq!(&buf[..n], &expected[..]);
}

/// Test next_many with bitmap store (dense values)
#[test]
fn next_many_bitmap_store() {
    // More than 4096 values in a container triggers bitmap storage
    let bitmap: RoaringBitmap = (0..10000).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 512];
    let mut all_values = Vec::new();

    loop {
        let n = iter.next_many(&mut buf);
        if n == 0 {
            break;
        }
        all_values.extend_from_slice(&buf[..n]);
    }

    let expected: Vec<u32> = (0..10000).collect();
    assert_eq!(all_values, expected);
}

/// Test next_many with run store (consecutive values)
#[test]
fn next_many_run_store() {
    let mut bitmap = RoaringBitmap::new();
    bitmap.insert_range(0..1000);
    bitmap.insert_range(2000..3000);
    
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 256];
    let mut all_values = Vec::new();

    loop {
        let n = iter.next_many(&mut buf);
        if n == 0 {
            break;
        }
        all_values.extend_from_slice(&buf[..n]);
    }

    let expected: Vec<u32> = (0..1000).chain(2000..3000).collect();
    assert_eq!(all_values, expected);
}

/// Test interleaving next_many with next()
#[test]
fn next_many_interleaved_with_next() {
    let bitmap: RoaringBitmap = (0..100).collect();
    let mut iter = bitmap.iter();
    let mut buf = [0u32; 10];

    // Read first 10 via next_many
    let n = iter.next_many(&mut buf);
    assert_eq!(n, 10);
    assert_eq!(&buf[..n], &(0..10).collect::<Vec<_>>()[..]);

    // Read one via next
    assert_eq!(iter.next(), Some(10));

    // Read next 10 via next_many
    let n = iter.next_many(&mut buf);
    assert_eq!(n, 10);
    assert_eq!(&buf[..n], &(11..21).collect::<Vec<_>>()[..]);

    // Read one via next
    assert_eq!(iter.next(), Some(21));
}

/// Test next_many preserves no gaps/duplicates
proptest! {
    #[test]
    fn next_many_correctness(values in btree_set(any::<u32>(), ..=10_000)) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();
        let mut iter = bitmap.iter();
        let mut buf = [0u32; 128];
        let mut collected = Vec::new();

        loop {
            let n = iter.next_many(&mut buf);
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&buf[..n]);
        }

        let expected: Vec<u32> = values.into_iter().collect();
        assert_eq!(collected, expected);
    }
}

/// Test next_many with various buffer sizes
proptest! {
    #[test]
    fn next_many_various_buffer_sizes(
        values in btree_set(any::<u32>(), 100..=1000),
        buf_size in 1usize..=500
    ) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();
        let mut iter = bitmap.iter();
        let mut buf = vec![0u32; buf_size];
        let mut collected = Vec::new();

        loop {
            let n = iter.next_many(&mut buf);
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&buf[..n]);
        }

        let expected: Vec<u32> = values.into_iter().collect();
        assert_eq!(collected, expected);
    }
}

/// Test next_many with IntoIter correctness
proptest! {
    #[test]
    fn next_many_into_iter_correctness(values in btree_set(any::<u32>(), ..=10_000)) {
        let bitmap = RoaringBitmap::from_sorted_iter(values.iter().cloned()).unwrap();
        let mut iter = bitmap.into_iter();
        let mut buf = [0u32; 128];
        let mut collected = Vec::new();

        loop {
            let n = iter.next_many(&mut buf);
            if n == 0 {
                break;
            }
            collected.extend_from_slice(&buf[..n]);
        }

        let expected: Vec<u32> = values.into_iter().collect();
        assert_eq!(collected, expected);
    }
}
