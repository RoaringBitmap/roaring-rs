extern crate roaring;
use roaring::RoaringBitmap;

#[test]
fn dense_equality_checks_contents() {
    for pattern in [0x01, 0x55, 0x7f] {
        let bitmap = RoaringBitmap::from_lsb0_bytes(0, &vec![pattern; 3 * 8192]);
        assert_eq!(bitmap.statistics().n_bitset_containers, 3);
        assert_eq!(bitmap, bitmap.clone());

        // Keep cardinality unchanged while changing the first/last words and container boundaries.
        for offset in [0, 64, 65536 - 8, 65536, 3 * 65536 - 8] {
            let mut changed = bitmap.clone();
            assert!(changed.remove(offset));
            assert_ne!(bitmap, changed);
            assert!(changed.insert(offset + 7));
            assert_eq!(bitmap.len(), changed.len());
            assert_ne!(bitmap, changed);
            assert_ne!(changed, bitmap);

            assert!(changed.remove(offset + 7));
            assert!(changed.insert(offset));
            assert_eq!(bitmap, changed);
        }
    }
}

#[test]
fn equality_across_container_representations() {
    for count in [1000, 10000] {
        let bitmap: RoaringBitmap = (0..count).collect();
        let stats = bitmap.statistics();
        if count < 4096 {
            assert_eq!(stats.n_array_containers, 1);
        } else {
            assert_eq!(stats.n_bitset_containers, 1);
        }
        let mut runs = bitmap.clone();
        assert!(runs.optimize());
        assert_eq!(runs.statistics().n_run_containers, 1);
        assert_eq!(bitmap, runs);
        assert_eq!(runs, bitmap);

        assert!(runs.remove(0));
        assert!(runs.insert(count));
        assert_eq!(bitmap.len(), runs.len());
        assert_ne!(bitmap, runs);
        assert_ne!(runs, bitmap);
    }
}

#[test]
fn or() {
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (1..6).collect::<RoaringBitmap>();

    assert_eq!(rb3, &rb1 | &rb2);
    assert_eq!(rb3, &rb1 | rb2.clone());
    assert_eq!(rb3, rb1.clone() | &rb2);
    assert_eq!(rb3, rb1.clone() | rb2.clone());
    assert_eq!(rb3.len(), rb1.union_len(&rb2));

    rb1 |= &rb2;
    rb1 |= rb2;

    assert_eq!(rb3, rb1);
}

#[test]
fn and() {
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (3..4).collect::<RoaringBitmap>();

    assert_eq!(rb3, &rb1 & &rb2);
    assert_eq!(rb3, &rb1 & rb2.clone());
    assert_eq!(rb3, rb1.clone() & &rb2);
    assert_eq!(rb3, rb1.clone() & rb2.clone());
    assert_eq!(rb3.len(), rb1.intersection_len(&rb2));

    rb1 &= &rb2;
    rb1 &= rb2;

    assert_eq!(rb3, rb1);
}

#[test]
fn sub() {
    let mut rb1 = (1..4000).collect::<RoaringBitmap>();
    let rb2 = (3..5000).collect::<RoaringBitmap>();
    let rb3 = (1..3).collect::<RoaringBitmap>();

    assert_eq!(rb3, &rb1 - &rb2);
    assert_eq!(rb3, &rb1 - rb2.clone());
    assert_eq!(rb3, rb1.clone() - &rb2);
    assert_eq!(rb3, rb1.clone() - rb2.clone());
    assert_eq!(rb3.len(), rb1.difference_len(&rb2));

    rb1 -= &rb2;
    rb1 -= rb2;

    assert_eq!(rb3, rb1);
}

// See issue #327
#[test]
fn subtraction_preserves_zero_element() {
    let mut a = RoaringBitmap::from([0, 35, 80, 104, 138, 214, 235, 258]);
    let b = RoaringBitmap::from([9, 35, 42, 51, 111, 134, 231, 239]);

    a -= b;

    // The bug: element 0 should still be present but was being removed
    assert!(a.contains(0), "Element 0 should be present after subtraction");

    // Verify the complete result
    let expected: Vec<u32> = vec![0, 80, 104, 138, 214, 235, 258];
    let actual: Vec<u32> = a.iter().collect();
    assert_eq!(actual, expected, "Subtraction result should match expected values");
}

#[test]
fn xor() {
    let mut rb1 = (1..4).collect::<RoaringBitmap>();
    let rb2 = (3..6).collect::<RoaringBitmap>();
    let rb3 = (1..3).chain(4..6).collect::<RoaringBitmap>();
    let rb4 = (0..0).collect::<RoaringBitmap>();

    assert_eq!(rb3, &rb1 ^ &rb2);
    assert_eq!(rb3, &rb1 ^ rb2.clone());
    assert_eq!(rb3, rb1.clone() ^ &rb2);
    assert_eq!(rb3, rb1.clone() ^ rb2.clone());
    assert_eq!(rb3.len(), rb1.symmetric_difference_len(&rb2));

    rb1 ^= &rb2;

    assert_eq!(rb3, rb1);

    rb1 ^= rb3;

    assert_eq!(rb4, rb1);
}
