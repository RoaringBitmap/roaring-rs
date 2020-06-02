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

struct Muple<'a>(Vec<(&'a Container, Peekable<slice::Iter<'a, Container>>)>);

impl<'a> Muple<'a> {
    fn new(iters: Vec<Peekable<slice::Iter<'a, Container>>>) -> Muple<'a> {
        let mut vec = Vec::with_capacity(iters.len());

        // We peek the first key of every the container, this is ugly but
        // the sort_unstable_by_key function does not allow us to mutable the
        // element we are evaluating, probably to logic errors. Same for the BinaryHeap.
        for mut i in iters {
            if let Some(c) = i.peek() {
                vec.push((*c, i));
            }
        }

        vec.sort_unstable_by_key(|(c, _)| c.key);

        Muple(vec)
    }
}

impl<'a> Iterator for Muple<'a> {
    type Item = Vec<&'a Container>;

    fn next(&mut self) -> Option<Self::Item> {
        // We retrieve the lowest key that we must return containers for.
        let key = match self.0.get(0) {
            Some((c, _)) => c.key,
            None => return None,
        };

        let mut output = Vec::new();
        let mut to_remove = Vec::new();

        // We iterate over the containers iterators that are related to the lowest key,
        // poll the containers to return and mark the empty containers identified.
        for (i, (c, iter)) in self.0.iter_mut().enumerate().take_while(|(_, (c, _))| c.key == key) {
            let container = iter.next().unwrap();
            output.push(container);
            match iter.next() {
                Some(x) => *c = x,
                None => to_remove.push(i),
            }
        }

        // We remove all the containers iterator that are empty.
        // We reverse iterate to avoid invalidating the indexes when swap removing.
        to_remove.into_iter().rev().for_each(|i| drop(self.0.swap_remove(i)));

        // We sort to move the lowest keys at the front of the list.
        self.0.sort_unstable_by_key(|(c, _)| c.key);

        Some(output)
    }
}
