use std::{ u16, u32 };
use std::slice::Iter;

use util::Either;
use util::Either::{ Left, Right };
use store::Store::{ Array, Bitmap };
use container::{ Container, BitmapIter };

pub struct RoaringIterator<'a> {
    inner_iter: Option<(u16, Either<Iter<'a, u16>, BitmapIter<'a>>)>,
    container_iter: &'a mut Iter<'a, Container>,
}

impl<'a> RoaringIterator<'a> {
    pub fn new(container_iter: &'a mut Iter<'a, Container>) -> RoaringIterator<'a> {
        RoaringIterator {
            inner_iter: next_iter(container_iter),
            container_iter: container_iter
        }
    }
}

impl<'a> Iterator<u32> for RoaringIterator<'a> {
    fn next(&mut self) -> Option<u32> {
        match self.inner_iter {
            Some((key, iter)) => match iter {
                Left(iter) => self.do_next(key, &mut iter),
                Right(iter) => self.do_next(key, &mut iter),
            },
            None => None,
        }
    }
}

fn next_iter<'a>(container_iter: &'a mut Iter<'a, Container>) -> Option<(u16, Either<Iter<'a, u16>, BitmapIter<'a>>)> {
    match container_iter.next() {
        Some(container) => Some((container.key(), container.iter())),
        None => None
    }
}

impl<'a> RoaringIterator<'a> {
    fn do_next<T: Iterator<&'a u16>>(&mut self, key: u16, iter: &mut T) -> Option<u32> {
        match iter.next() {
            Some(value) => Some(((key as u32) << u16::BITS) + (*value as u32)),
            None => {
                self.inner_iter = next_iter(self.container_iter);
                self.next()
            },
        }
    }
}

