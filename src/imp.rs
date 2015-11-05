use std::iter::{ IntoIterator };
use std::slice;
use std::cmp::Ordering;

use num::traits::{ Zero, Bounded };

use iter::{ self, Iter, UnionIter, IntersectionIter, DifferenceIter, SymmetricDifferenceIter };
use container::{ Container };
use util::{ self, Halveable, ExtInt };

use RoaringBitmap as RB;

#[inline]
pub fn new<Size: ExtInt + Halveable>() -> RB<Size> {
    RB { containers: Vec::new() }
}

pub fn insert<Size: ExtInt + Halveable>(this: &mut RB<Size>, value: Size) -> bool {
    let (key, index) = value.split();
    let container = match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => &mut this.containers[loc],
        Err(loc) => {
            this.containers.insert(loc, Container::new(key));
            &mut this.containers[loc]
        },
    };
    container.insert(index)
}

pub fn remove<Size: ExtInt + Halveable>(this: &mut RB<Size>, value: Size) -> bool {
    let (key, index) = value.split();
    match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => {
            if this.containers[loc].remove(index) {
                if this.containers[loc].len() == Zero::zero() {
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

pub fn contains<Size: ExtInt + Halveable>(this: &RB<Size>, value: Size) -> bool {
    let (key, index) = value.split();
    match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
        Ok(loc) => this.containers[loc].contains(index),
        Err(_) => false,
    }
}

#[inline]
pub fn clear<Size: ExtInt + Halveable>(this: &mut RB<Size>) {
    this.containers.clear();
}

#[inline]
pub fn is_empty<Size: ExtInt + Halveable>(this: &RB<Size>) -> bool {
    this.containers.is_empty()
}

pub fn len<Size: ExtInt + Halveable>(this: &RB<Size>) -> Size {
    this.containers
        .iter()
        .map(|container| container.len())
        .fold(Zero::zero(), |sum: Size, len| sum + util::cast(len))
}

#[inline]
pub fn iter<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>) -> Iter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    iter::new(this.containers.iter())
}

fn pairs<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>, other: &'a RB<Size>) -> Pairs<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    Pairs::new(this.containers.iter(), other.containers.iter())
}

pub fn is_disjoint<Size: ExtInt + Halveable>(this: &RB<Size>, other: &RB<Size>) -> bool {
    pairs(this, other)
        .filter(|&(c1, c2)| c1.is_some() && c2.is_some())
        .all(|(c1, c2)| c1.unwrap().is_disjoint(c2.unwrap()))
}

pub fn is_subset<Size: ExtInt + Halveable>(this: &RB<Size>, other: &RB<Size>) -> bool {
    for pair in pairs(this, other) {
        match pair {
            (None, _) => (),
            (_, None) => { return false; },
            (Some(c1), Some(c2)) => if !c1.is_subset(c2) { return false; },
        }
    }
    true
}

pub fn is_subset_opt<Size: ExtInt + Halveable>(this: &RB<Size>, other: &RB<Size>) -> bool {
    let tv = &this.containers;
    let ov = &other.containers;
    let tlen = tv.len();
    let olen = ov.len();
    if tlen > olen { return false; }
    let mut ti = 0;
    let mut oi = 0;
    loop {
        let tc = &tv[ti];
        let oc = &ov[oi];
        match tc.key().cmp(&oc.key()) {
            Ordering::Less => { return false; },
            Ordering::Equal => {
                if !tc.is_subset(&oc) { return false; }
                ti += 1;
                if ti >= tlen { return true; }
            },
            Ordering::Greater => (),
        }
        oi += 1;
        if oi >= olen { return false }
    }
}

#[inline]
pub fn is_superset<Size: ExtInt + Halveable>(this: &RB<Size>, other: &RB<Size>) -> bool {
    other.is_subset(this)
}

#[inline]
pub fn union<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>, other: &'a RB<Size>) -> UnionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    iter::union::new(this.iter(), other.iter())
}

#[inline]
pub fn intersection<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>, other: &'a RB<Size>) -> IntersectionIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    iter::intersection::new(this.iter(), other.iter())
}

