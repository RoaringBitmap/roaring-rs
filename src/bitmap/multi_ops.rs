use std::cmp::Ordering;
use std::collections::binary_heap::{BinaryHeap, PeekMut};
use std::mem;

use crate::bitmap::container::Container;
use crate::bitmap::store::Store;
use crate::RoaringBitmap;

struct PeekedRefContainer<'a> {
    container: &'a Container,
    iter: std::slice::Iter<'a, Container>,
}

impl Ord for PeekedRefContainer<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.container.key.cmp(&other.container.key).reverse()
    }
}

impl PartialOrd for PeekedRefContainer<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PeekedRefContainer<'_> {}

impl PartialEq for PeekedRefContainer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.container.key == other.container.key
    }
}

struct PeekedContainer {
    container: Container,
    iter: std::vec::IntoIter<Container>,
}

impl Ord for PeekedContainer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.container.key.cmp(&other.container.key).reverse()
    }
}

impl PartialOrd for PeekedContainer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PeekedContainer {}

impl PartialEq for PeekedContainer {
    fn eq(&self, other: &Self) -> bool {
        self.container.key == other.container.key
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
                heap.push(PeekedRefContainer { container, iter });
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
        self.into_iter().reduce(|a, b| a & b).unwrap_or_default()
    }
}
