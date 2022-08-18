use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::binary_heap::PeekMut;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BinaryHeap};
use std::mem;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use crate::{IterExt, RoaringBitmap, RoaringTreemap};

impl RoaringTreemap {
    /// Computes the len of the union with the specified other treemap without creating a new
    /// treemap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the union.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.union_len(&rb2), (rb1 | rb2).len());
    /// ```
    pub fn union_len(&self, other: &RoaringTreemap) -> u64 {
        self.len().wrapping_add(other.len()).wrapping_sub(self.intersection_len(other))
    }

    /// Computes the len of the intersection with the specified other treemap without creating a
    /// new treemap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.intersection_len(&rb2), (rb1 & rb2).len());
    /// ```
    pub fn intersection_len(&self, other: &RoaringTreemap) -> u64 {
        self.pairs(other)
            .map(|pair| match pair {
                (Some(..), None) => 0,
                (None, Some(..)) => 0,
                (Some(lhs), Some(rhs)) => lhs.intersection_len(rhs),
                (None, None) => 0,
            })
            .sum()
    }

    /// Computes the len of the difference with the specified other treemap without creating a new
    /// treemap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the difference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.difference_len(&rb2), (rb1 - rb2).len());
    /// ```
    pub fn difference_len(&self, other: &RoaringTreemap) -> u64 {
        self.len() - self.intersection_len(other)
    }

    /// Computes the len of the symmetric difference with the specified other treemap without
    /// creating a new bitmap.
    ///
    /// This is faster and more space efficient when you're only interested in the cardinality of
    /// the symmetric difference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let rb1: RoaringTreemap = (1..4).collect();
    /// let rb2: RoaringTreemap = (3..5).collect();
    ///
    ///
    /// assert_eq!(rb1.symmetric_difference_len(&rb2), (rb1 ^ rb2).len());
    /// ```
    pub fn symmetric_difference_len(&self, other: &RoaringTreemap) -> u64 {
        let intersection_len = self.intersection_len(other);

        self.len()
            .wrapping_add(other.len())
            .wrapping_sub(intersection_len)
            .wrapping_sub(intersection_len)
    }
}

impl BitOr<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitOrAssign::bitor_assign(&mut self, rhs);
        self
    }
}

impl BitOr<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitOr::bitor(rhs, self)
    }
}

impl BitOr<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `union` between two sets.
    fn bitor(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if self.len() <= rhs.len() {
            BitOr::bitor(rhs.clone(), self)
        } else {
            BitOr::bitor(self.clone(), rhs)
        }
    }
}

impl BitOrAssign<RoaringTreemap> for RoaringTreemap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, mut rhs: RoaringTreemap) {
        // We make sure that we apply the union operation on the biggest map.
        if self.len() < rhs.len() {
            mem::swap(self, &mut rhs);
        }

        for (key, other_rb) in rhs.map {
            match self.map.entry(key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb);
                }
                Entry::Occupied(mut ent) => {
                    BitOrAssign::bitor_assign(ent.get_mut(), other_rb);
                }
            }
        }
    }
}

impl BitOrAssign<&RoaringTreemap> for RoaringTreemap {
    /// An `union` between two sets.
    fn bitor_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, other_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(ent) => {
                    ent.insert(other_rb.clone());
                }
                Entry::Occupied(mut ent) => {
                    BitOrAssign::bitor_assign(ent.get_mut(), other_rb);
                }
            }
        }
    }
}

impl BitAnd<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitAndAssign::bitand_assign(&mut self, rhs);
        self
    }
}

impl BitAnd<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitAnd::bitand(rhs, self)
    }
}

impl BitAnd<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// An `intersection` between two sets.
    fn bitand(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if rhs.len() < self.len() {
            BitAnd::bitand(self.clone(), rhs)
        } else {
            BitAnd::bitand(rhs.clone(), self)
        }
    }
}

