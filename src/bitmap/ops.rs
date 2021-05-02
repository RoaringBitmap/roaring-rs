use std::cmp::Ordering;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};
use std::{cmp, mem};

use retain_mut::RetainMut;

use crate::bitmap::container::Container;
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
        let len = cmp::max(self.containers.len(), rhs.containers.len());
        let mut containers = Vec::with_capacity(len);
        let mut len = 0;

        let mut iter_lhs = self.containers.iter().peekable();
        let mut iter_rhs = rhs.containers.iter().peekable();

        loop {
            match (iter_lhs.peek(), iter_rhs.peek()) {
                (Some(lhs), Some(rhs)) => {
                    let container = match lhs.key.cmp(&rhs.key) {
                        Ordering::Less => iter_lhs.next().cloned().unwrap(),
                        Ordering::Greater => iter_rhs.next().cloned().unwrap(),
                        Ordering::Equal => {
                            let (lhs, rhs) = iter_lhs.next().zip(iter_rhs.next()).unwrap();
                            BitOr::bitor(lhs, rhs)
                        }
                    };
                    len += container.len;
                    containers.push(container);
                }
                (Some(_), None) => {
                    iter_lhs.by_ref().cloned().for_each(|container| {
                        len += container.len;
                        containers.push(container);
                    });
                }
                (None, Some(_)) => {
                    iter_rhs.by_ref().cloned().for_each(|container| {
                        len += container.len;
                        containers.push(container);
                    });
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { containers, len }
    }
}

impl BitOrAssign<RoaringBitmap> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, mut rhs: RoaringBitmap) {
        // We make sure that we apply the union operation on the biggest map.
        if self.len() < rhs.len() {
            mem::swap(self, &mut rhs);
        }

        for container in rhs.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => {
                    self.len += container.len;
                    self.containers.insert(loc, container);
                }
                Ok(loc) => {
                    let this_container = &mut self.containers[loc];
                    self.len -= this_container.len;
                    BitOrAssign::bitor_assign(this_container, container);
                    self.len += this_container.len;
                }
            }
        }
    }
}

impl BitOrAssign<&RoaringBitmap> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, rhs: &RoaringBitmap) {
        for container in &rhs.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => {
                    self.len += container.len;
                    self.containers.insert(loc, container.clone());
                }
                Ok(loc) => {
                    let this_container = &mut self.containers[loc];
                    self.len -= this_container.len;
                    BitOrAssign::bitor_assign(this_container, container);
                    self.len += this_container.len;
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

        let mut iter_lhs = self.containers.iter().peekable();
        let mut iter_rhs = rhs.containers.iter().peekable();

        loop {
            match (iter_lhs.peek(), iter_rhs.peek()) {
                (None, None) => break,
                (Some(lhs), Some(rhs)) => {
                    if lhs.key == rhs.key {
                        let (lhs, rhs) = iter_lhs.next().zip(iter_rhs.next()).unwrap();
                        let container = BitAnd::bitand(lhs, rhs);
                        if container.len != 0 {
                            len += container.len;
                            containers.push(container);
                        }
                    }
                }
                _otherwise => (),
            }
        }

        RoaringBitmap { containers, len }
    }
}

impl BitAndAssign<RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: RoaringBitmap) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.len() < self.len() {
            mem::swap(self, &mut rhs);
        }

        let mut removed = 0;
        self.containers.retain_mut(|cont| {
            let key = cont.key;
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    let rhs_cont = &mut rhs.containers[loc];
                    let rhs_cont = mem::replace(rhs_cont, Container::new(rhs_cont.key));
                    removed += cont.len;
                    BitAndAssign::bitand_assign(cont, rhs_cont);
                    if cont.len != 0 {
                        removed -= cont.len;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => {
                    removed += cont.len;
                    false
                }
            }
        });

        self.len -= removed;
    }
}

impl BitAndAssign<&RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &RoaringBitmap) {
        let mut removed = 0;
        self.containers.retain_mut(|cont| {
            let key = cont.key;
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    removed += cont.len;
                    BitAndAssign::bitand_assign(cont, &rhs.containers[loc]);
                    if cont.len != 0 {
                        removed -= cont.len;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => {
                    removed += cont.len;
                    false
                }
            }
        });

        self.len -= removed;
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

        let mut iter_lhs = self.containers.iter().peekable();
        let mut iter_rhs = rhs.containers.iter().peekable();

