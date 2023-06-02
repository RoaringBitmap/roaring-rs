use std::{
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign},
};

use retain_mut::RetainMut;

use crate::{
    bitmap::{container::Container, Pairs},
    RoaringBitmap,
};

impl RoaringBitmap {
    /// Computes the len of the intersection with the specified other bitmap without creating a
    /// new bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.intersection_len(&rb2), (rb1 & rb2).len());
    /// ```
    pub fn intersection_len(&self, other: &Self) -> u64 {
        Pairs::new(&self.containers, &other.containers)
            .map(|pair| match pair {
                (Some(lhs), Some(rhs)) => lhs.intersection_len(rhs),
                _ => 0,
            })
            .sum()
    }

    /// Computes the len of the union with the specified other bitmap without creating a new bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the union.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.union_len(&rb2), (rb1 | rb2).len());
    /// ```
    pub fn union_len(&self, other: &Self) -> u64 {
        self.len().wrapping_add(other.len()).wrapping_sub(self.intersection_len(other))
    }

    /// Computes the len of the difference with the specified other bitmap without creating a new
    /// bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the difference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.difference_len(&rb2), (rb1 - rb2).len());
    /// ```
    pub fn difference_len(&self, other: &Self) -> u64 {
        self.len() - self.intersection_len(other)
    }

    /// Computes the len of the symmetric difference with the specified other bitmap without
    /// creating a new bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the symmetric difference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1: RoaringBitmap = (1..4).collect();
    /// let rb2: RoaringBitmap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.symmetric_difference_len(&rb2), (rb1 ^ rb2).len());
    /// ```
    pub fn symmetric_difference_len(&self, other: &Self) -> u64 {
        let intersection_len = self.intersection_len(other);
        self.len()
            .wrapping_add(other.len())
            .wrapping_sub(intersection_len)
            .wrapping_sub(intersection_len)
    }
}

impl BitOr<Self> for RoaringBitmap {
    type Output = Self;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: Self) -> Self {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<&Self> for RoaringBitmap {
    type Output = Self;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: &Self) -> Self {
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
        let mut containers = vec![];

        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), Some(rhs)) => containers.push(BitOr::bitor(lhs, rhs)),
                (Some(lhs), None) => containers.push(lhs.clone()),
                (None, Some(rhs)) => containers.push(rhs.clone()),
                (None, None) => break,
            }
        }

        RoaringBitmap { containers }
    }
}

impl BitOrAssign<Self> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, mut rhs: Self) {
        // We make sure that we apply the union operation on the biggest map.
        if self.len() < rhs.len() {
            mem::swap(self, &mut rhs);
        }

        for container in rhs.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container),
                Ok(loc) => BitOrAssign::bitor_assign(&mut self.containers[loc], container),
            }
        }
    }
}

impl BitOrAssign<&Self> for RoaringBitmap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, rhs: &Self) {
        for container in &rhs.containers {
            let key = container.key;
            match self.containers.binary_search_by_key(&key, |c| c.key) {
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => BitOrAssign::bitor_assign(&mut self.containers[loc], container),
            }
        }
    }
}

impl BitAnd<Self> for RoaringBitmap {
    type Output = Self;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: Self) -> Self {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<&Self> for RoaringBitmap {
    type Output = Self;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: &Self) -> Self {
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
        let mut containers = vec![];

        for pair in Pairs::new(&self.containers, &rhs.containers) {
            if let (Some(lhs), Some(rhs)) = pair {
                let container = BitAnd::bitand(lhs, rhs);
                if container.len() != 0 {
                    containers.push(container);
                }
            }
        }

        RoaringBitmap { containers }
    }
}

impl BitAndAssign<Self> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: Self) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.containers.len() < self.containers.len() {
            mem::swap(self, &mut rhs);
        }

        RetainMut::retain_mut(&mut self.containers, |cont| {
            let key = cont.key;
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    let rhs_cont = &mut rhs.containers[loc];
                    let rhs_cont = mem::replace(rhs_cont, Container::new(rhs_cont.key));
                    BitAndAssign::bitand_assign(cont, rhs_cont);
                    cont.len() != 0
                }
                Err(_) => false,
            }
        })
    }
}