impl BitAndAssign<RoaringTreemap> for RoaringTreemap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, mut rhs: RoaringTreemap) {
        // We make sure that we apply the intersection operation on the smallest map.
        if rhs.len() < self.len() {
            mem::swap(self, &mut rhs);
        }

        BitAndAssign::bitand_assign(self, &rhs)
    }
}

impl BitAndAssign<&RoaringTreemap> for RoaringTreemap {
    /// An `intersection` between two sets.
    fn bitand_assign(&mut self, rhs: &RoaringTreemap) {
        let mut keys_to_remove: Vec<u32> = Vec::new();
        for (key, self_rb) in &mut self.map {
            match rhs.map.get(key) {
                Some(other_rb) => {
                    BitAndAssign::bitand_assign(self_rb, other_rb);
                    if self_rb.is_empty() {
                        keys_to_remove.push(*key);
                    }
                }
                None => keys_to_remove.push(*key),
            }
        }

        for key in keys_to_remove {
            self.map.remove(&key);
        }
    }
}

impl Sub<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        SubAssign::sub_assign(&mut self, rhs);
        self
    }
}

impl Sub<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        SubAssign::sub_assign(&mut self, rhs);
        self
    }
}

impl Sub<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(self, rhs: RoaringTreemap) -> RoaringTreemap {
        Sub::sub(self.clone(), rhs)
    }
}

impl Sub<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `difference` between two sets.
    fn sub(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        Sub::sub(self.clone(), rhs)
    }
}

impl SubAssign<RoaringTreemap> for RoaringTreemap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: RoaringTreemap) {
        SubAssign::sub_assign(self, &rhs)
    }
}

impl SubAssign<&RoaringTreemap> for RoaringTreemap {
    /// A `difference` between two sets.
    fn sub_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, rhs_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(_entry) => (),
                Entry::Occupied(mut entry) => {
                    SubAssign::sub_assign(entry.get_mut(), rhs_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}

impl BitXor<RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<&RoaringTreemap> for RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(mut self, rhs: &RoaringTreemap) -> RoaringTreemap {
        BitXorAssign::bitxor_assign(&mut self, rhs);
        self
    }
}

impl BitXor<RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: RoaringTreemap) -> RoaringTreemap {
        BitXor::bitxor(rhs, self)
    }
}

impl BitXor<&RoaringTreemap> for &RoaringTreemap {
    type Output = RoaringTreemap;

    /// A `symmetric difference` between two sets.
    fn bitxor(self, rhs: &RoaringTreemap) -> RoaringTreemap {
        if self.len() < rhs.len() {
            BitXor::bitxor(self, rhs.clone())
        } else {
            BitXor::bitxor(self.clone(), rhs)
        }
    }
}

