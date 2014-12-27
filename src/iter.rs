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
            inner_iter: match container_iter.next() {
                Some(container) => Some((container.key(), container.iter())),
                None => None
            },
            container_iter: container_iter
        }
    }
}

impl<'a> Iterator<u32> for RoaringIterator<'a> {
    fn next(&mut self) -> Option<u32> {
        match (match self.inner_iter {
            Some((key, ref mut iter)) => match *iter {
                Left(ref mut iter) => match iter.next() {
                    Some(value) => (Some(((key as u32) << u16::BITS) + (*value as u32)), None),
                    None => {
                        (None, match self.container_iter.next() {
                            Some(container) => Some((container.key(), container.iter())),
                            None => None
                        })
                    },
                },
                Right(ref mut iter) => match iter.next() {
                    Some(value) => (Some(((key as u32) << u16::BITS) + (value as u32)), None),
                    None => {
                        (None, match self.container_iter.next() {
                            Some(container) => Some((container.key(), container.iter())),
                            None => None
                        })
                    },
                },
            },
            None => (None, None),
        }) {
            (None, None) => None,
            (None, new_iter) => {
                self.inner_iter = new_iter;
                self.next()
            },
            (val, None) => val,
            _ => panic!(),
        }
    }
}
