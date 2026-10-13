use alloc::collections::btree_map;
use core::iter::Peekable;

use crate::RoaringBitmap;
use crate::RoaringTreemap;

pub(crate) struct Pairs<'a>(
    Peekable<btree_map::Iter<'a, u32, RoaringBitmap>>,
    Peekable<btree_map::Iter<'a, u32, RoaringBitmap>>,
);

impl RoaringTreemap {
    pub(crate) fn pairs<'a>(&'a self, other: &'a RoaringTreemap) -> Pairs<'a> {
        Pairs(self.map.iter().peekable(), other.map.iter().peekable())
    }

    /// Returns true if the set has no elements in common with other. This is equivalent to
    /// checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1 = RoaringTreemap::new();
    /// let mut rb2 = RoaringTreemap::new();
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
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1 = RoaringTreemap::new();
    /// let mut rb2 = RoaringTreemap::new();
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
    /// use roaring::RoaringTreemap;
    ///
    /// let mut rb1 = RoaringTreemap::new();
    /// let mut rb2 = RoaringTreemap::new();
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

impl PartialEq for RoaringTreemap {
    fn eq(&self, other: &RoaringTreemap) -> bool {
        let lhs = self.map.iter().filter(|(_, rb)| !rb.is_empty());
        let rhs = other.map.iter().filter(|(_, rb)| !rb.is_empty());
        lhs.eq(rhs)
    }
}

impl<'a> Iterator for Pairs<'a> {
    type Item = (Option<&'a RoaringBitmap>, Option<&'a RoaringBitmap>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.0.next_if(|(_, rb)| rb.is_empty()).is_some() {}
        while self.1.next_if(|(_, rb)| rb.is_empty()).is_some() {}
        enum Which {
            Left,
            Right,
            Both,
            None,
        }
        let which = match (self.0.peek(), self.1.peek()) {
            (None, None) => Which::None,
            (Some(_), None) => Which::Left,
            (None, Some(_)) => Which::Right,
            (Some(c1), Some(c2)) => match (c1.0, c2.0) {
                (key1, key2) if key1 == key2 => Which::Both,
                (key1, key2) if key1 < key2 => Which::Left,
                (key1, key2) if key1 > key2 => Which::Right,
                (_, _) => unreachable!(),
            },
        };
        match which {
            Which::Left => Some((self.0.next().map(|e| e.1), None)),
            Which::Right => Some((None, self.1.next().map(|e| e.1))),
            Which::Both => Some((self.0.next().map(|e| e.1), self.1.next().map(|e| e.1))),
            Which::None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{RoaringBitmap, RoaringTreemap};

    #[test]
    fn eq_and_subset_ignore_empty_partitions() {
        let mut non_empty = RoaringBitmap::new();
        non_empty.insert(42);

        let with_empty =
            RoaringTreemap::from_bitmaps([(0, RoaringBitmap::new()), (1, non_empty.clone())]);
        let without_empty = RoaringTreemap::from_bitmaps([(1, non_empty)]);

        assert_eq!(with_empty, without_empty);
        assert!(with_empty.is_subset(&without_empty));
        assert!(without_empty.is_subset(&with_empty));

        let only_empty = RoaringTreemap::from_bitmaps([(0, RoaringBitmap::new())]);
        let default_empty = RoaringTreemap::new();
        assert_eq!(only_empty, default_empty);
        assert!(only_empty.is_subset(&default_empty));
        assert!(default_empty.is_subset(&only_empty));
        assert!(only_empty.is_subset(&without_empty));
        assert!(!without_empty.is_subset(&only_empty));
    }
}
