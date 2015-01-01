use std::{ u16 };
use std::slice;

use util::Either;
use util::Either::{ Left, Right };
use container::{ Container };

pub struct Iter<'a> {
    inner_iter: Option<(u16, Box<Iterator<u16> + 'a>)>,
    container_iter: slice::Iter<'a, Container>,
}

#[inline]
fn calc(key: u16, value: u16) -> u32 {
    ((key as u32) << u16::BITS) + (value as u32)
}

#[inline]
fn next_iter<'a>(container_iter: &mut slice::Iter<'a, Container>) -> Option<(u16, Box<Iterator<u16> + 'a>)> {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

impl<'a> Iter<'a> {
    #[inline]
    pub fn new(mut container_iter: slice::Iter<'a, Container>) -> Iter<'a> {
        Iter {
            inner_iter: next_iter(&mut container_iter),
            container_iter: container_iter
        }
    }

    #[inline]
    fn choose_next(&mut self) -> Option<Either<u32, Option<(u16, Box<Iterator<u16> + 'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match iter.next() {
                Some(value) => Left(calc(key, value)),
                None => Right(next_iter(&mut self.container_iter)),
            }),
            None => None,
        }
    }
}

impl<'a> Iterator<u32> for Iter<'a> {
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

pub struct UnionIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

impl<'a> UnionIter<'a> {
    #[inline]
    pub fn new(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> UnionIter<'a> {
        UnionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator<u32> for UnionIter<'a> {
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

pub struct IntersectionIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

impl<'a> IntersectionIter<'a> {
    #[inline]
    pub fn new(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> IntersectionIter<'a> {
        IntersectionIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator<u32> for IntersectionIter<'a> {
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

pub struct DifferenceIter<'a> {
    current1: Option<u32>,
    current2: Option<u32>,
    iter1: Iter<'a>,
    iter2: Iter<'a>,
}

impl<'a> DifferenceIter<'a> {
    #[inline]
    pub fn new(mut iter1: Iter<'a>, mut iter2: Iter<'a>) -> DifferenceIter<'a> {
        DifferenceIter {
            current1: iter1.next(),
            current2: iter2.next(),
            iter1: iter1,
            iter2: iter2,
        }
    }
}

impl<'a> Iterator<u32> for DifferenceIter<'a> {
    fn next(&mut self) -> Option<u32> {
        match (self.current1, self.current2) {
            (None, _) | (_, None) => None,
            (val1, val2) if val1 < val2 => { self.current1 = self.iter1.next(); val1 },
            (val1, val2) if val1 > val2 => { self.current2 = self.iter2.next(); self.next() },
            (val1, val2) if val1 == val2 => {
                self.current1 = self.iter1.next();
                self.current2 = self.iter2.next();
                self.next()
            },
            _ => panic!("Should not be possible to get here"),
        }
    }
}
