use std::collections::btree_map::Entry;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use crate::RoaringTreemap;

impl RoaringTreemap {
    /// Unions in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (1..5).collect();
    ///
    /// rb1.union_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitOr` operator.
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (1..5).collect();
    ///
    /// let rb1 = rb1 | rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn union_with(&mut self, other: &RoaringTreemap) {
        for (key, other_rb) in &other.map {
            match self.map.entry(*key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb.clone());
                }
                Entry::Occupied(mut ent) => {
                    ent.get_mut().union_with(other_rb);
                }
            };
        }
    }

    /// Intersects in-place with the specified other bitmap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (3..4).collect();
    ///
    /// rb1.intersect_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitAnd` operator.
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (3..4).collect();
    ///
    /// let rb1 = rb1 & rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn intersect_with(&mut self, other: &RoaringTreemap) {
        let mut keys_to_remove: Vec<u32> = Vec::new();
        for (key, self_rb) in &mut self.map {
            match other.map.get(key) {
                None => {
                    keys_to_remove.push(*key);
                }
                Some(other_rb) => {
                    self_rb.intersect_with(other_rb);
                    if self_rb.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }

    /// Removes all values in the specified other bitmap from self, in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (1..3).collect();
    ///
    /// rb1.difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `Sub` operator.
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    /// let rb3: RoaringTreemap = (1..3).collect();
    ///
    /// let rb1 = rb1 - rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn difference_with(&mut self, other: &RoaringTreemap) {
        let mut keys_to_remove: Vec<u32> = Vec::new();
        for (key, self_rb) in &mut self.map {
            if let Some(other_rb) = other.map.get(key) {
                self_rb.difference_with(other_rb);
                if self_rb.is_empty() {
                    keys_to_remove.push(*key);
                }
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }

    /// Replaces this bitmap with one that is equivalent to `self XOR other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..6).collect();
    /// let rb3: RoaringTreemap = (1..3).chain(4..6).collect();
    ///
    /// rb1.symmetric_difference_with(&rb2);
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    ///
    /// Can also be done via the `BitXor` operator.
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..6).collect();
    /// let rb3: RoaringTreemap = (1..3).chain(4..6).collect();
    ///
    /// let rb1 = rb1 ^ rb2;
    ///
    /// assert_eq!(rb1, rb3);
    /// ```
    pub fn symmetric_difference_with(&mut self, other: &RoaringTreemap) {
        let mut keys_to_remove: Vec<u32> = Vec::new();
        for (key, other_rb) in &other.map {
            match self.map.entry(*key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb.clone());
                }
                Entry::Occupied(mut ent) => {
                    ent.get_mut().symmetric_difference_with(other_rb);
                    if ent.get().is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
            };
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }
}

impl BitOr<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        self.union_with(&rhs);
        self
    }
}

impl<'a> BitOr<&'a RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitor(mut self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.union_with(rhs);
        self
    }
}

impl<'a> BitOr<RoaringTreemap> for &'a RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        rhs | self
    }
}

impl<'a, 'b> BitOr<&'a RoaringTreemap> for &'b RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitor(self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.clone() | rhs
    }
}

impl BitOrAssign<RoaringTreemap> for RoaringTreemap {
    fn bitor_assign(&mut self, rhs: RoaringTreemap) {
        self.union_with(&rhs)
    }
}

impl<'a> BitOrAssign<&'a RoaringTreemap> for RoaringTreemap {
    fn bitor_assign(&mut self, rhs: &'a RoaringTreemap) {
        self.union_with(rhs)
    }
}

impl BitAnd<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitand(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        self.intersect_with(&rhs);
        self
    }
}

impl<'a> BitAnd<&'a RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitand(mut self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.intersect_with(rhs);
        self
    }
}

impl<'a> BitAnd<RoaringTreemap> for &'a RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitand(self, rhs: RoaringTreemap) -> RoaringTreemap {
        rhs & self
    }
}

impl<'a, 'b> BitAnd<&'a RoaringTreemap> for &'b RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitand(self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.clone() & rhs
    }
}

impl BitAndAssign<RoaringTreemap> for RoaringTreemap {
    fn bitand_assign(&mut self, rhs: RoaringTreemap) {
        self.intersect_with(&rhs)
    }
}

impl<'a> BitAndAssign<&'a RoaringTreemap> for RoaringTreemap {
    fn bitand_assign(&mut self, rhs: &'a RoaringTreemap) {
        self.intersect_with(rhs)
    }
}

impl Sub<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn sub(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        self.difference_with(&rhs);
        self
    }
}

impl<'a> Sub<&'a RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn sub(mut self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.difference_with(rhs);
        self
    }
}

impl<'a> Sub<RoaringTreemap> for &'a RoaringTreemap {
    type Output = RoaringTreemap;

    fn sub(self, rhs: RoaringTreemap) -> RoaringTreemap {
        self.clone() - rhs
    }
}

impl<'a, 'b> Sub<&'a RoaringTreemap> for &'b RoaringTreemap {
    type Output = RoaringTreemap;

    fn sub(self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.clone() - rhs
    }
}

impl SubAssign<RoaringTreemap> for RoaringTreemap {
    fn sub_assign(&mut self, rhs: RoaringTreemap) {
        self.difference_with(&rhs)
    }
}

impl<'a> SubAssign<&'a RoaringTreemap> for RoaringTreemap {
    fn sub_assign(&mut self, rhs: &'a RoaringTreemap) {
        self.difference_with(rhs)
    }
}

impl BitXor<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitxor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        self.symmetric_difference_with(&rhs);
        self
    }
}

impl<'a> BitXor<&'a RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitxor(mut self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.symmetric_difference_with(rhs);
        self
    }
}

impl<'a> BitXor<RoaringTreemap> for &'a RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitxor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        rhs ^ self
    }
}

impl<'a, 'b> BitXor<&'a RoaringTreemap> for &'b RoaringTreemap {
    type Output = RoaringTreemap;

    fn bitxor(self, rhs: &'a RoaringTreemap) -> RoaringTreemap {
        self.clone() ^ rhs
    }
}

impl BitXorAssign<RoaringTreemap> for RoaringTreemap {
    fn bitxor_assign(&mut self, rhs: RoaringTreemap) {
        self.symmetric_difference_with(&rhs)
    }
}

impl<'a> BitXorAssign<&'a RoaringTreemap> for RoaringTreemap {
    fn bitxor_assign(&mut self, rhs: &'a RoaringTreemap) {
        self.symmetric_difference_with(rhs)
    }
}
