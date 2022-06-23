#[cfg(test)]
mod test {
    use crate::{RoaringBitmap, RoaringTreemap};
    use proptest::collection::btree_map;
    use proptest::prelude::*;

    impl RoaringTreemap {
        prop_compose! {
            pub fn arbitrary()(map in btree_map(0u32..=16, RoaringBitmap::arbitrary(), 0usize..=16)) -> RoaringTreemap {
               RoaringTreemap { map }
           }
        }
    }
}
