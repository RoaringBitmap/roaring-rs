#![allow(clippy::suspicious_op_assign_impl)] // allow for +/- len in op assign

use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use retain_mut::RetainMut;

use crate::bitmap::container::Container;
use crate::bitmap::Pairs;
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
    /// rb1 |= rb2;
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
    #[deprecated(
        since = "0.6.7",
        note = "Please use the `BitOrAssign::bitor_assign` (`|=`) ops method instead"
    )]
    pub fn union_with(&mut self, other: &RoaringBitmap) {
        BitOrAssign::bitor_assign(self, other)
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
    /// rb1 &= rb2;
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
    #[deprecated(
        since = "0.6.7",
        note = "Please use the `BitAndAssign::bitand_assign` (`&=`) ops method instead"
    )]
    pub fn intersect_with(&mut self, other: &RoaringBitmap) {
        BitAndAssign::bitand_assign(self, other)
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
    /// rb1 -= rb2;
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
    #[deprecated(
        since = "0.6.7",
        note = "Please use the `SubAssign::sub_assign` (`-=`) ops method instead"
    )]
    pub fn difference_with(&mut self, other: &RoaringBitmap) {
        SubAssign::sub_assign(self, other)
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
    /// rb1 ^= rb2;
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
    #[deprecated(
        since = "0.6.7",
        note = "Please use the `BitXorAssign::bitxor_assign` (`^=`) ops method instead"
    )]
    pub fn symmetric_difference_with(&mut self, other: &RoaringBitmap) {
        BitXorAssign::bitxor_assign(self, other)
    }
}

impl BitOr<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `union` between two sets.
    fn bitor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitOr::bitor(rhs, self)
    }
}

impl BitOr<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `union` between two sets.
    fn bitor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        let mut containers = Vec::new();

        let mut len = 0;
        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => {
                    len += lhs.len();
                    containers.push(lhs.clone());
                }
                (None, Some(rhs)) => {
                    len += rhs.len();
                    containers.push(rhs.clone());
                }
                (Some(lhs), Some(rhs)) => {
                    let container = BitOr::bitor(lhs, rhs);
                    len += container.len();
                    containers.push(container);
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { len, containers }
    }
}

impl BitOrAssign<RoaringBitmap> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, mut rhs: RoaringBitmap) {
        // We make sure that we apply the union operation on the biggest map.
        if self.len() < rhs.len() {
            mem::swap(self, &mut rhs);
        }

        for rhs_cont in rhs.containers {
            let key = rhs_cont.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => {
                    self.len += rhs_cont.len();
                    self.containers.insert(loc, rhs_cont);
                }
                Ok(loc) => {
                    let lhs_cont = &mut self.containers[loc];
                    self.len -= lhs_cont.len();
                    BitOrAssign::bitor_assign(lhs_cont, rhs_cont);
                    self.len += lhs_cont.len();
                }
            }
        }
    }
}

impl BitOrAssign<&RoaringBitmap> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, rhs: &RoaringBitmap) {
        for rhs_cont in &rhs.containers {
            let key = rhs_cont.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => {
                    self.len += rhs_cont.len();
                    self.containers.insert(loc, rhs_cont.clone());
                }
                Ok(loc) => {
                    let lhs_cont = &mut self.containers[loc];
                    self.len -= lhs_cont.len();
                    BitOrAssign::bitor_assign(lhs_cont, rhs_cont);
                    self.len += lhs_cont.len();
                }
            }
        }
    }
}

impl BitAnd<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitAnd::bitand(rhs, self)
    }
}

impl BitAnd<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        let mut containers = Vec::new();

        let mut len = 0;
        for pair in Pairs::new(&self.containers, &rhs.containers) {
            if let (Some(lhs), Some(rhs)) = pair {
                let container = BitAnd::bitand(lhs, rhs);
                if container.len() != 0 {
                    len += container.len();
                    containers.push(container);
                }
            }
        }

        RoaringBitmap { len, containers }
    }
}

impl BitAndAssign<RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: RoaringBitmap) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.len() < self.len() {
            mem::swap(self, &mut rhs);
        }

        let mut len = self.len;
        RetainMut::retain_mut(&mut self.containers, |cont| {
            let key = cont.key;
            len -= cont.len();
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    let rhs_cont = &mut rhs.containers[loc];
                    let rhs_cont = mem::replace(rhs_cont, Container::new(rhs_cont.key));
                    BitAndAssign::bitand_assign(cont, rhs_cont);
                    len += cont.len();
                    cont.len() != 0
                }
                Err(_) => false,
            }
        });
        self.len = len;
    }
}

