use std::{
    borrow::Cow,
    convert::Infallible,
    mem,
    ops::{BitOrAssign, BitXorAssign},
};

use retain_mut::RetainMut;

use crate::{IterExt, RoaringBitmap};

use super::{container::Container, store::Store};

/// When collecting bitmaps for optimizing the computation.If we don't know how many
// elements are in the iterator we collect 10 elements.
const BASE_COLLECT: usize = 10;

/// If an iterator contain 50 elements or less we collect everything because it'll be so
/// much faster without impacting the memory usage too much (in most cases).
const MAX_COLLECT: usize = 50;

impl<I> IterExt<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    type Output = RoaringBitmap;

    fn or(self) -> Self::Output {
        try_multi_or_owned(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn and(self) -> Self::Output {
        try_multi_and_owned(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn sub(self) -> Self::Output {
        try_multi_sub_owned(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn xor(self) -> Self::Output {
        try_multi_xor_owned(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }
}

impl<I, E> IterExt<Result<RoaringBitmap, E>> for I
where
    I: IntoIterator<Item = Result<RoaringBitmap, E>>,
{
    type Output = Result<RoaringBitmap, E>;

    fn or(self) -> Self::Output {
        try_multi_xor_owned(self)
    }

    fn and(self) -> Self::Output {
        try_multi_and_owned(self)
    }

    fn sub(self) -> Self::Output {
        try_multi_sub_owned(self)
    }

    fn xor(self) -> Self::Output {
        try_multi_xor_owned(self)
    }
}

impl<'a, I> IterExt<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    type Output = RoaringBitmap;

    fn or(self) -> Self::Output {
        try_naive_lazy_multi_op_ref(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            BitOrAssign::bitor_assign(a, b)
        })
        .unwrap()
    }

    fn and(self) -> Self::Output {
        try_multi_and_ref(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn sub(self) -> Self::Output {
        try_multi_sub_ref(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn xor(self) -> Self::Output {
        try_naive_lazy_multi_op_ref(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            BitXorAssign::bitxor_assign(a, b)
        })
        .unwrap()
    }
}

impl<'a, I, E: 'a> IterExt<Result<&'a RoaringBitmap, E>> for I
where
    I: IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
{
    type Output = Result<RoaringBitmap, E>;

    fn or(self) -> Self::Output {
        try_naive_lazy_multi_op_ref(self, |a, b| BitOrAssign::bitor_assign(a, b))
    }

    fn and(self) -> Self::Output {
        try_multi_and_ref(self)
    }

    fn sub(self) -> Self::Output {
        try_multi_sub_ref(self)
    }

    fn xor(self) -> Self::Output {
        try_naive_lazy_multi_op_ref(self, |a, b| BitXorAssign::bitxor_assign(a, b))
    }
}

#[inline]
fn try_multi_and_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    let mut start = collect_starting_elements::<_, Result<Vec<_>, _>>(iter.by_ref())?;

    if let Some((idx, _)) = start.iter().enumerate().min_by_key(|(_, b)| b.containers.len()) {
        let mut lhs = start.swap_remove(idx);
        for rhs in start {
            if lhs.is_empty() {
                return Ok(lhs);
            }
            lhs &= rhs;
        }

        for rhs in iter {
            if lhs.is_empty() {
                return Ok(lhs);
            }
            lhs &= rhs?;
        }
        Ok(lhs)
    } else {
        Ok(RoaringBitmap::new())
    }
}

#[inline]
fn try_multi_and_ref<'a, E>(
    bitmaps: impl IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    let mut start = collect_starting_elements::<_, Result<Vec<_>, _>>(iter.by_ref())?;

    if let Some((idx, _)) = start.iter().enumerate().min_by_key(|(_, b)| b.containers.len()) {
        let mut lhs = start.swap_remove(idx).clone();

        for rhs in start {
            if lhs.is_empty() {
                return Ok(lhs);
            }
            lhs &= rhs;
        }

        for rhs in iter {
            if lhs.is_empty() {
                return Ok(lhs);
            }
            lhs &= rhs?;
        }
        Ok(lhs)
    } else {
        Ok(RoaringBitmap::new())
    }
}

#[inline]
fn try_multi_sub_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    match iter.next().transpose()? {
        Some(mut lhs) => {
            for rhs in iter {
                if lhs.is_empty() {
                    return Ok(lhs);
                }
                lhs -= rhs?;
            }
            Ok(lhs)
        }
        None => Ok(RoaringBitmap::default()),
    }
}

#[inline]
fn try_multi_sub_ref<'a, E>(
    bitmaps: impl IntoIterator<Item = Result<&'a RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    match iter.next().transpose()?.cloned() {
        Some(mut lhs) => {
            for rhs in iter {
                if lhs.is_empty() {
                    return Ok(lhs);
                }
                lhs -= rhs?;
            }

            Ok(lhs)
        }
        None => Ok(RoaringBitmap::default()),
    }
}

#[inline]
fn try_multi_or_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    let mut start = collect_starting_elements::<_, Result<Vec<_>, _>>(iter.by_ref())?;

    let mut containers =
        if let Some((idx, _)) = start.iter().enumerate().max_by_key(|(_, b)| b.containers.len()) {
            let c = start.swap_remove(idx).containers.clone();
            if c.is_empty() {
                // everything must be empty if the max is empty
                start.clear();
            }
            c
        } else {
            return Ok(RoaringBitmap::new());
        };

    for bitmap in start {
        merge_containers(&mut containers, bitmap.containers, BitOrAssign::bitor_assign);
    }

    for bitmap in iter {
        merge_containers(&mut containers, bitmap?.containers, BitOrAssign::bitor_assign);
    }

    RetainMut::retain_mut(&mut containers, |container| {
        container.ensure_correct_store();
        container.len() > 0
    });

    Ok(RoaringBitmap { containers })
}

#[inline]
fn try_multi_xor_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    let mut containers = match iter.next().transpose()? {
        None => Vec::new(),
        Some(v) => v.containers,
    };

    for bitmap in iter {
        merge_containers(&mut containers, bitmap?.containers, BitXorAssign::bitxor_assign);
    }

    RetainMut::retain_mut(&mut containers, |container| {
        container.ensure_correct_store();
        container.len() > 0
    });

    Ok(RoaringBitmap { containers })
}

fn merge_containers(lhs: &mut Vec<Container>, rhs: Vec<Container>, op: impl Fn(&mut Store, Store)) {
    for mut rhs in rhs {
        match lhs.binary_search_by_key(&rhs.key, |c| c.key) {
            Err(loc) => lhs.insert(loc, rhs),
            Ok(loc) => {
                let lhs = &mut lhs[loc];
                match (&lhs.store, &rhs.store) {
                    (Store::Array(..), Store::Array(..)) => lhs.store = lhs.store.to_bitmap(),
                    (Store::Array(..), Store::Bitmap(..)) => mem::swap(lhs, &mut rhs),
                    _ => (),
                };
                op(&mut lhs.store, rhs.store);
            }
        }
    }
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

fn collect_starting_elements<I, O>(iter: impl IntoIterator<Item = I>) -> O
where
    O: FromIterator<I>,
{
    let mut iter = iter.into_iter();
    let mut to_collect = iter.size_hint().1.unwrap_or(BASE_COLLECT);
    if to_collect > MAX_COLLECT {
        to_collect = BASE_COLLECT;
    }

    iter.by_ref().take(to_collect).collect()
}
