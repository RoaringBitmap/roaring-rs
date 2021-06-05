use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::binary_heap::{BinaryHeap, PeekMut};
use std::mem;

use crate::bitmap::container::Container;
use crate::bitmap::store::Store;
use crate::RoaringBitmap;

struct PeekedContainer<C, I> {
    container: C,
    iter: I,
}

impl<C: Borrow<Container>, I> Ord for PeekedContainer<C, I> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.container.borrow().key.cmp(&other.container.borrow().key).reverse()
    }
}

impl<C: Borrow<Container>, I> PartialOrd for PeekedContainer<C, I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Borrow<Container>, I> Eq for PeekedContainer<C, I> {}

impl<C: Borrow<Container>, I> PartialEq for PeekedContainer<C, I> {
    fn eq(&self, other: &Self) -> bool {
        self.container.borrow().key == other.container.borrow().key
    }
}

pub trait MultiBitOr<Rbs>: IntoIterator<Item = Rbs> {
    fn bitor(self) -> RoaringBitmap;
}

impl<'a, I> MultiBitOr<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    fn bitor(self) -> RoaringBitmap {
        let iter = self.into_iter();
        let mut heap = BinaryHeap::with_capacity(iter.size_hint().0);

        for rb in iter {
            let mut iter = rb.containers.iter();
            if let Some(container) = iter.next() {
                heap.push(PeekedContainer { container, iter });
            }
        }

        let mut containers = Vec::new();
        let mut current = None;

        while let Some(mut peek) = heap.peek_mut() {
            let pkey = peek.container.key;
            let container = match peek.iter.next() {
                Some(next) => mem::replace(&mut peek.container, next),
                None => PeekMut::pop(peek).container,
            };

            match current.as_mut() {
                Some((ckey, cstore)) => {
                    if *ckey == pkey {
                        *cstore |= &container.store;
                    } else {
                        let key = mem::replace(ckey, container.key);
                        let store = mem::replace(cstore, container.store.to_bitmap());

                        let mut container = Container { key, len: store.len(), store };
                        container.ensure_correct_store();
                        containers.push(container);
                    }
                }
                None => current = Some((container.key, container.store.to_bitmap())),
            }
        }

        if let Some((key, store)) = current {
            let mut container = Container { key, len: store.len(), store };
            container.ensure_correct_store();
            containers.push(container);
        }

        RoaringBitmap { containers }
    }
}

impl<I> MultiBitOr<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    fn bitor(self) -> RoaringBitmap {
        fn into_bitmap(store: Store) -> Store {
            match store {
                Store::Bitmap(_) => store,
                Store::Array(_) => store.to_bitmap(),
            }
        }

        let iter = self.into_iter();
        let mut heap = BinaryHeap::with_capacity(iter.size_hint().0);

        for rb in iter {
            let mut iter = rb.containers.into_iter();
            if let Some(container) = iter.next() {
                heap.push(PeekedContainer { container, iter });
            }
        }

        let mut containers = Vec::new();
        let mut current = None;

        while let Some(mut peek) = heap.peek_mut() {
            let pkey = peek.container.key;
            let container = match peek.iter.next() {
                Some(next) => mem::replace(&mut peek.container, next),
                None => PeekMut::pop(peek).container,
            };

            match current.as_mut() {
                Some((ckey, cstore)) => {
                    if *ckey == pkey {
                        *cstore |= &container.store;
                    } else {
                        let key = mem::replace(ckey, container.key);
                        let store = mem::replace(cstore, into_bitmap(container.store));

                        let mut container = Container { key, len: store.len(), store };
                        container.ensure_correct_store();
                        containers.push(container);
                    }
                }
                None => current = Some((container.key, into_bitmap(container.store))),
            }
        }

        if let Some((key, store)) = current {
            let mut container = Container { key, len: store.len(), store };
            container.ensure_correct_store();
            containers.push(container);
        }

        RoaringBitmap { containers }
    }
}

pub trait MultiBitAnd<Rbs>: IntoIterator<Item = Rbs> {
    fn bitand(self) -> RoaringBitmap;
}

impl<'a, I> MultiBitAnd<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    fn bitand(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next().cloned() {
            Some(mut first) => {
                for rb in iter {
                    if first.is_empty() {
                        break;
                    }
                    first &= rb;
                }
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}

impl<I> MultiBitAnd<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    fn bitand(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next() {
            Some(mut first) => {
                for rb in iter {
                    if first.is_empty() {
                        break;
                    }
                    first &= rb;
                }
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}

pub trait MultiBitXor<Rbs>: IntoIterator<Item = Rbs> {
    fn bitxor(self) -> RoaringBitmap;
}

impl<'a, I> MultiBitXor<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    fn bitxor(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next().cloned() {
            Some(mut first) => {
                iter.for_each(|rb| first ^= rb);
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}

impl<I> MultiBitXor<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    fn bitxor(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next() {
            Some(mut first) => {
                iter.for_each(|rb| first ^= rb);
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}

pub trait MultiSub<Rbs>: IntoIterator<Item = Rbs> {
    fn sub(self) -> RoaringBitmap;
}

impl<'a, I> MultiSub<&'a RoaringBitmap> for I
where
    I: IntoIterator<Item = &'a RoaringBitmap>,
{
    fn sub(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next().cloned() {
            Some(mut first) => {
                iter.for_each(|rb| first -= rb);
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}

impl<I> MultiSub<RoaringBitmap> for I
where
    I: IntoIterator<Item = RoaringBitmap>,
{
    fn sub(self) -> RoaringBitmap {
        let mut iter = self.into_iter();
        match iter.next() {
            Some(mut first) => {
                iter.for_each(|rb| first -= rb);
                first
            }
            None => RoaringBitmap::default(),
        }
    }
}
