use std::{
    borrow::Cow,
    convert::Infallible,
    mem,
    ops::{BitAndAssign, BitOrAssign, BitXorAssign, SubAssign},
};

use retain_mut::RetainMut;

use crate::{IterExt, RoaringBitmap};

use super::{container::Container, store::Store};

impl<I> IterExt<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    type Output = RoaringBitmap;

    fn or(self) -> Self::Output {
        try_naive_lazy_multi_op_owned(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            BitOrAssign::bitor_assign(a, b)
        })
        .unwrap()
    }

    fn and(self) -> Self::Output {
        try_multi_and_owned(self.into_iter().map(Ok::<_, Infallible>)).unwrap()
    }

    fn sub(self) -> Self::Output {
        try_simple_multi_op_owned(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            SubAssign::sub_assign(a, b)
        })
        .unwrap()
    }

    fn xor(self) -> Self::Output {
        try_naive_lazy_multi_op_owned(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            BitXorAssign::bitxor_assign(a, b)
        })
        .unwrap()
    }
}

impl<I, E> IterExt<Result<RoaringBitmap, E>> for I
where
    I: IntoIterator<Item = Result<RoaringBitmap, E>>,
{
    type Output = Result<RoaringBitmap, E>;

    fn or(self) -> Self::Output {
        try_naive_lazy_multi_op_owned(self, |a, b| BitOrAssign::bitor_assign(a, b))
    }

    fn and(self) -> Self::Output {
        try_multi_and_owned(self)
    }

    fn sub(self) -> Self::Output {
        try_simple_multi_op_owned(self, |a, b| SubAssign::sub_assign(a, b))
    }

    fn xor(self) -> Self::Output {
        try_naive_lazy_multi_op_owned(self, |a, b| BitXorAssign::bitxor_assign(a, b))
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
        try_simple_multi_op_ref(self.into_iter().map(Ok::<_, Infallible>), |a, b| {
            SubAssign::sub_assign(a, b)
        })
        .unwrap()
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
        try_simple_multi_op_ref(self, |a, b| SubAssign::sub_assign(a, b))
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
    let mut start = iter.by_ref().take(10).collect::<Result<Vec<_>, _>>()?;

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
    let mut start = iter.by_ref().take(10).collect::<Result<Vec<_>, _>>()?;

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
fn try_simple_multi_op_owned<E>(
    bitmaps: impl IntoIterator<Item = Result<RoaringBitmap, E>>,
    op: impl Fn(&mut RoaringBitmap, RoaringBitmap),
) -> Result<RoaringBitmap, E> {
    let mut iter = bitmaps.into_iter();
    match iter.next().transpose()? {
        Some(mut lhs) => {
            for rhs in iter {
                if lhs.is_empty() {
                    return Ok(lhs);
                }
                op(&mut lhs, rhs?);
            }
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
            for rhs in iter {
                if lhs.is_empty() {
                    return Ok(lhs);
                }
                op(&mut lhs, rhs?);
            }

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