impl BitXorAssign<RoaringTreemap> for RoaringTreemap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: RoaringTreemap) {
        for (key, other_rb) in rhs.map {
            match self.map.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(other_rb);
                }
                Entry::Occupied(mut entry) => {
                    BitXorAssign::bitxor_assign(entry.get_mut(), other_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}

impl BitXorAssign<&RoaringTreemap> for RoaringTreemap {
    /// A `symmetric difference` between two sets.
    fn bitxor_assign(&mut self, rhs: &RoaringTreemap) {
        for (key, other_rb) in &rhs.map {
            match self.map.entry(*key) {
                Entry::Vacant(entry) => {
                    entry.insert(other_rb.clone());
                }
                Entry::Occupied(mut entry) => {
                    BitXorAssign::bitxor_assign(entry.get_mut(), other_rb);
                    if entry.get().is_empty() {
                        entry.remove_entry();
                    }
                }
            }
        }
    }
}

impl<I> IterExt<RoaringTreemap> for I
where
    I: IntoIterator<Item = RoaringTreemap>,
{
    type Output = RoaringTreemap;

    fn or(self) -> Self::Output {
        try_simple_multi_op_owned::<_, _, OrOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn and(self) -> Self::Output {
        try_ordered_multi_op_owned::<_, _, AndOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn sub(self) -> Self::Output {
        try_ordered_multi_op_owned::<_, _, SubOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn xor(self) -> Self::Output {
        try_simple_multi_op_owned::<_, _, XorOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }
}

impl<I, E> IterExt<Result<RoaringTreemap, E>> for I
where
    I: IntoIterator<Item = Result<RoaringTreemap, E>>,
{
    type Output = Result<RoaringTreemap, E>;

    fn or(self) -> Self::Output {
        try_simple_multi_op_owned::<_, _, OrOp>(self)
    }

    fn and(self) -> Self::Output {
        try_ordered_multi_op_owned::<_, _, AndOp>(self)
    }

    fn sub(self) -> Self::Output {
        try_ordered_multi_op_owned::<_, _, SubOp>(self)
    }

    fn xor(self) -> Self::Output {
        try_simple_multi_op_owned::<_, _, XorOp>(self)
    }
}

#[inline]
fn try_simple_multi_op_owned<E, I, O: Op>(treemaps: I) -> Result<RoaringTreemap, E>
where
    I: IntoIterator<Item = Result<RoaringTreemap, E>>,
{
    let treemaps = treemaps.into_iter().collect::<Result<Vec<_>, _>>()?;

    let mut heap: BinaryHeap<_> = treemaps
        .into_iter()
        .filter_map(|treemap| {
            let mut iter = treemap.map.into_iter();
            iter.next().map(|(key, bitmap)| PeekedRoaringBitmap { key, bitmap, iter })
        })
        .collect();

    let mut bitmaps = Vec::new();
    let mut map = BTreeMap::new();

    while let Some(mut peek) = heap.peek_mut() {
        let (key, bitmap) = match peek.iter.next() {
            Some((next_key, next_bitmap)) => {
                let key = peek.key;
                peek.key = next_key;
                let bitmap = mem::replace(&mut peek.bitmap, next_bitmap);
                (key, bitmap)
            }
            None => {
                let poped = PeekMut::pop(peek);
                (poped.key, poped.bitmap)
            }
        };

        if let Some((first_key, _)) = bitmaps.first() {
            if *first_key != key {
                let current_key = *first_key;
                let computed_bitmap = O::op_owned(bitmaps.drain(..).map(|(_, rb)| rb));
                if !computed_bitmap.is_empty() {
                    map.insert(current_key, computed_bitmap);
                }
            }
        }

        bitmaps.push((key, bitmap));
    }

    if let Some((first_key, _)) = bitmaps.first() {
        let current_key = *first_key;
        let computed_bitmap = O::op_owned(bitmaps.drain(..).map(|(_, rb)| rb));
        if !computed_bitmap.is_empty() {
            map.insert(current_key, computed_bitmap);
        }
    }

    Ok(RoaringTreemap { map })
}

#[inline]
fn try_ordered_multi_op_owned<E, I, O: Op>(treemaps: I) -> Result<RoaringTreemap, E>
where
    I: IntoIterator<Item = Result<RoaringTreemap, E>>,
{
    let mut treemaps = treemaps.into_iter();
    let mut treemap = match treemaps.next().transpose()? {
        Some(treemap) => treemap,
        None => return Ok(RoaringTreemap::new()),
    };
    let mut treemaps = treemaps.collect::<Result<Vec<_>, _>>()?;

    // for each keys in the first treemap we're going find and accumulate all the corresponding bitmaps
    let keys: Vec<_> = treemap.map.keys().copied().collect();
    for k in keys {
        // the unwrap is safe since we're iterating on our keys
        let current_bitmap = treemap.map.remove(&k).unwrap();
        let new_bitmap =
            O::op_owned(std::iter::once(current_bitmap).chain(
                treemaps.iter_mut().map(|treemap| treemap.map.remove(&k).unwrap_or_default()),
            ));
        if !new_bitmap.is_empty() {
            treemap.map.insert(k, new_bitmap);
        }
    }

    Ok(treemap)
}

#[inline]
fn try_ordered_multi_op_ref<'a, E: 'a, I, O: Op>(treemaps: I) -> Result<RoaringTreemap, E>
where
    I: IntoIterator<Item = Result<&'a RoaringTreemap, E>>,
{
    let mut treemaps = treemaps.into_iter();
    let treemap = match treemaps.next().transpose()? {
        Some(treemap) => treemap,
        None => return Ok(RoaringTreemap::new()),
    };
    let treemaps = treemaps.collect::<Result<Vec<_>, _>>()?;

    let mut ret = RoaringTreemap::new();

    // for each keys in the first treemap we're going find and accumulate all the corresponding bitmaps
    let keys: Vec<_> = treemap.map.keys().copied().collect();
    let empty_bitmap = RoaringBitmap::new();
    for k in keys {
        // the unwrap is safe since we're iterating on our keys
        let current_bitmap = treemap.map.get(&k).unwrap();
        let new_bitmap = O::op_ref(
            std::iter::once(current_bitmap)
                .chain(treemaps.iter().map(|treemap| treemap.map.get(&k).unwrap_or(&empty_bitmap))),
        );
        if !new_bitmap.is_empty() {
            ret.map.insert(k, new_bitmap);
        }
    }

    Ok(ret)
}

#[inline]
fn try_simple_multi_op_ref<'a, E: 'a, I, O: Op>(treemaps: I) -> Result<RoaringTreemap, E>
where
    I: IntoIterator<Item = Result<&'a RoaringTreemap, E>>,
{
    let treemaps = treemaps.into_iter().collect::<Result<Vec<_>, E>>()?;

    let mut heap: BinaryHeap<_> = treemaps
        .into_iter()
        .filter_map(|treemap| {
            let mut iter = treemap.map.iter();
            iter.next().map(|(&key, bitmap)| PeekedRoaringBitmap { key, bitmap, iter })
        })
        .collect();

    let mut bitmaps = Vec::new();
    let mut map = BTreeMap::new();

    while let Some(mut peek) = heap.peek_mut() {
        let (key, bitmap) = match peek.iter.next() {
            Some((&next_key, next_bitmap)) => {
                let key = peek.key;
                peek.key = next_key;
                let bitmap = mem::replace(&mut peek.bitmap, next_bitmap);
                (key, bitmap)
            }
            None => {
                let poped = PeekMut::pop(peek);
                (poped.key, poped.bitmap)
            }
        };

        if let Some((first_key, _)) = bitmaps.first() {
            if *first_key != key {
                let current_key = *first_key;
                let computed_bitmap = O::op_ref(bitmaps.drain(..).map(|(_, rb)| rb));
                if !computed_bitmap.is_empty() {
                    map.insert(current_key, computed_bitmap);
                }
            }
        }

        bitmaps.push((key, bitmap));
    }

    if let Some((first_key, _)) = bitmaps.first() {
        let current_key = *first_key;
        let computed_bitmap = O::op_ref(bitmaps.drain(..).map(|(_, rb)| rb));
        if !computed_bitmap.is_empty() {
            map.insert(current_key, computed_bitmap);
        }
    }

    Ok(RoaringTreemap { map })
}

trait Op {
    fn op_owned<I: IntoIterator<Item = RoaringBitmap>>(iter: I) -> RoaringBitmap;
    fn op_ref<'a, I: IntoIterator<Item = &'a RoaringBitmap>>(iter: I) -> RoaringBitmap;
}

enum OrOp {}

impl Op for OrOp {
    fn op_owned<J: IntoIterator<Item = RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.or()
    }

    fn op_ref<'a, J: IntoIterator<Item = &'a RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.or()
    }
}

enum AndOp {}

impl Op for AndOp {
    fn op_owned<J: IntoIterator<Item = RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.and()
    }

    fn op_ref<'a, J: IntoIterator<Item = &'a RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.and()
    }
}

enum SubOp {}

impl Op for SubOp {
    fn op_owned<J: IntoIterator<Item = RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.sub()
    }

