use std::fmt;

use RoaringTreemap;

impl fmt::Debug for RoaringTreemap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() < 16 {
            write!(f, "RoaringBitmap<{:?}>", self.iter().collect::<Vec<u64>>())
        } else {
            write!(f,
                   "RoaringBitmap<{:?} values between {:?} and {:?}>",
                   self.len(),
                   self.min().unwrap(),
                   self.max().unwrap())
        }
    }
}
