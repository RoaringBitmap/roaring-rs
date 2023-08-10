use crate::{RoaringBitmap, Value};
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::marker::PhantomData;

impl<'de, V: Value> Deserialize<'de> for RoaringBitmap<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BitmapVisitor<V> {
            value_type: PhantomData<V>,
        }

        impl<'de, V: Value> Visitor<'de> for BitmapVisitor<V> {
            type Value = RoaringBitmap<V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("roaring bitmap")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<RoaringBitmap<V>, E>
            where
                E: serde::de::Error,
            {
                RoaringBitmap::deserialize_from(bytes).map_err(serde::de::Error::custom)
            }

            // in some case bytes will be serialized as a sequence thus we need to accept both
            // even if it means non optimal performance
            fn visit_seq<A>(self, mut seq: A) -> Result<RoaringBitmap<V>, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bytes: Vec<u8> = Vec::new();
                while let Some(el) = seq.next_element()? {
                    bytes.push(el);
                }
                RoaringBitmap::deserialize_from(&*bytes).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(BitmapVisitor { value_type: PhantomData })
    }
}

impl<V: Value> Serialize for RoaringBitmap<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::new();
        self.serialize_into(&mut buf).map_err(serde::ser::Error::custom)?;

        serializer.serialize_bytes(&buf)
    }
}

#[cfg(test)]
mod test {
    use crate::Roaring32;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serde_json(
            bitmap in Roaring32::arbitrary()
        ) {
            let json = serde_json::to_vec(&bitmap).unwrap();
            prop_assert_eq!(bitmap, serde_json::from_slice(&json).unwrap());
        }

        #[test]
        fn test_bincode(
            bitmap in Roaring32::arbitrary()
        ) {
            let buffer = bincode::serialize(&bitmap).unwrap();
            prop_assert_eq!(bitmap, bincode::deserialize(&buffer).unwrap());
        }
    }
}