    fn op_ref<'a, J: IntoIterator<Item = &'a RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.sub()
    }
}

enum XorOp {}

impl Op for XorOp {
    fn op_owned<J: IntoIterator<Item = RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.xor()
    }

    fn op_ref<'a, J: IntoIterator<Item = &'a RoaringBitmap>>(iter: J) -> RoaringBitmap {
        iter.xor()
    }
}

impl<'a, I> IterExt<&'a RoaringTreemap> for I
where
    I: IntoIterator<Item = &'a RoaringTreemap>,
{
    type Output = RoaringTreemap;

    fn or(self) -> Self::Output {
        try_simple_multi_op_ref::<_, _, OrOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn and(self) -> Self::Output {
        try_ordered_multi_op_ref::<_, _, AndOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn sub(self) -> Self::Output {
        try_ordered_multi_op_ref::<_, _, SubOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }

    fn xor(self) -> Self::Output {
        try_simple_multi_op_ref::<_, _, XorOp>(
            self.into_iter().map(Ok::<_, std::convert::Infallible>),
        )
        .unwrap()
    }
}

impl<'a, I, E: 'a> IterExt<Result<&'a RoaringTreemap, E>> for I
where
    I: IntoIterator<Item = Result<&'a RoaringTreemap, E>>,
{
    type Output = Result<RoaringTreemap, E>;

    fn or(self) -> Self::Output {
        try_simple_multi_op_ref::<_, _, OrOp>(self)
    }

    fn and(self) -> Self::Output {
        try_ordered_multi_op_ref::<_, _, AndOp>(self)
    }

    fn sub(self) -> Self::Output {
        try_ordered_multi_op_ref::<_, _, SubOp>(self)
    }

    fn xor(self) -> Self::Output {
        try_simple_multi_op_ref::<_, _, XorOp>(self)
    }
}

struct PeekedRoaringBitmap<R, I> {
    key: u32,
    bitmap: R,
    iter: I,
}

impl<R: Borrow<RoaringBitmap>, I> Ord for PeekedRoaringBitmap<R, I> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key).reverse()
    }
}

