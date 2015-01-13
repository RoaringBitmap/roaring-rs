use std::{ u16 };
use std::slice;

use util::Either::{ self, Left, Right };
use container::{ Container };

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a> {
    inner_iter: Option<(u16, Box<Iterator<Item = u16> + 'a>)>,
    container_iter: slice::Iter<'a, Container<u16>>,
}

#[inline]
fn calc(key: u16, value: u16) -> u32 {
    ((key as u32) << u16::BITS) + (value as u32)
}

#[inline]
fn next_iter<'a>(container_iter: &mut slice::Iter<'a, Container<u16>>) -> Option<(u16, Box<Iterator<Item = u16> + 'a>)> {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

#[inline]
pub fn new<'a>(mut container_iter: slice::Iter<'a, Container<u16>>) -> Iter<'a> {
    Iter {
        inner_iter: next_iter(&mut container_iter),
        container_iter: container_iter
    }
}

impl<'a> Iter<'a> {
    #[inline]
    fn choose_next(&mut self) -> Option<Either<u32, Option<(u16, Box<Iterator<Item = u16> + 'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match iter.next() {
                Some(value) => Left(calc(key, value)),
                None => Right(next_iter(&mut self.container_iter)),
            }),
            None => None,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
pub struct UnionIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

pub mod union {
    use super::{ Iter, UnionIter };

    #[inline]
    pub fn new<'a>(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> UnionIter<'a> {
        UnionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator for UnionIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
pub struct IntersectionIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

pub mod intersection {
    use super::{ Iter, IntersectionIter };

    #[inline]
    pub fn new<'a>(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> IntersectionIter<'a> {
        IntersectionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator for IntersectionIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
pub struct DifferenceIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

pub mod difference {
    use super::{ Iter, DifferenceIter };

    #[inline]
    pub fn new<'a>(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> DifferenceIter<'a> {
        DifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator for DifferenceIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
pub struct SymmetricDifferenceIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

pub mod symmetric_difference {
    use super::{ Iter, SymmetricDifferenceIter };

    #[inline]
    pub fn new<'a>(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> SymmetricDifferenceIter<'a> {
        SymmetricDifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator for SymmetricDifferenceIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
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
