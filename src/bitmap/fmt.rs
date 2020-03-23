use std::fmt;

use crate::RoaringBitmap;

impl fmt::Debug for RoaringBitmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() < 16 {
            write!(f, "RoaringBitmap<{:?}>", self.iter().collect::<Vec<u32>>())
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