        loop {
            match (iter_lhs.peek(), iter_rhs.peek()) {
                (None, None) => break,
                (Some(_), None) => {
                    let container = iter_lhs.next().cloned().unwrap();
                    len += container.len;
                    containers.push(container);
                }
                (None, Some(_)) => {
                    iter_rhs.next().unwrap();
                }
                (Some(lhs), Some(rhs)) => match lhs.key.cmp(&rhs.key) {
                    Ordering::Less => {
                        let container = iter_lhs.next().cloned().unwrap();
                        len += container.len;
                        containers.push(container);
                    }
                    Ordering::Equal => {
                        let (lhs, rhs) = iter_lhs.next().zip(iter_rhs.next()).unwrap();
                        let container = Sub::sub(lhs, rhs);
                        if container.len != 0 {
                            len += container.len;
                            containers.push(container);
                        }
                    }
                    Ordering::Greater => {
                        iter_rhs.next().unwrap();
                    }
                },
            }
        }

        RoaringBitmap { containers, len }
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
        let mut removed = 0;
        self.containers.retain_mut(|cont| {
            match rhs.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    removed += cont.len;
                    SubAssign::sub_assign(cont, &rhs.containers[loc]);
                    if cont.len != 0 {
                        removed -= cont.len;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => true,
            }
        });

        self.len -= removed;
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

        let mut iter_lhs = self.containers.iter().peekable();
        let mut iter_rhs = rhs.containers.iter().peekable();

        loop {
            match (iter_lhs.peek(), iter_rhs.peek()) {
                (None, None) => break,
                (Some(_), None) => containers.extend(iter_lhs.by_ref().cloned()),
                (None, Some(_)) => containers.extend(iter_rhs.by_ref().cloned()),
                (Some(lhs), Some(rhs)) => {
                    let container = match lhs.key.cmp(&rhs.key) {
                        Ordering::Equal => {
                            let (lhs, rhs) = iter_lhs.next().zip(iter_rhs.next()).unwrap();
                            let container = BitXor::bitxor(lhs, rhs);
                            if container.len != 0 {
                                container
                            } else {
                                continue;
                            }
                        }
                        Ordering::Less => iter_lhs.next().cloned().unwrap(),
                        Ordering::Greater => iter_rhs.next().cloned().unwrap(),
                    };
                    len += container.len;
                    containers.push(container);
                }
            }
        }

        RoaringBitmap { containers, len }
    }
}

impl BitXorAssign<RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: RoaringBitmap) {
        let mut left = mem::take(&mut self.containers).into_iter().peekable();
        let mut right = rhs.containers.into_iter().peekable();

        self.len = 0;

        loop {
            match (left.peek(), right.peek()) {
                (None, None) => break,
                (Some(_), None) => {
                    self.containers.reserve(left.len());
                    left.for_each(|container| {
                        self.len += container.len;
                        self.containers.push(container);
                    });
                    break;
                }
                (None, Some(_)) => {
                    self.containers.reserve(right.len());
                    right.for_each(|container| {
                        self.len += container.len;
                        self.containers.push(container);
                    });
                    break;
                }
                (Some(l), Some(r)) => match l.key.cmp(&r.key) {
                    Ordering::Equal => {
                        let mut container = left.next().unwrap();
                        let rhs = right.next().unwrap();
                        BitXorAssign::bitxor_assign(&mut container, rhs);
                        if container.len != 0 {
                            self.len += container.len;
                            self.containers.push(container);
                        }
                    }
                    Ordering::Less => {
                        let container = left.next().unwrap();
                        self.len += container.len;
                        self.containers.push(container);
                    }
                    Ordering::Greater => {
                        let container = right.next().unwrap();
                        self.len += container.len;
                        self.containers.push(container);
                    }
                },
            }
        }
    }
}

impl BitXorAssign<&RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &RoaringBitmap) {
        let mut left = mem::take(&mut self.containers).into_iter().peekable();
        let mut right = rhs.containers.iter().peekable();

        self.len = 0;

        loop {
            match (left.peek(), right.peek()) {
                (None, None) => break,
                (Some(_), None) => {
                    self.containers.reserve(left.len());
                    left.for_each(|container| {
                        self.len += container.len;
                        self.containers.push(container);
                    });
                    break;
                }
                (None, Some(_)) => {
                    self.containers.reserve(right.len());
                    right.cloned().for_each(|container| {
                        self.len += container.len;
                        self.containers.push(container);
                    });
                    break;
                }
                (Some(l), Some(r)) => match l.key.cmp(&r.key) {
                    Ordering::Equal => {
                        let (mut container, rhs) = left.next().zip(right.next()).unwrap();
                        BitXorAssign::bitxor_assign(&mut container, rhs);
                        if container.len != 0 {
                            self.len += container.len;
                            self.containers.push(container);
                        }
                    }
                    Ordering::Less => {
                        let container = left.next().unwrap();
                        self.len += container.len;
                        self.containers.push(container);
                    }
                    Ordering::Greater => {
                        let container = right.next().unwrap();
                        self.len += container.len;
                        self.containers.push(container.clone());
                    }
                },
            }
        }
    }
}
