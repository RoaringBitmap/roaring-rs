use std::slice;

use util::{ ExtInt, Halveable };
use container::{ self, Container };

type HalfContainer<Size> = Container<<Size as Halveable>::HalfSize>;

enum Next<'a, Size: ExtInt + Halveable + 'a> {
    Done,
    Value(Size),
    NewIter(Option<container::Iter<'a, <Size as Halveable>::HalfSize>>),
}

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize: 'a {
    inner_iter: Option<container::Iter<'a, <Size as Halveable>::HalfSize>>,
    container_iters: slice::Iter<'a, HalfContainer<Size>>,
}

#[inline]
pub fn new<Size: ExtInt + Halveable>(mut container_iters: slice::Iter<HalfContainer<Size>>) -> Iter<Size> {
    Iter {
        inner_iter: container_iters.next().map(|i| i.iter()),
        container_iters: container_iters,
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    #[inline]
    fn choose_next(&mut self) -> Next<'a, Size> {
        match self.inner_iter {
            Some(ref mut inner_iter) => match inner_iter.next() {
                Some(value) => Next::Value(Halveable::join(inner_iter.key, value)),
                None => Next::NewIter(self.container_iters.next().map(|i| i.iter())),
            },
            None => Next::Done,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for Iter<'a, Size> where <Size as Halveable>::HalfSize: 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match self.choose_next() {
            Next::Done => None,
            Next::Value(val) => Some(val),
            Next::NewIter(new_iter) => {
                self.inner_iter = new_iter;
                self.next()
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let next = self.container_iters.clone().map(|container| container.len() as usize).fold(0, |acc, len| acc + len);
        match self.inner_iter {
            Some(ref inner_iter) => match inner_iter.size_hint() {
                (min, max) => (next + min, max.map(|m| next + m)),
            },
            None => (next, Some(next)),
        }
    }
}

/// An iterator for `RoaringBitmap`.
pub struct UnionIter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod union {
    use util::{ ExtInt, Halveable };
    use super::{ Iter, UnionIter };

    #[inline]
    pub fn new<'a, Size: ExtInt + Halveable + 'a>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> UnionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        UnionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for UnionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.current1, self.current2) {
            (None, None) => None,
            (val, None) => { self.current1 = self.iter1.next(); val },
            (None, val) => { self.current2 = self.iter2.next(); val },
            (val1, val2) if val1 < val2 => { self.current1 = self.iter1.next(); val1 },
            (val1, val2) if val1 > val2 => { self.current2 = self.iter2.next(); val2 },
            (val1, val2) if val1 == val2 => {
                self.current1 = self.iter1.next();
                self.current2 = self.iter2.next();
                val1
            },
            _ => panic!("Should not be possible to get here"),
        }
    }
}

/// An iterator for `RoaringBitmap`.
pub struct IntersectionIter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod intersection {
    use util::{ ExtInt, Halveable };
    use super::{ Iter, IntersectionIter };

    #[inline]
    pub fn new<'a, Size: ExtInt + Halveable + 'a>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> IntersectionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        IntersectionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for IntersectionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.current1, self.current2) {
            (None, _) | (_, None) => None,
            (val1, val2) if val1 < val2 => { self.current1 = self.iter1.next(); self.next() },
            (val1, val2) if val1 > val2 => { self.current2 = self.iter2.next(); self.next() },
            (val1, val2) if val1 == val2 => {
                self.current1 = self.iter1.next();
                self.current2 = self.iter2.next();
                val1
            },
            _ => panic!("Should not be possible to get here"),
        }
    }
}

/// An iterator for `RoaringBitmap`.
pub struct DifferenceIter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod difference {
    use util::{ ExtInt, Halveable };
    use super::{ Iter, DifferenceIter };

    #[inline]
    pub fn new<'a, Size: ExtInt + Halveable + 'a>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> DifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        DifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for DifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        loop {
            match (self.current1, self.current2) {
                (None, _) | (_, None) => return None,
                (val1, val2) if val1 < val2 => { self.current1 = self.iter1.next(); return val1; },
                (val1, val2) if val1 > val2 => self.current2 = self.iter2.next(),
                (val1, val2) if val1 == val2 => {
                    self.current1 = self.iter1.next();
                    self.current2 = self.iter2.next();
                },
                _ => panic!("Should not be possible to get here"),
            }
        }
    }
}

/// An iterator for `RoaringBitmap`.
pub struct SymmetricDifferenceIter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod symmetric_difference {
    use util::{ ExtInt, Halveable };
    use super::{ Iter, SymmetricDifferenceIter };

    #[inline]
    pub fn new<'a, Size: ExtInt + Halveable + 'a>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> SymmetricDifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
        SymmetricDifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for SymmetricDifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match (self.current1, self.current2) {
            (None, _) | (_, None) => None,
            (val1, val2) if val1 < val2 => { self.current1 = self.iter1.next(); val1 },
            (val1, val2) if val1 > val2 => { self.current2 = self.iter2.next(); val2 },
            (val1, val2) if val1 == val2 => {
                self.current1 = self.iter1.next();
                self.current2 = self.iter2.next();
                self.next()
            },
            _ => panic!("Should not be possible to get here"),
        }
    }
}
