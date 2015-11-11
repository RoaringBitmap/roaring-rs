extern crate roaring;
use roaring::RoaringBitmap;

use std::iter::{ Iterator, DoubleEndedIterator };

struct Alternator<T> {
    iter: T,
    fwd: bool,
}

impl<T> Iterator for Alternator<T> where T: DoubleEndedIterator {
    type Item = <T as Iterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.fwd = !self.fwd;
        if self.fwd {
            self.iter.next()
        } else {
            self.iter.next_back()
        }
    }
}

fn alt<I, T: Iterator<Item=I>>(iter: T) -> Alternator<T> {
    Alternator {
        iter: iter,
        fwd: false,
    }
}

#[test]
fn mini() {
    let original: RoaringBitmap<u32> = (0..5u32).collect();
    let clone: RoaringBitmap<u32> = alt(original.iter()).collect();

    assert_eq!(clone, original);
}

#[test]
fn array() {
    let original: RoaringBitmap<u32> = (0..2000u32).collect();
    let clone: RoaringBitmap<u32> = alt(original.iter()).collect();

    assert_eq!(clone, original);
}

#[test]
fn bitmap() {
    let original: RoaringBitmap<u32> = (0..6000u32).collect();
    let clone: RoaringBitmap<u32> = alt(original.iter()).collect();

    assert_eq!(clone, original);
}

#[test]
fn arrays() {
    let original: RoaringBitmap<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let clone: RoaringBitmap<u32> = alt(original.iter()).collect();

    assert_eq!(clone, original);
}

#[test]
fn bitmaps() {
    let original: RoaringBitmap<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let clone: RoaringBitmap<u32> = alt(original.iter()).collect();

    assert_eq!(clone, original);
}

#[test]
fn mini_vs_vec() {
    let original: Vec<u32> = (0..5u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = alt(bitmap.iter()).collect();
    let alternated: Vec<u32> = alt(original.iter()).map(|&i| i).collect();

    assert_eq!(clone, alternated);
}

#[test]
fn array_vs_vec() {
    let original: Vec<u32> = (0..2000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = alt(bitmap.iter()).collect();
    let alternated: Vec<u32> = alt(original.iter()).map(|&i| i).collect();

    assert_eq!(clone, alternated);
}

#[test]
fn bitmap_vs_vec() {
    let original: Vec<u32> = (0..6000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = alt(bitmap.iter()).collect();
    let alternated: Vec<u32> = alt(original.iter()).map(|&i| i).collect();

    assert_eq!(clone, alternated);
}

#[test]
fn arrays_vs_vec() {
    let original: Vec<u32> = (0..2000u32).chain(1000000..1002000u32).chain(2000000..2001000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = alt(bitmap.iter()).collect();
    let alternated: Vec<u32> = alt(original.iter()).map(|&i| i).collect();

    assert_eq!(clone, alternated);
}

#[test]
fn bitmaps_vs_vec() {
    let original: Vec<u32> = (0..6000u32).chain(1000000..1012000u32).chain(2000000..2010000u32).collect();
    let bitmap: RoaringBitmap<u32> = original.iter().collect();
    let clone: Vec<u32> = alt(bitmap.iter()).collect();
    let alternated: Vec<u32> = alt(original.iter()).map(|&i| i).collect();

    assert_eq!(clone, alternated);
}
