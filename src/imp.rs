use std::cmp::Ord;
use std::num::Int;
use std::slice;

use iter;
use iter::{ Iter, UnionIter, IntersectionIter, DifferenceIter, SymmetricDifferenceIter };
use container::Container;
use util;
use util::{ Halveable, ExtInt };

type RB<Size> = ::RoaringBitmap<Size>;

#[inline]
pub fn new<Size>() -> RB<Size> where Size: ExtInt {
    RB { containers: Vec::new() }
}

pub fn insert<Size>(this: &mut RB<Size>, value: Size) -> bool where Size: ExtInt {
    let (key, index) = calc_loc(value);
    let container = match this.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => &mut this.containers[loc],
        Err(loc) => {
            this.containers.insert(loc, Container::new(key));
            &mut this.containers[loc]
        },
    };
    container.insert(index)
}

pub fn remove<Size>(this: &mut RB<Size>, value: Size) -> bool where Size: ExtInt {
    let (key, index) = calc_loc(value);
    match this.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => {
            if this.containers[loc].remove(index) {
                if this.containers[loc].len() == Int::zero() {
                    this.containers.remove(loc);
                }
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn contains<Size>(this: &RB<Size>, value: Size) -> bool where Size: ExtInt {
    let (key, index) = calc_loc(value);
    match this.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => this.containers[loc].contains(index),
        Err(_) => false,
    }
}

#[inline]
pub fn clear<Size>(this: &mut RB<Size>) where Size: ExtInt {
    this.containers.clear();
}

#[inline]
pub fn is_empty<Size>(this: &RB<Size>) -> bool where Size: ExtInt {
    this.containers.is_empty()
}

pub fn len<Size>(this: &RB<Size>) -> Size where Size: ExtInt {
    this.containers
        .iter()
        .map(|container| container.len())
        .fold(Int::zero(), |sum: Size, len| sum + util::cast(len))
}

#[inline]
pub fn iter<'a, Size>(this: &'a RB<Size>) -> Iter<'a, Size> where Size: ExtInt {
    iter::new(this.containers.iter())
}

fn pairs<'a, Size>(this: &'a RB<Size>, other: &'a RB<Size>) -> Pairs<'a, Size> where Size: ExtInt {
    Pairs::new(this.containers.iter(), other.containers.iter())
}

pub fn is_disjoint<Size>(this: &RB<Size>, other: &RB<Size>) -> bool where Size: ExtInt {
    pairs(this, other)
        .filter(|&(c1, c2)| c1.is_some() && c2.is_some())
        .all(|(c1, c2)| c1.unwrap().is_disjoint(c2.unwrap()))
}

pub fn is_subset<Size>(this: &RB<Size>, other: &RB<Size>) -> bool where Size: ExtInt {
    pairs(this, other).all(|pairs| match pairs {
        (None, _) => return true,
        (_, None) => return false,
        (Some(c1), Some(c2)) => c1.is_subset(c2),
    })
}

#[inline]
pub fn is_superset<Size>(this: &RB<Size>, other: &RB<Size>) -> bool where Size: ExtInt {
    other.is_subset(this)
}

#[inline]
pub fn union<'a, Size>(this: &'a RB<Size>, other: &'a RB<Size>) -> UnionIter<'a, Size> where Size: ExtInt {
    iter::union::new(this.iter(), other.iter())
}

#[inline]
pub fn intersection<'a, Size>(this: &'a RB<Size>, other: &'a RB<Size>) -> IntersectionIter<'a, Size> where Size: ExtInt {
    iter::intersection::new(this.iter(), other.iter())
}

#[inline]
pub fn difference<'a, Size>(this: &'a RB<Size>, other: &'a RB<Size>) -> DifferenceIter<'a, Size> where Size: ExtInt {
    iter::difference::new(this.iter(), other.iter())
}

#[inline]
pub fn symmetric_difference<'a, Size>(this: &'a RB<Size>, other: &'a RB<Size>) -> SymmetricDifferenceIter<'a, Size> where Size: ExtInt {
    iter::symmetric_difference::new(this.iter(), other.iter())
}

#[inline]
pub fn union_with<Size>(this: &mut RB<Size>, other: &RB<Size>) where Size: ExtInt {
    for container in other.containers.iter() {
        let key = container.key();
        match this.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => this.containers[loc].union_with(container),
        };
    }
}

#[inline]
pub fn intersect_with<Size>(this: &mut RB<Size>, other: &RB<Size>) where Size: ExtInt {
    let mut index = 0;
    while index < this.containers.len() {
        let key = this.containers[index].key();
        match other.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
            Err(_) => {
                this.containers.remove(index);
            },
            Ok(loc) => {
                this.containers[index].intersect_with(&other.containers[loc]);
                index += 1;
            },
        };
    }
}

#[inline]
pub fn difference_with<Size>(this: &mut RB<Size>, other: &RB<Size>) where Size: ExtInt {
    for index in range(0, this.containers.len()) {
        let key = this.containers[index].key();
        match other.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => {
                this.containers[index].difference_with(&other.containers[loc]);
                if this.containers[index].len() == Int::zero() {
                    this.containers.remove(index);
                }
            },
            _ => (),
        };
    }
}

