use std::{ u16 };
use std::slice::Iter;

use util::Either;
use util::Either::{ Left, Right };
use container::{ Container, ContainerIter };

pub struct RoaringIterator<'a> {
    inner_iter: Option<(u16, ContainerIter<'a>)>,
    container_iter: Box<Iter<'a, Container>>,
}

fn calc(key: u16, value: u16) -> u32 {
    ((key as u32) << u16::BITS) + (value as u32)
}

fn find_next_iter<'a>(container_iter: &mut Iter<'a, Container>) -> Option<(u16, ContainerIter<'a>)> {
    match container_iter.next() {
        Some(container) => Some((container.key(), container.iter())),
        None => None
    }
}

impl<'a> RoaringIterator<'a> {
    pub fn new(mut container_iter: Box<Iter<'a, Container>>) -> RoaringIterator<'a> {
        RoaringIterator {
            inner_iter: find_next_iter(&mut *container_iter),
            container_iter: container_iter
        }
    }

    fn choose_next(&mut self) -> Option<Either<u32, Option<(u16, ContainerIter<'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match *iter {
                ContainerIter::ArrayIter(ref mut iter) => match iter.next() {
                    Some(value) => Left(calc(key, *value)),
                    None => Right(find_next_iter(&mut *self.container_iter)),
                },
                ContainerIter::BitmapIter(ref mut iter) => match iter.next() {
                    Some(value) => Left(calc(key, value)),
                    None => Right(find_next_iter(&mut *self.container_iter)),
                },
            }),
            None => None,
        }
    }
}

impl<'a> Iterator<u32> for RoaringIterator<'a> {
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
