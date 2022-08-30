use proptest::array::uniform2;
use proptest::collection::vec;
use proptest::prelude::*;
use roaring::RoaringBitmap;

proptest! {
    #[test]
    fn proptest_range(
        range in uniform2(..=262_143_u32),
        extra in vec(..=262_143_u32, ..=100),
    ){
        let range = range[0]..range[1];
        let mut bitmap = RoaringBitmap::new();
        bitmap.insert_range(range.clone());
        assert!(bitmap.contains_range(range.clone()));
        assert_eq!(bitmap.range_cardinality(range.clone()) as usize, range.len());

        for &val in &extra {
            bitmap.insert(val);
            assert!(bitmap.contains_range(range.clone()));
            assert_eq!(bitmap.range_cardinality(range.clone()) as usize, range.len());
        }

        for (i, &val) in extra.iter().filter(|x| range.contains(x)).enumerate() {
            bitmap.remove(val);
            assert!(!bitmap.contains_range(range.clone()));
            assert_eq!(bitmap.range_cardinality(range.clone()) as usize, range.len() - i - 1);
        }
    }
}
