use crate::{RoaringBitmap, Value};
use std::fmt;

impl<V: Value> fmt::Debug for RoaringBitmap<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() < 16 {
            write!(f, "RoaringBitmap<{:?}>", self.iter().collect::<Vec<V>>())
        } else {
            write!(
                f,
                "RoaringBitmap<{:?} values between {:?} and {:?}>",
                self.len(),
                self.min().unwrap(),
                self.max().unwrap()
            )
        }
    }
}