impl BitAndAssign<&Self> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &Self) {
        RetainMut::retain_mut(&mut self.containers, |cont| {
            let key = cont.key;
            match rhs.containers.binary_search_by_key(&key, |c| c.key) {
                Ok(loc) => {
                    BitAndAssign::bitand_assign(cont, &rhs.containers[loc]);
                    cont.len() != 0
                }
                Err(_) => false,
            }
        })
    }
}

impl Sub<Self> for RoaringBitmap {
    type Output = Self;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: Self) -> Self {
        SubAssign::sub_assign(&mut self, &rhs);
        self
    }
}

impl Sub<&Self> for RoaringBitmap {
    type Output = Self;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: &Self) -> Self {
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
        let mut containers = vec![];

        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => containers.push(lhs.clone()),
                (None, Some(_)) => (),
                (Some(lhs), Some(rhs)) => {
                    let container = Sub::sub(lhs, rhs);
                    if container.len() != 0 {
                        containers.push(container);
                    }
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { containers }
    }
}

impl SubAssign<Self> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: Self) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl SubAssign<&Self> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: &Self) {
        RetainMut::retain_mut(&mut self.containers, |cont| {
            match rhs.containers.binary_search_by_key(&cont.key, |c| c.key) {
                Ok(loc) => {
                    SubAssign::sub_assign(cont, &rhs.containers[loc]);
                    cont.len() != 0
                }
                Err(_) => true,
            }
        })
    }
}

impl BitXor<Self> for RoaringBitmap {
    type Output = Self;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: Self) -> Self {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<&Self> for RoaringBitmap {
    type Output = Self;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: &Self) -> Self {
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
        let mut containers = vec![];

        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => containers.push(lhs.clone()),
                (None, Some(rhs)) => containers.push(rhs.clone()),
                (Some(lhs), Some(rhs)) => {
                    let container = BitXor::bitxor(lhs, rhs);
                    if container.len() != 0 {
                        containers.push(container);
                    }
                }
                (None, None) => break,
            }
        }

        RoaringBitmap { containers }
    }
}

impl BitXorAssign<Self> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: Self) {
        for pair in Pairs::new(mem::take(&mut self.containers), rhs.containers) {
            match pair {
                (Some(mut lhs), Some(rhs)) => {
                    BitXorAssign::bitxor_assign(&mut lhs, rhs);
                    if lhs.len() != 0 {
                        self.containers.push(lhs);
                    }
                }
                (Some(lhs), None) => self.containers.push(lhs),
                (None, Some(rhs)) => self.containers.push(rhs),
                (None, None) => break,
            }
        }
    }
}

