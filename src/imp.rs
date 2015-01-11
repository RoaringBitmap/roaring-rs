use std::{ u16, u32 };
use std::slice;

use iter;
use iter::{ Iter, UnionIter, IntersectionIter, DifferenceIter, SymmetricDifferenceIter };
use container::Container;

type RB = ::RoaringBitmap;

#[inline]
pub fn new() -> RB {
    RB { containers: Vec::new() }
}

pub fn insert(this: &mut RB, value: u32) -> bool {
    let (key, index) = calc_loc(value);
    let container = match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => &mut this.containers[loc],
        Err(loc) => {
            this.containers.insert(loc, Container::new(key));
            &mut this.containers[loc]
        },
    };
    container.insert(index)
}

pub fn remove(this: &mut RB, value: u32) -> bool {
    let (key, index) = calc_loc(value);
    match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => {
            if this.containers[loc].remove(index) {
                if this.containers[loc].len() == 0 {
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

pub fn contains(this: &RB, value: u32) -> bool {
    let (key, index) = calc_loc(value);
    match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => this.containers[loc].contains(index),
        Err(_) => false,
    }
}

#[inline]
pub fn clear(this: &mut RB) {
    this.containers.clear();
}

#[inline]
pub fn is_empty(this: &RB) -> bool {
    this.containers.is_empty()
}

pub fn len(this: &RB) -> usize {
    this.containers
        .iter()
        .map(|container| container.len() as usize)
        .fold(0, |sum, len| sum + len)
}

#[inline]
pub fn iter<'a>(this: &'a RB) -> Iter<'a> {
    iter::new(this.containers.iter())
}

fn pairs<'a>(this: &'a RB, other: &'a RB) -> Pairs<'a> {
    Pairs::new(this.containers.iter(), other.containers.iter())
}

pub fn is_disjoint(this: &RB, other: &RB) -> bool {
    pairs(this, other)
        .filter(|&(c1, c2)| c1.is_some() && c2.is_some())
        .all(|(c1, c2)| c1.unwrap().is_disjoint(c2.unwrap()))
}

pub fn is_subset(this: &RB, other: &RB) -> bool {
    pairs(this, other).all(|pairs| match pairs {
        (None, _) => return true,
        (_, None) => return false,
        (Some(c1), Some(c2)) => c1.is_subset(c2),
    })
}

#[inline]
pub fn is_superset(this: &RB, other: &RB) -> bool {
    other.is_subset(this)
}

#[inline]
pub fn union<'a>(this: &'a RB, other: &'a RB) -> UnionIter<'a> {
    iter::union::new(this.iter(), other.iter())
}

#[inline]
pub fn intersection<'a>(this: &'a RB, other: &'a RB) -> IntersectionIter<'a> {
    iter::intersection::new(this.iter(), other.iter())
}

#[inline]
pub fn difference<'a>(this: &'a RB, other: &'a RB) -> DifferenceIter<'a> {
    iter::difference::new(this.iter(), other.iter())
}

#[inline]
pub fn symmetric_difference<'a>(this: &'a RB, other: &'a RB) -> SymmetricDifferenceIter<'a> {
    iter::symmetric_difference::new(this.iter(), other.iter())
}

#[inline]
pub fn union_with(this: &mut RB, other: &RB) {
    for container in other.containers.iter() {
        let key = container.key();
        match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => this.containers[loc].union_with(container),
        };
    }
}

#[inline]
pub fn intersect_with(this: &mut RB, other: &RB) {
    let mut index = 0;
    while index < this.containers.len() {
        let key = this.containers[index].key();
        match other.containers.binary_search_by(|container| container.key().cmp(&key)) {
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
pub fn difference_with(this: &mut RB, other: &RB) {
    for index in 0..this.containers.len() {
        let key = this.containers[index].key();
        match other.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => {
                this.containers[index].difference_with(&other.containers[loc]);
                if this.containers[index].len() == 0 {
                    this.containers.remove(index);
                }
            },
            _ => (),
        };
    }
}

#[inline]
pub fn symmetric_difference_with(this: &mut RB, other: &RB) {
    for container in other.containers.iter() {
        let key = container.key();
        match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => {
                this.containers[loc].symmetric_difference_with(container);
                if this.containers[loc].len() == 0 {
                    this.containers.remove(loc);
                }
            }
        };
    }
}

#[inline]
pub fn from_iter<I: Iterator<Item = u32>>(iterator: I) -> RB {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn from_iter_ref<'a, I: Iterator<Item = &'a u32>>(iterator: I) -> RB {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn extend<I: Iterator<Item = u32>>(this: &mut RB, mut iterator: I) {
    for value in iterator {
        this.insert(value);
    }
}

#[inline]
pub fn extend_ref<'a, I: Iterator<Item = &'a u32>>(this: &mut RB, mut iterator: I) {
    for value in iterator {
        this.insert(*value);
    }
}

pub fn min(this: &RB) -> u32 {
    match &this.containers[] {
        [ref head, ..] => calc(head.key(), head.min()),
        [] => u32::MIN,
    }
}

pub fn max(this: &RB) -> u32 {
    match &this.containers[] {
        [.., ref tail] => calc(tail.key(), tail.max()),
        [] => u32::MAX,
    }
}

#[inline]
fn calc(key: u16, value: u16) -> u32 {
    ((key as u32) << u16::BITS) + (value as u32)
}

#[inline]
fn calc_loc(index: u32) -> (u16, u16) { ((index >> u16::BITS) as u16, index as u16) }

struct Pairs<'a> {
    iter1: slice::Iter<'a, Container<u16>>,
    iter2: slice::Iter<'a, Container<u16>>,
    current1: Option<&'a Container<u16>>,
    current2: Option<&'a Container<u16>>,
}

impl<'a> Pairs<'a> {
    fn new(mut iter1: slice::Iter<'a, Container<u16>>, mut iter2: slice::Iter<'a, Container<u16>>) -> Pairs<'a> {
        Pairs {
            iter1: iter1,
            iter2: iter2,
            current1: iter1.next(),
            current2: iter2.next(),
        }
    }
}

impl<'a> Iterator for Pairs<'a> {
    type Item = (Option<&'a Container<u16>>, Option<&'a Container<u16>>);

    fn next(&mut self) -> Option<(Option<&'a Container<u16>>, Option<&'a Container<u16>>)> {
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
        assert_eq!((0, 0), calc_loc(0));
        assert_eq!((0, 1), calc_loc(1));
        assert_eq!((0, u16::MAX - 1), calc_loc(u16::MAX as u32 - 1));
        assert_eq!((0, u16::MAX), calc_loc(u16::MAX as u32));
        assert_eq!((1, 0), calc_loc(u16::MAX as u32 + 1));
        assert_eq!((1, 1), calc_loc(u16::MAX as u32 + 2));
        assert_eq!((u16::MAX, u16::MAX - 1), calc_loc(u32::MAX - 1));
        assert_eq!((u16::MAX, u16::MAX), calc_loc(u32::MAX));
    }
}
