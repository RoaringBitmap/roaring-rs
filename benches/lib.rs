extern crate roaring;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use roaring::RoaringBitmap;

fn create(c: &mut Criterion) {
    c.bench_function("create", |b| {
        b.iter(|| {
            RoaringBitmap::new();
        })
    });
}

fn insert(c: &mut Criterion) {
    c.bench_function("insert 1", |b| {
        b.iter(|| {
            let mut bitmap = RoaringBitmap::new();
            bitmap.insert(1);
            bitmap
        });
    });

    c.bench_function("insert 2", |b| {
        b.iter(|| {
            let mut bitmap = RoaringBitmap::new();
            bitmap.insert(1);
            bitmap.insert(2);
            bitmap
        });
    });
}

fn is_subset(c: &mut Criterion) {
    c.bench_function("is_subset 1", |b| {
        let bitmap: RoaringBitmap = (1..250).collect();
        b.iter(|| black_box(bitmap.is_subset(&bitmap)))
    });

    c.bench_function("is_subset 2", |b| {
        let sub: RoaringBitmap = (1000..8196).collect();
        let sup: RoaringBitmap = (0..16384).collect();
        b.iter(|| black_box(sub.is_subset(&sup)))
    });

    c.bench_function("is_subset 3", |b| {
        let sub: RoaringBitmap = (1000..4096).map(|x| x * 2).collect();
        let sup: RoaringBitmap = (0..16384).collect();
        b.iter(|| black_box(sub.is_subset(&sup)))
    });

    c.bench_function("is_subset 4", |b| {
        let sub: RoaringBitmap = (0..17).map(|x| 1 << x).collect();
        let sup: RoaringBitmap = (0..65536).collect();
        b.iter(|| black_box(sub.is_subset(&sup)))
    });
}

fn remove_range_bitmap(c: &mut Criterion) {
    c.bench_function("remove_range_bitmap", |b| {
        let mut sub: RoaringBitmap = (0..65536).collect();
        b.iter(|| {
            // carefully delete part of the bitmap
            // only the first iteration will actually change something
            // but the runtime remains identical afterwards
            black_box(sub.remove_range(4096 + 1..65536));
            assert_eq!(sub.len(), 4096 + 1);
        });
    });
}

criterion_group!(benches, create, insert, is_subset, remove_range_bitmap);
criterion_main!(benches);