impl BitAndAssign<&RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &RoaringBitmap) {
        let mut len = self.len;
        RetainMut::retain_mut(&mut self.containers, |cont| {
            let key = cont.key;
            len -= cont.len();
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    BitAndAssign::bitand_assign(cont, &rhs.containers[loc]);
                    len += cont.len();
                    cont.len() != 0
                }
                Err(_) => false,
            }
        });
        self.len = len;
    }
}

impl Sub<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        SubAssign::sub_assign(&mut self, &rhs);
        self
    }
}

impl Sub<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        SubAssign::sub_assign(&mut self, rhs);
        self
    }
}

impl Sub<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `difference` between two sets.
    fn sub(self, rhs: RoaringBitmap) -> RoaringBitmap {
        Sub::sub(self, &rhs)
    }
}

impl Sub<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `difference` between two sets.
    fn sub(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        let mut containers = Vec::new();

        let mut len = 0;
        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => {
                    len += lhs.len();
                    containers.push(lhs.clone())
                }
                (None, Some(_)) => (),
                (Some(lhs), Some(rhs)) => {
                    let container = Sub::sub(lhs, rhs);
                    if container.len() != 0 {
                        len += container.len();
                        containers.push(container);
                    }
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { len, containers }
    }
}

impl SubAssign<RoaringBitmap> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: RoaringBitmap) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl SubAssign<&RoaringBitmap> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: &RoaringBitmap) {
        let mut len = self.len;
        RetainMut::retain_mut(&mut self.containers, |cont| {
            match rhs.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    len -= cont.len();
                    SubAssign::sub_assign(cont, &rhs.containers[loc]);
                    len += cont.len();
                    cont.len() != 0
                }
                Err(_) => true,
            }
        });
        self.len = len;
    }
}

impl BitXor<RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<&RoaringBitmap> for RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: &RoaringBitmap) -> RoaringBitmap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: RoaringBitmap) -> RoaringBitmap {
        BitXor::bitxor(rhs, self)
    }
}

impl BitXor<&RoaringBitmap> for &RoaringBitmap {
    type Output = RoaringBitmap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: &RoaringBitmap) -> RoaringBitmap {
        let mut containers = Vec::new();

        let mut len = 0;
        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => {
                    containers.push(lhs.clone());
                    len += lhs.len();
                }
                (None, Some(rhs)) => {
                    containers.push(rhs.clone());
                    len += rhs.len();
                }
                (Some(lhs), Some(rhs)) => {
                    let container = BitXor::bitxor(lhs, rhs);
                    if container.len() != 0 {
                        len += container.len();
                        containers.push(container);
                    }
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { len, containers }
    }
}

impl BitXorAssign<RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: RoaringBitmap) {
        for pair in Pairs::new(mem::take(&mut self.containers), rhs.containers) {
            match pair {
                (Some(mut lhs), Some(rhs)) => {
                    self.len -= lhs.len();
                    BitXorAssign::bitxor_assign(&mut lhs, rhs);
                    self.len += lhs.len();
                    if lhs.len() != 0 {
                        self.containers.push(lhs);
                    }
                }
                (Some(lhs), None) => {
                    self.containers.push(lhs);
                }
                (None, Some(rhs)) => {
                    self.len += rhs.len();
                    self.containers.push(rhs);
                }
                (None, None) => break,
            }
        }
    }
}

impl BitXorAssign<&RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &RoaringBitmap) {
        for pair in Pairs::new(mem::take(&mut self.containers), &rhs.containers) {
            match pair {
                (Some(mut lhs), Some(rhs)) => {
                    self.len -= lhs.len();
                    BitXorAssign::bitxor_assign(&mut lhs, rhs);
                    self.len += lhs.len();
                    if lhs.len() != 0 {
                        self.containers.push(lhs);
                    }
                }
                (Some(lhs), None) => self.containers.push(lhs),
                (None, Some(rhs)) => {
                    self.len += rhs.len();
                    self.containers.push(rhs.clone())
                }
                (None, None) => break,
            }
        }
    }
}
