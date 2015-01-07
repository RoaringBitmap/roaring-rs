use std::slice;

use util;
use util::{ Either, ExtInt, bits };
use util::Either::{ Left, Right };
use container::Container;

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, Size: ExtInt, HalfSize: ExtInt + 'a> {
    inner_iter: Option<(HalfSize, Box<Iterator<Item = HalfSize> + 'a>)>,
    container_iter: slice::Iter<'a, Container<HalfSize>>,
}

#[inline]
fn calc<Size: ExtInt, HalfSize: ExtInt>(key: HalfSize, value: HalfSize) -> Size {
    let bits = util::bits::<Size>();
    let _key: Size = util::cast(key);
    let _value: Size = util::cast(value);
    (_key << bits) + _value
}

#[inline]
fn next_iter<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(container_iter: &mut slice::Iter<'a, Container<HalfSize>>) -> Option<(HalfSize, Box<Iterator<Item = HalfSize> + 'a>)> {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

#[inline]
pub fn new<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(mut container_iter: slice::Iter<'a, Container<HalfSize>>) -> Iter<'a, Size, HalfSize> {
    Iter {
        inner_iter: next_iter::<'a, Size, HalfSize>(&mut container_iter),
        container_iter: container_iter
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt + 'a> Iter<'a, Size, HalfSize> {
    #[inline]
    fn choose_next(&mut self) -> Option<Either<Size, Option<(HalfSize, Box<Iterator<Item = HalfSize> + 'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match iter.next() {
                Some(value) => Left(calc(key, value)),
                None => Right(next_iter::<'a, Size, HalfSize>(&mut self.container_iter)),
            }),
            None => None,
        }
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt> Iterator for Iter<'a, Size, HalfSize> {
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
pub struct UnionIter<'a, Size: ExtInt, HalfSize: ExtInt + 'a> {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size, HalfSize>,
    iter2: Iter<'a, Size, HalfSize>,
}

pub mod union {
    use util::{ ExtInt };
    use super::{ Iter, UnionIter };

    #[inline]
    pub fn new<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(mut iter1: Iter<'a, Size, HalfSize>, mut iter2: Iter<'a, Size, HalfSize>) -> UnionIter<'a, Size, HalfSize> {
        UnionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt + 'a> Iterator for UnionIter<'a, Size, HalfSize> {
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
pub struct IntersectionIter<'a, Size: ExtInt, HalfSize: ExtInt + 'a> {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size, HalfSize>,
    iter2: Iter<'a, Size, HalfSize>,
}

pub mod intersection {
    use util::{ ExtInt };
    use super::{ Iter, IntersectionIter };

    #[inline]
    pub fn new<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(mut iter1: Iter<'a, Size, HalfSize>, mut iter2: Iter<'a, Size, HalfSize>) -> IntersectionIter<'a, Size, HalfSize> {
        IntersectionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt + 'a> Iterator for IntersectionIter<'a, Size, HalfSize> {
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
pub struct DifferenceIter<'a, Size: ExtInt, HalfSize: ExtInt + 'a> {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size, HalfSize>,
    iter2: Iter<'a, Size, HalfSize>,
}

pub mod difference {
    use util::{ ExtInt };
    use super::{ Iter, DifferenceIter };

    #[inline]
    pub fn new<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(mut iter1: Iter<'a, Size, HalfSize>, mut iter2: Iter<'a, Size, HalfSize>) -> DifferenceIter<'a, Size, HalfSize> {
        DifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt + 'a> Iterator for DifferenceIter<'a, Size, HalfSize> {
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
pub struct SymmetricDifferenceIter<'a, Size: ExtInt, HalfSize: ExtInt + 'a> {
    current1: Option<Size>,
    current2: Option<Size>,
    iter1: Iter<'a, Size, HalfSize>,
    iter2: Iter<'a, Size, HalfSize>,
}

pub mod symmetric_difference {
    use util::{ ExtInt };
    use super::{ Iter, SymmetricDifferenceIter };

    #[inline]
    pub fn new<'a, Size: ExtInt, HalfSize: ExtInt + 'a>(mut iter1: Iter<'a, Size, HalfSize>, mut iter2: Iter<'a, Size, HalfSize>) -> SymmetricDifferenceIter<'a, Size, HalfSize> {
        SymmetricDifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a, Size: ExtInt, HalfSize: ExtInt + 'a> Iterator for SymmetricDifferenceIter<'a, Size, HalfSize> {
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
