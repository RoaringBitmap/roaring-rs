#![feature(test)]

extern crate test;
extern crate roaring;

use test::Bencher;

use roaring::RoaringBitmap;

#[bench]
fn create(b: &mut Bencher) {
    b.iter(|| {
        RoaringBitmap::new();
    })
}

#[bench]
fn insert1(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap = RoaringBitmap::new();
        bitmap.insert(1);
        bitmap
    })
}

#[bench]
fn insert2(b: &mut Bencher) {
    b.iter(|| {
        let mut bitmap = RoaringBitmap::new();
        bitmap.insert(1);
        bitmap.insert(2);
        bitmap
    })
}

#[bench]
fn is_subset(b: &mut Bencher) {
    let bitmap: RoaringBitmap = (1..250).collect();
    b.iter(|| test::black_box(bitmap.is_subset(&bitmap)))
}

#[bench]
fn is_subset_2(b: &mut Bencher) {
    let sub: RoaringBitmap = (1000..8196).collect();
    let sup: RoaringBitmap = (0..16384).collect();
    b.iter(|| test::black_box(sub.is_subset(&sup)))
}

#[bench]
fn is_subset_3(b: &mut Bencher) {
    let sub: RoaringBitmap = (1000..4096).map(|x| x * 2).collect();
    let sup: RoaringBitmap = (0..16384).collect();
    b.iter(|| test::black_box(sub.is_subset(&sup)))
}

#[bench]
fn is_subset_4(b: &mut Bencher) {
    let sub: RoaringBitmap = (0..17).map(|x| 1 << x).collect();
    let sup: RoaringBitmap = (0..65536).collect();
    b.iter(|| test::black_box(sub.is_subset(&sup)))
}

#[bench]
fn remove_range_bitmap(b: &mut Bencher) {
    let mut sub: RoaringBitmap = (0..65536).collect();
    b.iter(|| {
        test::black_box(sub.remove_range(4096 + 1..65536));
        assert_eq!(sub.len(), 4096 + 1);
    });
}
