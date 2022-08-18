use std::borrow::Cow;
use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use retain_mut::RetainMut;

use crate::bitmap::container::Container;
use crate::bitmap::store::Store;
use crate::bitmap::Pairs;
use crate::{IterExt, RoaringBitmap};

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
    pub fn intersection_len(&self, other: &RoaringBitmap) -> u64 {
        Pairs::new(&self.containers, &other.containers)
            .map(|pair| match pair {
                (Some(..), None) => 0,
                (None, Some(..)) => 0,
                (Some(lhs), Some(rhs)) => lhs.intersection_len(rhs),
                (None, None) => 0,
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
    pub fn union_len(&self, other: &RoaringBitmap) -> u64 {
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
    pub fn difference_len(&self, other: &RoaringBitmap) -> u64 {
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
    pub fn symmetric_difference_len(&self, other: &RoaringBitmap) -> u64 {
        let intersection_len = self.intersection_len(other);
        self.len()
            .wrapping_add(other.len())
            .wrapping_sub(intersection_len)
            .wrapping_sub(intersection_len)
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

        for pair in Pairs::new(&self.containers, &rhs.containers) {
            match pair {
                (Some(lhs), None) => containers.push(lhs.clone()),
                (None, Some(rhs)) => containers.push(rhs.clone()),
                (Some(lhs), Some(rhs)) => containers.push(BitOr::bitor(lhs, rhs)),
                (None, None) => break,
            }
        }

        RoaringBitmap { containers }
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
                Err(loc) => self.containers.insert(loc, container),
                Ok(loc) => BitOrAssign::bitor_assign(&mut self.containers[loc], container),
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
                Err(loc) => self.containers.insert(loc, container.clone()),
                Ok(loc) => BitOrAssign::bitor_assign(&mut self.containers[loc], container),
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

impl BitAndAssign<RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: RoaringBitmap) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.len() < self.len() {
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

impl BitAndAssign<&RoaringBitmap> for RoaringBitmap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &RoaringBitmap) {
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

impl SubAssign<RoaringBitmap> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: RoaringBitmap) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl SubAssign<&RoaringBitmap> for RoaringBitmap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: &RoaringBitmap) {
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

impl BitXorAssign<RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: RoaringBitmap) {
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

impl BitXorAssign<&RoaringBitmap> for RoaringBitmap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &RoaringBitmap) {
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

impl<I> IterExt<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    type Bitmap = RoaringBitmap;

    fn or(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_owned(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| BitOrAssign::bitor_assign(a, b),
        )
        .unwrap()
    }

    fn and(self) -> Self::Bitmap {
        try_simple_multi_op_owned(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| BitAndAssign::bitand_assign(a, b),
        )
        .unwrap()
    }

    fn sub(self) -> Self::Bitmap {
        try_simple_multi_op_owned(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| SubAssign::sub_assign(a, b),
        )
        .unwrap()
    }

    fn xor(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_owned(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| BitXorAssign::bitxor_assign(a, b),
        )
        .unwrap()
    }
}

impl<I, E> IterExt<Result<RoaringBitmap, E>> for I
where
    I: IntoIterator<Item = Result<RoaringBitmap, E>>,
{
    type Bitmap = Result<RoaringBitmap, E>;

    fn or(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_owned(self, |a, b| BitOrAssign::bitor_assign(a, b))
    }

    fn and(self) -> Self::Bitmap {
        try_simple_multi_op_owned(self, |a, b| BitAndAssign::bitand_assign(a, b))
    }

    fn sub(self) -> Self::Bitmap {
        try_simple_multi_op_owned(self, |a, b| SubAssign::sub_assign(a, b))
    }

    fn xor(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_owned(self, |a, b| BitXorAssign::bitxor_assign(a, b))
    }
}

impl<'a, I> IterExt<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    type Bitmap = RoaringBitmap;

    fn or(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_ref(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| BitOrAssign::bitor_assign(a, b),
        )
        .unwrap()
    }

    fn and(self) -> Self::Bitmap {
        try_simple_multi_op_ref(self.into_iter().map(Ok::<_, std::convert::Infallible>), |a, b| {
            BitAndAssign::bitand_assign(a, b)
        })
        .unwrap()
    }

    fn sub(self) -> Self::Bitmap {
        try_simple_multi_op_ref(self.into_iter().map(Ok::<_, std::convert::Infallible>), |a, b| {
            SubAssign::sub_assign(a, b)
        })
        .unwrap()
    }

    fn xor(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_ref(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
            |a, b| BitXorAssign::bitxor_assign(a, b),
        )
        .unwrap()
    }
}

impl<'a, I, E: 'a> IterExt<Result<&'a RoaringBitmap, E>> for I
where
    I: IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
{
    type Bitmap = Result<RoaringBitmap, E>;

    fn or(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_ref(self, |a, b| BitOrAssign::bitor_assign(a, b))
    }

    fn and(self) -> Self::Bitmap {
        try_simple_multi_op_ref(self, |a, b| BitAndAssign::bitand_assign(a, b))
    }

    fn sub(self) -> Self::Bitmap {
        try_simple_multi_op_ref(self, |a, b| SubAssign::sub_assign(a, b))
    }

    fn xor(self) -> Self::Bitmap {
        try_naive_lazy_multi_op_ref(self, |a, b| BitXorAssign::bitxor_assign(a, b))
    }
}

#[inline]
fn try_simple_multi_op_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
    op: impl Fn(&mut RoaringBitmap, RoaringBitmap),
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    match iter.next().transpose()? {
        Some(mut lhs) => {
            iter.try_for_each(|rhs| rhs.map(|rhs| op(&mut lhs, rhs)))?;
            Ok(lhs)
        }
        None => Ok(RoaringBitmap::default()),
    }
}

#[inline]
fn try_simple_multi_op_ref<'a, E>(
    bitmaps: impl IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
    op: impl Fn(&mut RoaringBitmap, &RoaringBitmap),
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    match iter.next().transpose()?.cloned() {
        Some(mut lhs) => {
            iter.try_for_each(|rhs| rhs.map(|rhs| op(&mut lhs, rhs)))?;
            Ok(lhs)
        }
        None => Ok(RoaringBitmap::default()),
    }
}

#[inline]
fn try_naive_lazy_multi_op_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
    op: impl Fn(&mut Store, &Store),
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    let mut containers = match iter.next().transpose()? {
        None => Vec::new(),
        Some(v) => v.containers,
    };

    for bitmap in iter {
        for mut rhs in bitmap?.containers {
            match containers.binary_search_by_key(&rhs.key, |c| c.key) {
                Err(loc) => containers.insert(loc, rhs),
                Ok(loc) => {
                    let lhs = &mut containers[loc];
                    match (&lhs.store, &rhs.store) {
                        (Store::Array(..), Store::Array(..)) => lhs.store = lhs.store.to_bitmap(),
                        (Store::Array(..), Store::Bitmap(..)) => mem::swap(lhs, &mut rhs),
                        _ => (),
                    };
                    op(&mut lhs.store, &rhs.store);
                }
            }
        }
    }

    RetainMut::retain_mut(&mut containers, |container| {
        container.ensure_correct_store();
        container.len() > 0
    });

    Ok(RoaringBitmap { containers })
}

