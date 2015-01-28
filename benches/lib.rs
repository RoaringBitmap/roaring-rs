#![feature(slicing_syntax)]

extern crate test;
extern crate roaring;

use std::{ u32 };
use test::Bencher;

use roaring::RoaringBitmap;

#[bench]
fn create(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
        bitmap
    })
}

#[bench]
fn insert1(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
        bitmap.insert(1);
        bitmap
    })
}

#[bench]
fn insert2(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap: RoaringBitmap<u32> = RoaringBitmap::new();
        bitmap.insert(1);
        bitmap.insert(2);
        bitmap
    })
}

