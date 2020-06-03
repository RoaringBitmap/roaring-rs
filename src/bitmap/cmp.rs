use std::iter::Peekable;
use std::slice;

use super::container::Container;
use crate::RoaringBitmap;

struct Pairs<'a>(
    Peekable<slice::Iter<'a, Container>>,
    Peekable<slice::Iter<'a, Container>>,
);

impl RoaringBitmap {
    fn pairs<'a>(&'a self, other: &'a RoaringBitmap) -> Pairs<'a> {
        Pairs(
            self.containers.iter().peekable(),
            other.containers.iter().peekable(),
        )
    }

    /// Returns true if the set has no elements in common with other. This is equivalent to
    /// checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb1.is_disjoint(&rb2), true);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb1.is_disjoint(&rb2), false);
    ///
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.pairs(other)
            .filter(|&(c1, c2)| c1.is_some() && c2.is_some())
            .all(|(c1, c2)| c1.unwrap().is_disjoint(c2.unwrap()))
    }

    /// Returns `true` if this set is a subset of `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), false);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), true);
    ///
    /// rb1.insert(2);
    ///
    /// assert_eq!(rb1.is_subset(&rb2), false);
    /// ```
    pub fn is_subset(&self, other: &Self) -> bool {
        for pair in self.pairs(other) {
            match pair {
                (None, _) => (),
                (_, None) => {
                    return false;
                }
                (Some(c1), Some(c2)) => {
                    if !c1.is_subset(c2) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns `true` if this set is a superset of `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1 = RoaringBitmap::new();
    /// let mut rb2 = RoaringBitmap::new();
    ///
    /// rb1.insert(1);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), false);
    ///
    /// rb2.insert(1);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), true);
    ///
    /// rb1.insert(2);
    ///
    /// assert_eq!(rb2.is_superset(&rb1), false);
    /// ```
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    pub fn multi_union<'a, I>(bitmaps: I) -> Self
    where I: IntoIterator<Item = &'a Self>
    {
        let iter = bitmaps.into_iter().map(|b| b.containers.iter().peekable());
        let muple = Muple::new(iter);

        let mut containers = Vec::new(); // TODO with_capacity
        for mut cs in muple {
            let mut a = cs.pop().unwrap().clone(); // safe
            cs.into_iter().for_each(|c| a.union_with(c));
            containers.push(a);
        }

        RoaringBitmap { containers }
    }
}

impl<'a> Iterator for Pairs<'a> {
    type Item = (Option<&'a Container>, Option<&'a Container>);

    fn next(&mut self) -> Option<Self::Item> {
        enum Which {
            Left,
            Right,
            Both,
            None,
        };
        let which = match (self.0.peek(), self.1.peek()) {
            (None, None) => Which::None,
            (Some(_), None) => Which::Left,
            (None, Some(_)) => Which::Right,
            (Some(c1), Some(c2)) => match (c1.key, c2.key) {
                (key1, key2) if key1 == key2 => Which::Both,
                (key1, key2) if key1 < key2 => Which::Left,
                (key1, key2) if key1 > key2 => Which::Right,
                (_, _) => unreachable!(),
            },
        };
        match which {
            Which::Left => Some((self.0.next(), None)),
            Which::Right => Some((None, self.1.next())),
            Which::Both => Some((self.0.next(), self.1.next())),
            Which::None => None,
        }
    }
}

use std::cell::RefCell;
use std::cmp::{Reverse, Ordering};
use std::collections::BinaryHeap;

// This struct is here to bypass the `Ord::cmp` limitation
// where it is not possible to mutate self to get or compute a value.
struct InteriorMutable<'a>(RefCell<Peekable<slice::Iter<'a, Container>>>);

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

struct Muple<'a>(BinaryHeap<Reverse<InteriorMutable<'a>>>);

impl<'a> Muple<'a> {
    fn new<I>(iters: I) -> Muple<'a>
    where I: IntoIterator<Item = Peekable<slice::Iter<'a, Container>>>
    {
        let mut heap = BinaryHeap::new();

        iters.into_iter().for_each(|iter| {
            heap.push(Reverse(InteriorMutable(RefCell::new(iter))));
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
            },
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
                },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_union() {
        let a: RoaringBitmap = (0..5).collect();
        let b = (5..10).collect();
        let c = (10..15).collect();
        let d = (0..4).collect();

        let expected = (0..15).collect();
        let out = RoaringBitmap::multi_union(&[a, b, c, d]);

        assert_eq!(out, expected);
    }
}
