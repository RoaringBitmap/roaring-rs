use std::{ u16 };
use std::slice;

use util::Either;
use util::Either::{ Left, Right };
use container::{ Container };

pub struct RoaringIterator<'a> {
    inner_iter: Option<(u16, Box<Iterator<u16> + 'a>)>,
    container_iter: Box<slice::Iter<'a, Container>>,
}

fn calc(key: u16, value: u16) -> u32 {
    ((key as u32) << u16::BITS) + (value as u32)
}

fn next_iter<'a>(container_iter: &mut slice::Iter<'a, Container>) -> Option<(u16, Box<Iterator<u16> + 'a>)> {
    container_iter.next().map(|container| (container.key(), container.iter()))
}

impl<'a> RoaringIterator<'a> {
    pub fn new(mut container_iter: Box<slice::Iter<'a, Container>>) -> RoaringIterator<'a> {
        RoaringIterator {
            inner_iter: next_iter(&mut *container_iter),
            container_iter: container_iter
        }
    }

    fn choose_next(&mut self) -> Option<Either<u32, Option<(u16, Box<Iterator<u16> + 'a>)>>> {
        match self.inner_iter {
            Some((key, ref mut iter)) => Some(match iter.next() {
                Some(value) => Left(calc(key, value)),
                None => Right(next_iter(&mut *self.container_iter)),
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