impl BitXorAssign<&Self> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &Self) {
        for pair in Pairs::new(mem::take(&mut self.containers), &rhs.containers) {
            match pair {
                (Some(mut lhs), Some(rhs)) => {
                    BitXorAssign::bitxor_assign(&mut lhs, rhs);
                    if lhs.len() != 0 {
                        self.containers.push(lhs);
                    }
                }
                (Some(lhs), None) => self.containers.push(lhs),
                (None, Some(rhs)) => self.containers.push(rhs.clone()),
                (None, None) => break,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{MultiOps, RoaringBitmap};
    use proptest::prelude::*;
    use std::convert::Infallible;

    // fast count tests
    proptest! {
        #[test]
        fn union_len_eq_len_of_materialized_union(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(a.union_len(&b), (a | b).len());
        }

        #[test]
        fn intersection_len_eq_len_of_materialized_intersection(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(a.intersection_len(&b), (a & b).len());
        }

        #[test]
        fn difference_len_eq_len_of_materialized_difference(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(a.difference_len(&b), (a - b).len());
        }

        #[test]
        fn symmetric_difference_len_eq_len_of_materialized_symmetric_difference(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary()
        ) {
            prop_assert_eq!(a.symmetric_difference_len(&b), (a ^ b).len());
        }

        #[test]
        fn all_union_give_the_same_result(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let mut ref_assign = a.clone();
            ref_assign |= &b;
            ref_assign |= &c;

            let mut own_assign = a.clone();
            own_assign |= b.clone();
            own_assign |= c.clone();

            let ref_inline = &a | &b | &c;
            let own_inline = a.clone() | b.clone() | c.clone();

            let ref_multiop = [&a, &b, &c].union();
            let own_multiop = [a.clone(), b.clone(), c.clone()].union();

            let ref_multiop_try = [&a, &b, &c].map(Ok::<_, Infallible>).union().unwrap();
            let own_multiop_try = [a, b, c].map(Ok::<_, Infallible>).union().unwrap();

            for roar in &[
                own_assign,
                ref_inline,
                own_inline,
                ref_multiop,
                own_multiop,
                ref_multiop_try,
                own_multiop_try,
            ] {
                prop_assert_eq!(&ref_assign, roar);
            }
        }

        #[test]
        fn all_intersection_give_the_same_result(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let mut ref_assign = a.clone();
            ref_assign &= &b;
            ref_assign &= &c;

            let mut own_assign = a.clone();
            own_assign &= b.clone();
            own_assign &= c.clone();

            let ref_inline = &a & &b & &c;
            let own_inline = a.clone() & b.clone() & c.clone();

            let ref_multiop = [&a, &b, &c].intersection();
            let own_multiop = [a.clone(), b.clone(), c.clone()].intersection();

            let ref_multiop_try = [&a, &b, &c].map(Ok::<_, Infallible>).intersection().unwrap();
            let own_multiop_try = [a, b, c].map(Ok::<_, Infallible>).intersection().unwrap();

            for roar in &[
                own_assign,
                ref_inline,
                own_inline,
                ref_multiop,
                own_multiop,
                ref_multiop_try,
                own_multiop_try,
            ] {
                prop_assert_eq!(&ref_assign, roar);
            }
        }

        #[test]
        fn all_difference_give_the_same_result(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let mut ref_assign = a.clone();
            ref_assign -= &b;
            ref_assign -= &c;

            let mut own_assign = a.clone();
            own_assign -= b.clone();
            own_assign -= c.clone();

            let ref_inline = &a - &b - &c;
            let own_inline = a.clone() - b.clone() - c.clone();

            let ref_multiop = [&a, &b, &c].difference();
            let own_multiop = [a.clone(), b.clone(), c.clone()].difference();

            let ref_multiop_try = [&a, &b, &c].map(Ok::<_, Infallible>).difference().unwrap();
            let own_multiop_try = [a, b, c].map(Ok::<_, Infallible>).difference().unwrap();

            for roar in &[
                own_assign,
                ref_inline,
                own_inline,
                ref_multiop,
                own_multiop,
                ref_multiop_try,
                own_multiop_try,
            ] {
                prop_assert_eq!(&ref_assign, roar);
            }
        }

        #[test]
        fn all_symmetric_difference_give_the_same_result(
            a in RoaringBitmap::arbitrary(),
            b in RoaringBitmap::arbitrary(),
            c in RoaringBitmap::arbitrary()
        ) {
            let mut ref_assign = a.clone();
            ref_assign ^= &b;
            ref_assign ^= &c;

            let mut own_assign = a.clone();
            own_assign ^= b.clone();
            own_assign ^= c.clone();

            let ref_inline = &a ^ &b ^ &c;
            let own_inline = a.clone() ^ b.clone() ^ c.clone();

            let ref_multiop = [&a, &b, &c].symmetric_difference();
            let own_multiop = [a.clone(), b.clone(), c.clone()].symmetric_difference();

            let ref_multiop_try = [&a, &b, &c]
                .map(Ok::<_, Infallible>)
                .symmetric_difference()
                .unwrap();
            let own_multiop_try = [a, b, c]
                .map(Ok::<_, Infallible>)
                .symmetric_difference()
                .unwrap();

            for roar in &[
                own_assign,
                ref_inline,
                own_inline,
                ref_multiop,
                own_multiop,
                ref_multiop_try,
                own_multiop_try,
            ] {
                prop_assert_eq!(&ref_assign, roar);
            }
        }
    }
}
