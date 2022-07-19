mod arbitrary;
mod container;
mod fmt;
mod multiops;
mod proptests;
mod store;
mod util;

// Order of these modules matters as it determines the `impl` blocks order in
// the docs
mod cmp;
mod inherent;
mod iter;
mod ops;
mod serialization;

use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use self::cmp::Pairs;
pub use self::iter::IntoIter;
pub use self::iter::Iter;

/// A compressed bitmap using the [Roaring bitmap compression scheme](https://roaringbitmap.org/).
///
/// # Examples
///
/// ```rust
/// use roaring::RoaringBitmap;
///
/// let mut rb = RoaringBitmap::new();
///
/// // insert all primes less than 10
/// rb.insert(2);
/// rb.insert(3);
/// rb.insert(5);
/// rb.insert(7);
/// println!("total bits set to true: {}", rb.len());
/// ```
#[derive(PartialEq)]
pub struct RoaringBitmap {
    containers: Vec<container::Container>,
}

impl<'de> Deserialize<'de> for RoaringBitmap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BitmapVisitor;

        impl<'de> Visitor<'de> for BitmapVisitor {
            type Value = RoaringBitmap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("roaring bitmap")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<RoaringBitmap, E>
            where
                E: serde::de::Error,
            {
                RoaringBitmap::deserialize_from(bytes).map_err(serde::de::Error::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<RoaringBitmap, A::Error>
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

        deserializer.deserialize_bytes(BitmapVisitor)
    }
}

impl Serialize for RoaringBitmap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = Vec::new();
        self.serialize_into(&mut buf).map_err(serde::ser::Error::custom)?;

        serializer.serialize_bytes(&buf)
    }
}
