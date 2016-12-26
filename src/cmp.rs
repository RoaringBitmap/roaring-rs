use std::slice;
use std::cmp::Ordering;
use std::iter::Peekable;

use RoaringBitmap;
use util::{ ExtInt, Halveable };
use container::Container;

type HalfContainer<Size> = Container<<Size as Halveable>::HalfSize>;
type PeekableContainerIter<'a, Size> = Peekable<slice::Iter<'a, HalfContainer<Size>>>;

struct Pairs<'a, Size: ExtInt + Halveable + 'a>(PeekableContainerIter<'a, Size>, PeekableContainerIter<'a, Size>) where <Size as Halveable>::HalfSize: 'a;

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    fn pairs<'a>(&'a self, other: &'a RoaringBitmap<Size>) -> Pairs<'a, Size> where <Size as Halveable>::HalfSize: 'a {
        Pairs(self.containers.iter().peekable(), other.containers.iter().peekable())
    }

    /// Returns true if the set has no elements in common with other. This is equivalent to
    /// checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = RoaringBitmap::new();
    /// let mut rb2: RoaringBitmap<u32> = RoaringBitmap::new();
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
    #[inline]
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
    /// let mut rb1: RoaringBitmap<u32> = RoaringBitmap::new();
    /// let mut rb2: RoaringBitmap<u32> = RoaringBitmap::new();
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
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        for pair in self.pairs(other) {
            match pair {
                (None, _) => (),
                (_, None) => { return false; },
                (Some(c1), Some(c2)) => if !c1.is_subset(c2) { return false; },
            }
        }
        true
    }

    /// Returns `true` if this set is a subset of `other`.
    #[inline]
    pub fn is_subset_opt(&self, other: &Self) -> bool {
        let tv = &self.containers;
        let ov = &other.containers;
        let tlen = tv.len();
        let olen = ov.len();
        if tlen > olen { return false; }
        let mut ti = 0;
        let mut oi = 0;
        loop {
            let tc = &tv[ti];
            let oc = &ov[oi];
            match tc.key.cmp(&oc.key) {
                Ordering::Less => { return false; },
                Ordering::Equal => {
                    if !tc.is_subset(oc) { return false; }
                    ti += 1;
                    if ti >= tlen { return true; }
                },
                Ordering::Greater => (),
            }
            oi += 1;
            if oi >= olen { return false }
        }
    }

    /// Returns `true` if this set is a superset of `other`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roaring::RoaringBitmap;
    ///
    /// let mut rb1: RoaringBitmap<u32> = RoaringBitmap::new();
    /// let mut rb2: RoaringBitmap<u32> = RoaringBitmap::new();
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
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }
}

impl<'a, Size: ExtInt + Halveable> Iterator for Pairs<'a, Size> {
    type Item = (Option<&'a HalfContainer<Size>>, Option<&'a HalfContainer<Size>>);

    fn next(&mut self) -> Option<Self::Item> {
        enum Which { Left, Right, Both, None };
        let which = match (self.0.peek(), self.1.peek()) {
            (None, None) => Which::None,
            (Some(_), None) => Which::Left,
            (None, Some(_)) => Which::Right,
            (Some(c1), Some(c2)) => match (c1.key, c2.key) {
                (key1, key2) if key1 == key2 => Which::Both,
                (key1, key2) if key1 < key2 => Which::Left,
                (key1, key2) if key1 > key2 => Which::Right,
                (_, _) => unreachable!(),
            }
        };
        match which {
            Which::Left => Some((self.0.next(), None)),
            Which::Right => Some((None, self.1.next())),
            Which::Both => Some((self.0.next(), self.1.next())),
            Which::None => None,
        }
    }
}
