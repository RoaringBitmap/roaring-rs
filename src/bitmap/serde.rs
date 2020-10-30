use serde_cr::{Serialize, Serializer};
use serde_cr::{Deserialize, Deserializer};

use crate::RoaringBitmap;

impl Serialize for RoaringBitmap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = Vec::with_capacity(self.serialized_size());
        self.serialize_into(&mut buffer).unwrap();
        serde_bytes::serialize(&buffer, serializer)
    }
}

impl<'de> Deserialize<'de> for RoaringBitmap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?;
        let bytes = bytes.into_vec();
        Ok(RoaringBitmap::deserialize_from(&*bytes).unwrap())
    }
}