#[inline]
pub fn difference<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>, other: &'a RB<Size>) -> DifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    iter::difference::new(this.iter(), other.iter())
}

#[inline]
pub fn symmetric_difference<'a, Size: ExtInt + Halveable>(this: &'a RB<Size>, other: &'a RB<Size>) -> SymmetricDifferenceIter<'a, Size> where <Size as Halveable>::HalfSize : 'a {
    iter::symmetric_difference::new(this.iter(), other.iter())
}

#[inline]
pub fn union_with<Size: ExtInt + Halveable>(this: &mut RB<Size>, other: &RB<Size>) {
    for container in &other.containers {
        let key = container.key();
        match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => this.containers[loc].union_with(container),
        };
    }
}

#[inline]
pub fn intersect_with<Size: ExtInt + Halveable>(this: &mut RB<Size>, other: &RB<Size>) {
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
pub fn difference_with<Size: ExtInt + Halveable>(this: &mut RB<Size>, other: &RB<Size>) {
    let mut index = 0;
    while index < this.containers.len() {
        let key = this.containers[index].key();
        match other.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Ok(loc) => {
                this.containers[index].difference_with(&other.containers[loc]);
                if this.containers[index].len() == Zero::zero() {
                    this.containers.remove(index);
                } else {
                    index += 1;
                }
            },
            _ => {
                index += 1;
            }
        };
    }
}

#[inline]
pub fn symmetric_difference_with<Size: ExtInt + Halveable>(this: &mut RB<Size>, other: &RB<Size>) {
    for container in &other.containers {
        let key = container.key();
        match this.containers.binary_search_by(|container| container.key().cmp(&key)) {
            Err(loc) => this.containers.insert(loc, (*container).clone()),
            Ok(loc) => {
                this.containers[loc].symmetric_difference_with(container);
                if this.containers[loc].len() == Zero::zero() {
                    this.containers.remove(loc);
                }
            }
        };
    }
}

#[inline]
pub fn from_iter<Size: ExtInt + Halveable, I: IntoIterator<Item = Size>>(iterator: I) -> RB<Size> {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn from_iter_ref<'a, Size: ExtInt + Halveable + 'a, I: IntoIterator<Item = &'a Size>>(iterator: I) -> RB<Size> {
    let mut rb = new();
    rb.extend(iterator);
    rb
}

#[inline]
pub fn extend<Size: ExtInt + Halveable, I: IntoIterator<Item = Size>>(this: &mut RB<Size>, iterator: I) {
    for value in iterator {
        this.insert(value);
    }
}

#[inline]
pub fn extend_ref<'a, Size: ExtInt + Halveable + 'a, I: IntoIterator<Item = &'a Size>>(this: &mut RB<Size>, iterator: I) {
    for value in iterator {
        this.insert(*value);
    }
}

pub fn min<Size: ExtInt + Halveable>(this: &RB<Size>) -> Size {
    match this.containers.first() {
        Some(ref head) => Halveable::join(head.key(), head.min()),
        None => Bounded::min_value(),
    }
}

pub fn max<Size: ExtInt + Halveable>(this: &RB<Size>) -> Size {
    match this.containers.last() {
        Some(ref tail) => Halveable::join(tail.key(), tail.max()),
        None => Bounded::max_value(),
    }
}

struct Pairs<'a, Size: ExtInt + Halveable + 'a> where <Size as Halveable>::HalfSize : 'a {
    iter1: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
    iter2: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>,
    current1: Option<&'a Container<<Size as Halveable>::HalfSize>>,
    current2: Option<&'a Container<<Size as Halveable>::HalfSize>>,
}

impl<'a, Size: ExtInt + Halveable> Pairs<'a, Size> {
    fn new(mut iter1: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>, mut iter2: slice::Iter<'a, Container<<Size as Halveable>::HalfSize>>) -> Pairs<'a, Size> {
        let (current1, current2) = (iter1.next(), iter2.next());
        Pairs {
            iter1: iter1,
            iter2: iter2,
            current1: current1,
            current2: current2,
        }
    }
}

impl<'a, Size: ExtInt + Halveable> Iterator for Pairs<'a, Size> {
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