#[inline]
pub fn symmetric_difference_with<Size>(this: &mut RB<Size>, other: &RB<Size>) where Size: ExtInt {
    for container in other.containers.iter() {
        let key = container.key();
        match this.containers.as_slice().binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => {
                this.containers[loc].symmetric_difference_with(container);
                if this.containers[loc].len() == Int::zero() {
                    this.containers.remove(loc);
                }
            }
        };
    }
}

#[inline]
pub fn from_iter<Size, I: Iterator<Item = Size>>(iterator: I) -> RB<Size> where Size: ExtInt {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn from_iter_ref<'a, Size, I: Iterator<Item = &'a Size>>(iterator: I) -> RB<Size> where Size: ExtInt + 'a {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn extend<Size, I: Iterator<Item = Size>>(this: &mut RB<Size>, mut iterator: I) where Size: ExtInt {
    for value in iterator {
        this.insert(value);
    }
}

#[inline]
pub fn extend_ref<'a, Size, I: Iterator<Item = &'a Size>>(this: &mut RB<Size>, mut iterator: I) where Size: ExtInt + 'a {
    for value in iterator {
        this.insert(*value);
    }
}

pub fn min<Size>(this: &RB<Size>) -> Size where Size: ExtInt {
    match this.containers[] {
        [ref head, ..] => calc(head.key(), head.min()),
        [] => Int::min_value(),
    }
}

pub fn max<Size>(this: &RB<Size>) -> Size where Size: ExtInt {
    match this.containers[] {
        [.., ref tail] => calc(tail.key(), tail.max()),
        [] => Int::max_value(),
    }
}

#[inline]
fn calc<Size>(key: <Size as Halveable>::HalfSize, value: <Size as Halveable>::HalfSize) -> Size where Size: ExtInt {
    let bits = util::bits::<<Size as Halveable>::HalfSize>();
    (util::cast::<<Size as Halveable>::HalfSize, Size>(key) << bits) + util::cast(value)
}

#[inline]
fn calc_loc<Size>(index: Size) -> (<Size as Halveable>::HalfSize, <Size as Halveable>::HalfSize) where Size: ExtInt {
    let bits = util::bits::<<Size as Halveable>::HalfSize>();
    (util::cast::<Size, <Size as Halveable>::HalfSize>(index >> bits), util::cast(index))
}

struct Pairs<'a, Size> where Size: ExtInt {
    iter1: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
    iter2: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
    current1: Option<&'a Container<<Size as Halveable>::HalfSize>>,
    current2: Option<&'a Container<<Size as Halveable>::HalfSize>>,
}

impl<'a, Size> Pairs<'a, Size> where Size: ExtInt {
    fn new(mut iter1: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>, mut iter2: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Pairs<'a, Size> {
        Pairs {
            iter1: iter1,
            iter2: iter2,
            current1: iter1.next(),
            current2: iter2.next(),
        }
    }
}

impl<'a, Size> Iterator for Pairs<'a, Size> where Size: ExtInt {
    type Item = (Option<&'a Container<<Size as Halveable>::HalfSize>>, Option<&'a Container<<Size as Halveable>::HalfSize>>);

    fn next(&mut self) -> Option<(Option<&'a Container<<Size as Halveable>::HalfSize>>, Option<&'a Container<<Size as Halveable>::HalfSize>>)> {
        match (self.current1, self.current2) {
            (None, None) => None,
            (Some(c1), None) => {
                self.current1 = self.iter1.next();
                Some((Some(c1), None))
            },
            (None, Some(c2)) => {
                self.current2 = self.iter2.next();
                Some((None, Some(c2)))
            },
            (Some(c1), Some(c2)) => match (c1.key(), c2.key()) {
                (key1, key2) if key1 == key2 => {
                    self.current1 = self.iter1.next();
                    self.current2 = self.iter2.next();
                    Some((Some(c1), Some(c2)))
                },
                (key1, key2) if key1 < key2 => {
                    self.current1 = self.iter1.next();
                    Some((Some(c1), None))
                },
                (key1, key2) if key1 > key2 => {
                    self.current2 = self.iter2.next();
                    Some((None, Some(c2)))
                },
                (_, _) => panic!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{ u16, u32 };
    use super::{ calc_loc };

    #[test]
    fn test_calc_location() {
        assert_eq!((0u16, 0u16), calc_loc(0u32));
        assert_eq!((0u16, 1u16), calc_loc(1u32));
        assert_eq!((0u16, u16::MAX - 1u16), calc_loc(u16::MAX as u32 - 1u32));
        assert_eq!((0u16, u16::MAX), calc_loc(u16::MAX as u32));
        assert_eq!((1u16, 0u16), calc_loc(u16::MAX as u32 + 1u32));
        assert_eq!((1u16, 1u16), calc_loc(u16::MAX as u32 + 2u32));
        assert_eq!((u16::MAX, u16::MAX - 1u16), calc_loc(u32::MAX - 1u32));
        assert_eq!((u16::MAX, u16::MAX), calc_loc(u32::MAX));
    }
}