#[inline]
fn try_naive_lazy_multi_op_ref<'a, E: 'a>(
    bitmaps: impl IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
    op: impl Fn(&mut Store, &Store),
) -> Result<RoaringBitmap, E> {
    //
    // This algorithm operates on bitmaps. It must deal with arrays for which there are not (yet)
    // any others with the same key.
    //
    //   1. Eager cloning would create useless intermediate values that might become bitmaps
    //   2. Eager promoting forces disjoint containers to converted back to arrays at the end
    //
    // This strategy uses COW to lazily promote arrays to bitmaps as they are operated on.
    // More memory efficient, negligible wall time difference benchmarks

    // Phase 1. Borrow all the containers from the first element.
    let mut iter = bitmaps.into_iter();
    let mut containers: Vec<Cow<Container>> = match iter.next().transpose()? {
        None => Vec::new(),
        Some(v) => v.containers.iter().map(Cow::Borrowed).collect(),
    };

    // Phase 2: Operate on the remaining contaners
    for bitmap in iter {
        for rhs in &bitmap?.containers {
            match containers.binary_search_by_key(&rhs.key, |c| c.key) {
                Err(loc) => {
                    // A container not currently in containers. Borrow it.
                    containers.insert(loc, Cow::Borrowed(rhs))
                }
                Ok(loc) => {
                    // A container that is in containers. Operate on it.
                    let lhs = &mut containers[loc];
                    match (&lhs.store, &rhs.store) {
                        (Store::Array(..), Store::Array(..)) => {
                            // We had borrowed an array. Without cloning it, create a new bitmap
                            // Add all the elements to the new bitmap
                            let mut store = lhs.store.to_bitmap();
                            op(&mut store, &rhs.store);
                            *lhs = Cow::Owned(Container { key: lhs.key, store });
                        }
                        (Store::Array(..), Store::Bitmap(..)) => {
                            // We had borrowed an array. Copy the rhs bitmap, add lhs to it
                            let mut store = rhs.store.clone();
                            op(&mut store, &lhs.store);
                            *lhs = Cow::Owned(Container { key: lhs.key, store });
                        }
                        (Store::Bitmap(..), _) => {
                            // This might be a owned or borrowed bitmap.
                            // If it was borrowed it will clone-on-write
                            op(&mut lhs.to_mut().store, &rhs.store);
                        }
                    };
                }
            }
        }
    }

    // Phase 3: Clean up
    let containers: Vec<Container> = containers
        .into_iter()
        .map(|c| {
            // Any borrowed bitmaps or arrays left over get cloned here
            let mut container = c.into_owned();
            container.ensure_correct_store();
            container
        })
        .filter(|container| container.len() > 0)
        .collect();

    Ok(RoaringBitmap { containers })
}

#[cfg(test)]
mod test {
    use crate::{IterExt, RoaringBitmap};
    use proptest::prelude::*;

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

            let ref_multiop = [&a, &b, &c].or();
            let own_multiop = [a.clone(), b.clone(), c.clone()].or();

            for roar in &[own_assign, ref_inline, own_inline, ref_multiop, own_multiop] {
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

            let ref_multiop = [&a, &b, &c].and();
            let own_multiop = [a.clone(), b.clone(), c.clone()].and();

            for roar in &[own_assign, ref_inline, own_inline, ref_multiop, own_multiop] {
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

            let ref_multiop = [&a, &b, &c].sub();
            let own_multiop = [a.clone(), b.clone(), c.clone()].sub();

            for roar in &[own_assign, ref_inline, own_inline, ref_multiop, own_multiop] {
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

            let ref_multiop = [&a, &b, &c].xor();
            let own_multiop = [a.clone(), b.clone(), c.clone()].xor();

            for roar in &[own_assign, ref_inline, own_inline, ref_multiop, own_multiop] {
                prop_assert_eq!(&ref_assign, roar);
            }
        }
    }
}
