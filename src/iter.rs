use std::slice;

use util::{ Either, ExtInt, Halveable };
use util::Either::{ Left, Right };
use container::{ Container };

/// An iterator for `RoaringBitmap`.
pub struct Iter<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    fwd_inner_iter: Option<(<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)>,
    bwd_inner_iter: Option<(<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)>,
    container_iter: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
}

#[inline]
fn next_iter<'a, Size: ExtInt + Halveable + 'a>(container_iter: &mut slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Option<(<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)> {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

#[inline]
fn next_back_iter<'a, Size: ExtInt + Halveable + 'a>(container_iter: &mut slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Option<(<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)> {
    container_iter.next_back().map(|container| (container.key(), container.iter()))
}

#[inline]
pub fn new<'a, Size: ExtInt + Halveable + 'a>(mut container_iter: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Iter<'a, Size> {
    Iter {
        fwd_inner_iter: next_iter::<'a, Size>(&mut container_iter),
        bwd_inner_iter: next_back_iter::<'a, Size>(&mut container_iter),
        container_iter: container_iter
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    #[inline]
    fn choose_next(&mut self) -> Option<Either<Size, (<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)>> {
        let container_iter = &mut self.container_iter;
        let fwd_inner_iter = self.fwd_inner_iter.as_mut();
        let bwd_inner_iter = self.bwd_inner_iter.as_mut();

        fwd_inner_iter
            .and_then(|&mut (key, ref mut iter)|
                iter.next().map(|value| Left(Halveable::join(key, value)))
                    .or_else(|| next_iter::<'a, Size>(container_iter).map(|next_iter| Right(next_iter))))
            .or_else(||
                bwd_inner_iter
                    .and_then(|&mut (key, ref mut iter)|
                        iter.next().map(|value| Left(Halveable::join(key, value)))))
    }

    #[inline]
    fn choose_next_back(&mut self) -> Option<Either<Size, (<Size as Halveable>::HalfSize, Box<DoubleEndedIterator<Item = <Size as Halveable>::HalfSize> + 'a>)>> {
        let container_iter = &mut self.container_iter;
        let fwd_inner_iter = self.fwd_inner_iter.as_mut();
        let bwd_inner_iter = self.bwd_inner_iter.as_mut();

        bwd_inner_iter
            .and_then(|&mut (key, ref mut iter)|
                iter.next_back().map(|value| Left(Halveable::join(key, value)))
                    .or_else(|| next_back_iter::<'a, Size>(container_iter).map(|next_iter| Right(next_iter))))
            .or_else(||
                fwd_inner_iter
                    .and_then(|&mut (key, ref mut iter)|
                        iter.next_back().map(|value| Left(Halveable::join(key, value)))))
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> Iterator for Iter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    type Item = Size;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let next = self.container_iter.clone().map(|container| container.len() as usize).fold(0, |acc, len| acc + len);
        let mut hint = (next, Some(next));
        match self.fwd_inner_iter {
            Some((_, ref iter)) => match iter.size_hint() {
                (min, max) => hint = (hint.0 + min, max.and_then(|m| hint.1.map(|h| h + m))),
            },
            None => {
            },
        };
        match self.bwd_inner_iter {
            Some((_, ref iter)) => match iter.size_hint() {
                (min, max) => hint = (hint.0 + min, max.and_then(|m| hint.1.map(|h| h + m))),
            },
            None => {
            },
        };
        hint
    }

    fn next(&mut self) -> Option<Size> {
        match self.choose_next() {
            None => None,
            Some(Left(val)) => Some(val),
            Some(Right(new_iter)) => {
                self.fwd_inner_iter = Some(new_iter);
                self.next()
            },
        }
    }
}

impl<'a, Size: ExtInt + Halveable + 'a> DoubleEndedIterator for Iter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    fn next_back(&mut self) -> Option<Size> {
        match self.choose_next_back() {
            None => None,
            Some(Left(val)) => Some(val),
            Some(Right(new_iter)) => {
                self.bwd_inner_iter = Some(new_iter);
                self.next_back()
            },
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
