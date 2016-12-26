use std::fmt;

use RoaringBitmap;
use util::{ self, ExtInt, Halveable };

impl<Size: ExtInt + Halveable> fmt::Debug for RoaringBitmap<Size> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() < util::cast(16u8) {
            write!(f, "RoaringBitmap<{:?}>", self.iter().collect::<Vec<Size>>())
        } else {
            write!(f, "RoaringBitmap<{:?} values between {:?} and {:?}>", self.len(), self.min().unwrap(), self.max().unwrap())
        }
    }
}
