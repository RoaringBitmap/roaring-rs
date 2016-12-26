use std::ops::{
    BitAnd, BitAndAssign,
    BitOr, BitOrAssign,
    BitXor, BitXorAssign,
    Sub, SubAssign
};

use num::traits::Zero;

use RoaringBitmap;
use util::{ Halveable, ExtInt };

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    /// Unions in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// rb1.union_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitOr` operator.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..5u32).collect();
    ///
    /// let rb1 = rb1 | rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn union_with(&mut self, other: &RoaringBitmap<Size>) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => self.containers[loc].union_with(container),
            }
        }
    }

    /// Intersects in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (3..4u32).collect();
    ///
    /// rb1.intersect_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitAnd` operator.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (3..4u32).collect();
    ///
    /// let rb1 = rb1 & rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn intersect_with(&mut self, other: &RoaringBitmap<Size>) {
        let mut index = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            match other.containers.binary_search_by_key(&key, |c| c.key) {
                Err(_) => { self.containers.remove(index); }
                Ok(loc) => {
                    self.containers[index].intersect_with(&other.containers[loc]);
                    if self.containers[index].len == Zero::zero() {
                        self.containers.remove(index);
                    } else {
                        index += 1;
                    }
                }
            }
        }
    }

    /// Removes all values in the specified other bitmap from self, in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// rb1.difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `Sub` operator.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..5u32).collect();
    /// let rb3: RoaringBitmap<u32> = (1..3u32).collect();
    ///
    /// let rb1 = rb1 - rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn difference_with(&mut self, other: &RoaringBitmap<Size>) {
        let mut index = 0;
        while index < self.containers.len() {
            let key = self.containers[index].key;
            match other.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    self.containers[index].difference_with(&other.containers[loc]);
                    if self.containers[index].len == Zero::zero() {
                        self.containers.remove(index);
                    } else {
                        index += 1;
                    }
                },
                _ => { index += 1; }
            }
        }
    }

    /// Replaces this bitmap with one that is equivalent to `self XOR other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// rb1.symmetric_difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitXor` operator.
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = (1..4u32).collect();
    /// let rb2: RoaringBitmap<u32> = (3..6u32).collect();
    /// let rb3: RoaringBitmap<u32> = ((1..3u32).chain(4..6u32)).collect();
    ///
    /// let rb1 = rb1 ^ rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn symmetric_difference_with(&mut self, other: &RoaringBitmap<Size>) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => {
                    self.containers[loc].symmetric_difference_with(container);
                    if self.containers[loc].len == Zero::zero() {
                        self.containers.remove(loc);
                    }
                }
            }
        }
    }
}

impl<Size: ExtInt + Halveable> BitOr<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitor(mut self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.union_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitOr<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitor(mut self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.union_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitOr<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitor(self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs | self
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitOr<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitor(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.clone() | rhs
    }
}

impl<Size: ExtInt + Halveable> BitOrAssign<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitor_assign(&mut self, rhs: RoaringBitmap<Size>) {
        self.union_with(&rhs)
    }
}

impl<'a, Size: ExtInt + Halveable> BitOrAssign<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitor_assign(&mut self, rhs: &'a RoaringBitmap<Size>) {
        self.union_with(rhs)
    }
}

impl<Size: ExtInt + Halveable> BitAnd<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitand(mut self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.intersect_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitAnd<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitand(mut self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.intersect_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitAnd<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitand(self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs & self
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitAnd<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitand(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.clone() & rhs
    }
}

impl<Size: ExtInt + Halveable> BitAndAssign<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitand_assign(&mut self, rhs: RoaringBitmap<Size>) {
        self.intersect_with(&rhs)
    }
}

impl<'a, Size: ExtInt + Halveable> BitAndAssign<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitand_assign(&mut self, rhs: &'a RoaringBitmap<Size>) {
        self.intersect_with(rhs)
    }
}

impl<Size: ExtInt + Halveable> Sub<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn sub(mut self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.difference_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> Sub<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn sub(mut self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.difference_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> Sub<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn sub(self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.clone() - rhs
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> Sub<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn sub(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.clone() - rhs
    }
}

impl<Size: ExtInt + Halveable> SubAssign<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn sub_assign(&mut self, rhs: RoaringBitmap<Size>) {
        self.difference_with(&rhs)
    }
}

impl<'a, Size: ExtInt + Halveable> SubAssign<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn sub_assign(&mut self, rhs: &'a RoaringBitmap<Size>) {
        self.difference_with(rhs)
    }
}

impl<Size: ExtInt + Halveable> BitXor<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitxor(mut self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.symmetric_difference_with(&rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitXor<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitxor(mut self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.symmetric_difference_with(rhs);
        self
    }
}

impl<'a, Size: ExtInt + Halveable> BitXor<RoaringBitmap<Size>> for &'a RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitxor(self, rhs: RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        rhs ^ self
    }
}

impl<'a, 'b, Size: ExtInt + Halveable> BitXor<&'a RoaringBitmap<Size>> for &'b RoaringBitmap<Size> {
    type Output = RoaringBitmap<Size>;

    fn bitxor(self, rhs: &'a RoaringBitmap<Size>) -> RoaringBitmap<Size> {
        self.clone() ^ rhs
    }
}

impl<Size: ExtInt + Halveable> BitXorAssign<RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitxor_assign(&mut self, rhs: RoaringBitmap<Size>) {
        self.symmetric_difference_with(&rhs)
    }
}

impl<'a, Size: ExtInt + Halveable> BitXorAssign<&'a RoaringBitmap<Size>> for RoaringBitmap<Size> {
    fn bitxor_assign(&mut self, rhs: &'a RoaringBitmap<Size>) {
        self.symmetric_difference_with(rhs)
    }
}
