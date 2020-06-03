use std::cell::RefCell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::iter::Peekable;
use std::slice;

use super::container::Container;
use crate::RoaringBitmap;

// This struct is here to bypass the `Ord::cmp` limitation
// where it is not possible to mutate self to get or compute a value.
struct InteriorMutable<'a>(RefCell<Peekable<slice::Iter<'a, Container>>>);

struct Muple<'a>(BinaryHeap<Reverse<InteriorMutable<'a>>>);

impl RoaringBitmap {
    /// Unions in-place with the specified others bitmaps.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let rb1 = (0..5).collect();
    /// let rb2 = (5..10).collect();
    /// let rb3 = (10..15).collect();
    /// let rb4 = (0..4).collect();
    ///
    /// let out = RoaringBitmap::union_of(&[rb1, rb2, rb3, rb4]);
    ///
    /// assert_eq!(out, (0..15).collect());
    /// ```
    pub fn union_of<'a>(bitmaps: impl IntoIterator<Item = &'a Self>) -> Self {
        let iter = bitmaps.into_iter().map(|b| b.containers.iter().peekable());
        let muple = Muple::new(iter);

        let mut stores = Vec::new();
        for mut cs in muple {
            let a = cs.pop().unwrap().clone(); // safe
            let mut store = a.store;
            cs.into_iter().for_each(|c| store.union_with(&c.store));
            stores.push((a.key, store));
        }

        // We reconstruct the containers from the stores
        let containers = stores
            .into_iter()
            .map(|(key, store)| {
                let mut container = Container {
                    key,
                    len: store.len(),
                    store,
                };
                container.ensure_correct_store();
                container
            })
            .collect();

        RoaringBitmap { containers }
    }
}

impl Ord for InteriorMutable<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut c1 = self.0.borrow_mut();
        let mut c2 = other.0.borrow_mut();

        match (c1.peek(), c2.peek()) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less, // move Nones to the back
            (None, Some(_)) => Ordering::Greater,
            (Some(c1), Some(c2)) => match (c1.key, c2.key) {
                (key1, key2) if key1 == key2 => Ordering::Equal,
                (key1, key2) if key1 < key2 => Ordering::Less,
                (key1, key2) if key1 > key2 => Ordering::Greater,
                (_, _) => unreachable!(),
            },
        }
    }
}

impl<'a> InteriorMutable<'a> {
    fn new(iter: Peekable<slice::Iter<'a, Container>>) -> Self {
        InteriorMutable(RefCell::new(iter))
    }
}

impl PartialOrd for InteriorMutable<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InteriorMutable<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for InteriorMutable<'_> {}

impl<'a> Muple<'a> {
    fn new<I>(iters: I) -> Muple<'a>
    where
        I: IntoIterator<Item = Peekable<slice::Iter<'a, Container>>>,
    {
        let mut heap = BinaryHeap::new();

        iters.into_iter().for_each(|iter| {
            heap.push(Reverse(InteriorMutable::new(iter)));
        });

        Muple(heap)
    }
}

impl<'a> Iterator for Muple<'a> {
    type Item = Vec<&'a Container>;

    fn next(&mut self) -> Option<Self::Item> {
        // We retrieve the lowest key that we must return containers for.
        let key = match self.0.peek_mut() {
            Some(mut iter) => {
                match (iter.0).0.get_mut().peek() {
                    Some(c) => c.key,
                    // Nones are moved to the back,
                    // it means that we only have empty iterators.
                    None => return None,
                }
            }
            None => return None,
        };

        let mut output = Vec::new();

        while let Some(mut iter) = self.0.peek_mut() {
            let containers = (iter.0).0.get_mut();
            match containers.peek() {
                // This iterator gives us a key that is corresponding
                // to the lowest one, we must return this container
                Some(c) if c.key == key => {
                    let container = containers.next().unwrap();
                    output.push(container);
                }
                // Keys are no more equal to the lowest one, we must stop.
                Some(_) => break,
                // This iterator is exhauted we must stop here as empty iterators
                // are pushed to the back of the heap. This means that we will
                // continue to see this empty iterator if we continue peeking.
                None => break,
            }
        }

        if !output.is_empty() {
            Some(output)
        } else {
            None
        }
    }
}
