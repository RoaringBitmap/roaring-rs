use retain_mut::RetainMut;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use crate::RoaringBitmap;

impl RoaringBitmap {
    /// Unions in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
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
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..5).collect();
    ///
    /// let rb1 = rb1 | rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn union_with(&mut self, other: &RoaringBitmap) {
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
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (3..4).collect();
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
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (3..4).collect();
    ///
    /// let rb1 = rb1 & rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn intersect_with(&mut self, other: &RoaringBitmap) {
        self.containers.retain_mut(|cont| {
            match other.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    cont.intersect_with(&other.containers[loc]);
                    cont.len != 0
                }
                Err(_) => false,
            }
        })
    }

    /// Removes all values in the specified other bitmap from self, in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
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
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    /// let rb3: RoaringBitmap = (1..3).collect();
    ///
    /// let rb1 = rb1 - rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn difference_with(&mut self, other: &RoaringBitmap) {
        self.containers.retain_mut(|cont| {
            match other.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    cont.difference_with(&other.containers[loc]);
                    cont.len != 0
                }
                Err(_) => true,
            }
        })
    }

    /// Replaces this bitmap with one that is equivalent to `self XOR other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = (1..3).chain(4..6).collect();
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
    /// let mut rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..6).collect();
    /// let rb3: RoaringBitmap = (1..3).chain(4..6).collect();
    ///
    /// let rb1 = rb1 ^ rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn symmetric_difference_with(&mut self, other: &RoaringBitmap) {
        for container in &other.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => {
                    self.containers[loc].symmetric_difference_with(container);
                    if self.containers[loc].len == 0 {
                        self.containers.remove(loc);
                    }
                }
            }
        }
    }
}

impl BitOr<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(mut self, mut rhs: RoaringBitmap) -> RoaringBitmap {
        if self.len() <= rhs.len() {
            rhs.union_with(&self);
            rhs
        } else {
            self.union_with(&rhs);
            self
        }
    }
}

impl BitOr<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.union_with(rhs);
        self
    }
}

impl BitOr<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        rhs | self
    }
}

impl BitOr<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        if self.len() <= rhs.len() {
            rhs.clone() | self
        } else {
            self.clone() | rhs
        }
    }
}

impl BitOrAssign<RoaringBitmap> for RoaringBitmap {
    fn bitor_assign(&mut self, mut rhs: RoaringBitmap) {
        if self.len() <= rhs.len() {
            rhs.union_with(&self);
            *self = rhs;
        } else {
            self.union_with(&rhs);
        }
    }
}

impl BitOrAssign<&RoaringBitmap> for RoaringBitmap {
    fn bitor_assign(&mut self, rhs: &RoaringBitmap) {
        self.union_with(rhs)
    }
}

impl BitAnd<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(mut self, mut rhs: RoaringBitmap) -> RoaringBitmap {
        if self.len() <= rhs.len() {
            self.intersect_with(&rhs);
            self
        } else {
            rhs.intersect_with(&self);
            rhs
        }
    }
}

impl BitAnd<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        self.intersect_with(rhs);
        self
    }
}

impl BitAnd<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(self, rhs: RoaringBitmap) -> RoaringBitmap {
        rhs & self
    }
}

impl BitAnd<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    fn bitand(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        if self.len() <= rhs.len() {
            self.clone() & rhs
        } else {
            rhs.clone() & self
        }
    }
}

impl BitAndAssign<RoaringBitmap> for RoaringBitmap {
    fn bitand_assign(&mut self, mut rhs: RoaringBitmap) {
        if self.len() <= rhs.len() {
            self.intersect_with(&rhs);
        } else {
            rhs.intersect_with(self);
            *self = rhs;
        }
    }
}

impl BitAndAssign<&RoaringBitmap> for RoaringBitmap {
    fn bitand_assign(&mut self, rhs: &RoaringBitmap) {
        self.intersect_with(rhs)
    }
}

impl Sub<RoaringBitmap> for RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn sub(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.difference_with(&rhs);
        self
    }
}

impl<'a> Sub<&'a RoaringBitmap> for RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn sub(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.difference_with(rhs);
        self
    }
}

impl<'a> Sub<RoaringBitmap> for &'a RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn sub(self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.clone() - rhs
    }
}

impl<'a, 'b> Sub<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn sub(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.clone() - rhs
    }
}

impl SubAssign<RoaringBitmap> for RoaringBitmap {
    fn sub_assign(&mut self, rhs: RoaringBitmap) {
        self.difference_with(&rhs)
    }
}

impl<'a> SubAssign<&'a RoaringBitmap> for RoaringBitmap {
    fn sub_assign(&mut self, rhs: &'a RoaringBitmap) {
        self.difference_with(rhs)
    }
}

impl BitXor<RoaringBitmap> for RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn bitxor(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference_with(&rhs);
        self
    }
}

impl<'a> BitXor<&'a RoaringBitmap> for RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn bitxor(mut self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.symmetric_difference_with(rhs);
        self
    }
}

impl<'a> BitXor<RoaringBitmap> for &'a RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn bitxor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        rhs ^ self
    }
}

impl<'a, 'b> BitXor<&'a RoaringBitmap> for &'b RoaringBitmap {
    type Output = crate::RoaringBitmap;

    fn bitxor(self, rhs: &'a RoaringBitmap) -> RoaringBitmap {
        self.clone() ^ rhs
    }
}

impl BitXorAssign<RoaringBitmap> for RoaringBitmap {
    fn bitxor_assign(&mut self, rhs: RoaringBitmap) {
        self.symmetric_difference_with(&rhs)
    }
}

impl<'a> BitXorAssign<&'a RoaringBitmap> for RoaringBitmap {
    fn bitxor_assign(&mut self, rhs: &'a RoaringBitmap) {
        self.symmetric_difference_with(rhs)
    }
}
