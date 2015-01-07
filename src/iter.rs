use std::{ u16 };
use std::num;
use std::slice;

use util;
use util::{ Either, Halveable, ExtInt, bits };
use util::Either::{ Left, Right };
use container::Container;

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, Size> where Size: ExtInt {
    inner_iter: Option<(<Size as Halveable>::HalfSize, Box<Iterator<Item = <Size as Halveable>::HalfSize> + 'a>)>,
    container_iter: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
}

#[inline]
fn calc<Size>(key: <Size as Halveable>::HalfSize, value: <Size as Halveable>::HalfSize) -> Size where Size: ExtInt {
    let bits = util::bits::<Size>();
    let _key: Size = util::cast(key);
    let _value: Size = util::cast(value);
    (_key << bits) + _value
}

#[inline]
fn next_iter<'a, Size>(container_iter: &mut slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Option<(<Size as Halveable>::HalfSize, Box<Iterator<Item = <Size as Halveable>::HalfSize> + 'a>)> where Size: ExtInt {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

#[inline]
pub fn new<'a, Size>(mut container_iter: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Iter<'a, Size> where Size: ExtInt {
    Iter {
        inner_iter: next_iter(&mut container_iter),
        container_iter: container_iter
    }
}

impl<'a, Size> Iter<'a, Size> where Size: ExtInt {
    #[inline]
    fn choose_next(&mut self) -> Option<Either<Size, Option<(<Size as Halveable>::HalfSize, Box<Iterator<Item = <Size as Halveable>::HalfSize> + 'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match iter.next() {
                Some(value) => Left(calc(key, value)),
                None => Right(next_iter(&mut self.container_iter)),
            }),
            None => None,
        }
    }
}

impl<'a, Size> Iterator for Iter<'a, Size> where Size: ExtInt {
    type Item = Size;

    fn next(&mut self) -> Option<Size> {
        match self.choose_next() {
            None => None,
            Some(Left(val)) => Some(val),
            Some(Right(new_iter)) => {
                self.inner_iter = new_iter;
                self.next()
            },
        }
    }
}

/// An iterator for `RoaringBitmap`.
pub struct UnionIter<'a, Size> where Size: ExtInt {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod union {
    use util::{ ExtInt };
    use super::{ Iter, UnionIter };

    #[inline]
    pub fn new<'a, Size>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> UnionIter<'a, Size> where Size: ExtInt {
        UnionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size> Iterator for UnionIter<'a, Size> where Size: ExtInt {
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
pub struct IntersectionIter<'a, Size> where Size: ExtInt {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod intersection {
    use util::{ ExtInt };
    use super::{ Iter, IntersectionIter };

    #[inline]
    pub fn new<'a, Size>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> IntersectionIter<'a, Size> where Size: ExtInt {
        IntersectionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size> Iterator for IntersectionIter<'a, Size> where Size: ExtInt {
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
pub struct DifferenceIter<'a, Size> where Size: ExtInt {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod difference {
    use util::{ ExtInt };
    use super::{ Iter, DifferenceIter };

    #[inline]
    pub fn new<'a, Size>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> DifferenceIter<'a, Size> where Size: ExtInt {
        DifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size> Iterator for DifferenceIter<'a, Size> where Size: ExtInt {
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
pub struct SymmetricDifferenceIter<'a, Size> where Size: ExtInt {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size>,
    iter2: Iter<'a, Size>,
}

pub mod symmetric_difference {
    use util::{ ExtInt };
    use super::{ Iter, SymmetricDifferenceIter };

    #[inline]
    pub fn new<'a, Size>(mut iter1: Iter<'a, Size>, mut iter2: Iter<'a, Size>) -> SymmetricDifferenceIter<'a, Size> where Size: ExtInt {
        SymmetricDifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size> Iterator for SymmetricDifferenceIter<'a, Size> where Size: ExtInt {
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
