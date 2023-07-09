#[cfg(test)]
mod test {
    use crate::bitmap::container::Container;
    use crate::bitmap::store::{ArrayStore, BitmapStore, Store};
    use crate::RoaringBitmap;
    use proptest::bits::{BitSetLike, BitSetStrategy, SampledBitSetStrategy};
    use proptest::collection::{vec, SizeRange};
    use proptest::prelude::*;
    use std::fmt::{Debug, Formatter};

    impl Debug for BitmapStore {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.len() < 16 {
                write!(f, "BitmapStore<{:?}>", self.iter().collect::<Vec<u16>>())
            } else {
                write!(
                    f,
                    "BitmapStore<{:?} values between {:?} and {:?}>",
                    self.len(),
                    self.min().unwrap(),
                    self.max().unwrap()
                )
            }
        }
    }

    impl BitSetLike for BitmapStore {
        fn new_bitset(max: usize) -> Self {
            assert!(max <= BitmapStore::MAX + 1);
            BitmapStore::new()
        }

        fn len(&self) -> usize {
            BitmapStore::MAX + 1
        }

        fn test(&self, bit: usize) -> bool {
            assert!(bit <= BitmapStore::MAX);
            self.contains(bit as u16)
        }

        fn set(&mut self, bit: usize) {
            assert!(bit <= BitmapStore::MAX);
            self.insert(bit as u16);
        }

        fn clear(&mut self, bit: usize) {
            assert!(bit <= BitmapStore::MAX);
            self.remove(bit as u16);
        }

        fn count(&self) -> usize {
            self.len() as usize
        }
    }

    impl BitmapStore {
        const MAX: usize = u16::MAX as usize;

        pub fn universe() -> Self {
            BitmapStore::try_from(1 + u16::MAX as u64, Box::new([u64::MAX; 1024])).unwrap()
        }

        pub fn between(min: u16, max: u16) -> BitSetStrategy<Self> {
            BitSetStrategy::new(min as usize, max as usize)
        }

        pub fn masked(mask: Self) -> BitSetStrategy<Self> {
            BitSetStrategy::masked(mask)
        }

        pub fn sampled(
            size: impl Into<SizeRange>,
            bits: impl Into<SizeRange>,
        ) -> SampledBitSetStrategy<Self> {
            SampledBitSetStrategy::new(size.into(), bits.into())
        }

        pub fn arbitrary() -> SampledBitSetStrategy<Self> {
            Self::sampled(..=u16::MAX as usize, ..=u16::MAX as usize)
        }
    }

    impl Debug for ArrayStore {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.len() < 16 {
                write!(f, "ArrayStore<{:?}>", self.as_slice())
            } else {
                write!(
                    f,
                    "ArrayStore<{:?} values between {:?} and {:?}>",
                    self.len(),
                    self.min().unwrap(),
                    self.max().unwrap()
                )
            }
        }
    }

    impl BitSetLike for ArrayStore {
        fn new_bitset(max: usize) -> Self {
            assert!(max <= ArrayStore::MAX + 1);
            ArrayStore::new()
        }

        fn len(&self) -> usize {
            ArrayStore::MAX + 1
        }

        fn test(&self, bit: usize) -> bool {
            assert!(bit <= ArrayStore::MAX);
            self.contains(bit as u16)
        }

        fn set(&mut self, bit: usize) {
            assert!(bit <= ArrayStore::MAX);
            self.insert(bit as u16);
        }

        fn clear(&mut self, bit: usize) {
            assert!(bit <= ArrayStore::MAX);
            self.remove(bit as u16);
        }

        fn count(&self) -> usize {
            self.len() as usize
        }
    }

    impl ArrayStore {
        const MAX: usize = u16::MAX as usize;

        pub fn between(min: u16, max: u16) -> BitSetStrategy<ArrayStore> {
            BitSetStrategy::new(min as usize, max as usize)
        }

        pub fn masked(mask: ArrayStore) -> BitSetStrategy<ArrayStore> {
            BitSetStrategy::masked(mask)
        }

        pub fn sampled(
            size: impl Into<SizeRange>,
            bits: impl Into<SizeRange>,
        ) -> SampledBitSetStrategy<ArrayStore> {
            SampledBitSetStrategy::new(size.into(), bits.into())
        }

        pub fn arbitrary() -> SampledBitSetStrategy<ArrayStore> {
            Self::sampled(..=4096_usize, ..=u16::MAX as usize)
        }
    }

    impl Debug for Store {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Store::Array(a) => write!(f, "Store({:?})", a),
                Store::Bitmap(b) => write!(f, "Store({:?})", b),
            }
        }
    }

    impl Store {
        fn arbitrary() -> impl Strategy<Value = Store> {
            prop_oneof![
                ArrayStore::sampled(1..=4096, ..=u16::MAX as usize).prop_map(Store::Array),
                BitmapStore::sampled(4097..u16::MAX as usize, ..=u16::MAX as usize)
                    .prop_map(Store::Bitmap),
            ]
        }
    }

    prop_compose! {
        fn containers(n: usize)
                     (keys in ArrayStore::sampled(..=n, ..=n),
                      stores in vec(Store::arbitrary(), n)) -> RoaringBitmap {
            let containers = keys.into_iter().zip(stores).map(|(key, store)| {
                let mut container = Container { key, store };
                container.ensure_correct_store();
                container
            }).collect::<Vec<Container>>();
            RoaringBitmap { containers }
       }
    }

    impl RoaringBitmap {
        prop_compose! {
            pub fn arbitrary()(bitmap in (0usize..=16).prop_flat_map(containers)) -> RoaringBitmap {
                bitmap
            }
        }
    }
}
