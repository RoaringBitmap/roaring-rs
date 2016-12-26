use std::iter::Peekable;

use RoaringBitmap;
use iter::Iter;
use util::{ ExtInt, Halveable };
use container::{ Container };

type HalfContainer<Size> = Container<<Size as Halveable>::HalfSize>;

/// An iterator for `RoaringBitmap`.
pub struct UnionIter<'a, Size: ExtInt + Halveable + 'a>(Peekable<Iter<'a, Size>>, Peekable<Iter<'a, Size>>) where <Size as Halveable>::HalfSize: 'a;

/// An iterator for `RoaringBitmap`.
pub struct IntersectionIter<'a, Size: ExtInt + Halveable + 'a>(Peekable<Iter<'a, Size>>, Peekable<Iter<'a, Size>>) where <Size as Halveable>::HalfSize: 'a;

/// An iterator for `RoaringBitmap`.
pub struct DifferenceIter<'a, Size: ExtInt + Halveable + 'a>(Peekable<Iter<'a, Size>>, Peekable<Iter<'a, Size>>) where <Size as Halveable>::HalfSize: 'a;

/// An iterator for `RoaringBitmap`.
pub struct SymmetricDifferenceIter<'a, Size: ExtInt + Halveable + 'a>(Peekable<Iter<'a, Size>>, Peekable<Iter<'a, Size>>) where <Size as Halveable>::HalfSize: 'a;

impl<Size: ExtInt + Halveable> RoaringBitmap<Size> {
    /// Returns an iterator over the union of this bitmap with the `other` bitmap.
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
    /// rb1.insert(2);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    ///
    /// let mut iter = rb1.union(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn union<'a>(&'a self, other: &'a Self) -> UnionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        UnionIter(self.iter().peekable(), other.iter().peekable())
    }

    /// Returns an iterator over the intersection of this bitmap with the `other` bitmap.
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
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter = rb1.intersection(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(4));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn intersection<'a>(&'a self, other: &'a Self) -> IntersectionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        IntersectionIter(self.iter().peekable(), other.iter().peekable())
    }

    /// Returns an iterator over the set of values in `this` that are not in `other`.
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
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter1 = rb1.difference(&rb2);
    ///
    /// assert_eq!(iter1.next(), Some(2));
    /// assert_eq!(iter1.next(), None);
    ///
    /// let mut iter2 = rb2.difference(&rb1);
    ///
    /// assert_eq!(iter2.next(), Some(3));
    /// assert_eq!(iter2.next(), None);
    /// ```
    #[inline]
    pub fn difference<'a>(&'a self, other: &'a Self) -> DifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        DifferenceIter(self.iter().peekable(), other.iter().peekable())
    }

    /// Returns an iterator over the set of values in `this` that are not in `other` + in `other`
    /// that are not in `this`.
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
    /// rb1.insert(2);
    /// rb1.insert(4);
    ///
    /// rb2.insert(1);
    /// rb2.insert(3);
    /// rb2.insert(4);
    ///
    /// let mut iter = rb1.symmetric_difference(&rb2);
    ///
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), Some(3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        SymmetricDifferenceIter(self.iter().peekable(), other.iter().peekable())
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for UnionIter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.0.peek().cloned(), self.1.peek().cloned()) {
            (None, None) => None,
            (val, None) => { self.0.next(); val },
            (None, val) => { self.1.next(); val },
            (val1, val2) if val1 < val2 => { self.0.next(); val1 },
            (val1, val2) if val1 > val2 => { self.1.next(); val2 },
            (val1, val2) if val1 == val2 => {
                self.0.next();
                self.1.next();
                val1
            },
            _ => unreachable!(),
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for IntersectionIter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.0.peek().cloned(), self.1.peek().cloned()) {
            (None, _) | (_, None) => None,
            (val1, val2) if val1 < val2 => { self.0.next(); self.next() },
            (val1, val2) if val1 > val2 => { self.1.next(); self.next() },
            (val1, val2) if val1 == val2 => {
                self.0.next();
                self.1.next();
                val1
            },
            _ => unreachable!(),
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for DifferenceIter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        loop {
            match (self.0.peek().cloned(), self.1.peek().cloned()) {
                (None, _) | (_, None) => return None,
                (val1, val2) if val1 < val2 => { self.0.next(); return val1; }
                (val1, val2) if val1 > val2 => { self.1.next(); }
                (val1, val2) if val1 == val2 => {
                    self.0.next();
                    self.1.next();
                }
                _ => unreachable!(),
            }
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for SymmetricDifferenceIter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.0.peek().cloned(), self.1.peek().cloned()) {
            (None, _) | (_, None) => None,
            (val1, val2) if val1 < val2 => { self.0.next(); val1 },
            (val1, val2) if val1 > val2 => { self.1.next(); val2 },
            (val1, val2) if val1 == val2 => {
                self.0.next();
                self.1.next();
                self.next()
            },
            _ => unreachable!(),
        }
    }
}