impl<R: Borrow<RoaringBitmap>, I> PartialOrd for PeekedRoaringBitmap<R, I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<R: Borrow<RoaringBitmap>, I> Eq for PeekedRoaringBitmap<R, I> {}

impl<R: Borrow<RoaringBitmap>, I> PartialEq for PeekedRoaringBitmap<R, I> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

#[cfg(test)]
mod test {
    use crate::{IterExt, RoaringTreemap};
    use proptest::prelude::*;

    // fast count tests
    proptest! {
        #[test]
        fn union_len_eq_len_of_materialized_union(
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary()
        ) {
            prop_assert_eq!(a.union_len(&b), (a | b).len());
        }

        #[test]
        fn intersection_len_eq_len_of_materialized_intersection(
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary()
        ) {
            prop_assert_eq!(a.intersection_len(&b), (a & b).len());
        }

        #[test]
        fn difference_len_eq_len_of_materialized_difference(
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary()
        ) {
            prop_assert_eq!(a.difference_len(&b), (a - b).len());
        }

        #[test]
        fn symmetric_difference_len_eq_len_of_materialized_symmetric_difference(
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary()
        ) {
            prop_assert_eq!(a.symmetric_difference_len(&b), (a ^ b).len());
        }

        #[test]
        fn all_union_give_the_same_result(
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary(),
            c in RoaringTreemap::arbitrary()
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
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary(),
            c in RoaringTreemap::arbitrary()
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
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary(),
            c in RoaringTreemap::arbitrary()
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
            a in RoaringTreemap::arbitrary(),
            b in RoaringTreemap::arbitrary(),
            c in RoaringTreemap::arbitrary()
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
