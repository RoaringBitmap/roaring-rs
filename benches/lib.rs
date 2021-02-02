extern crate roaring;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize};
use roaring::RoaringBitmap;
use roaring::borrowed_bitmap::BorrowedRoaringBitmap;
use rand::{Rng, SeedableRng};

fn create(c: &mut Criterion) {
    c.bench_function("create", |b| {
        b.iter(|| {
            RoaringBitmap::new();
        })
    });
}

fn insert(c: &mut Criterion) {
    c.bench_function("create & insert 1", |b| {
        b.iter(|| {
            let mut bitmap = RoaringBitmap::new();
            bitmap.insert(black_box(1));
        });
    });

    c.bench_function("insert 1", |b| {
        let mut bitmap = RoaringBitmap::new();
        b.iter(|| {
            bitmap.insert(black_box(1));
        });
    });

    c.bench_function("create & insert several", |b| {
        b.iter(|| {
            let mut bitmap = RoaringBitmap::new();
            bitmap.insert(black_box(1));
            bitmap.insert(black_box(10));
            bitmap.insert(black_box(100));
            bitmap.insert(black_box(1_000));
            bitmap.insert(black_box(10_000));
            bitmap.insert(black_box(100_000));
            bitmap.insert(black_box(1_000_000));
        });
    });

    c.bench_function("insert several", |b| {
        let mut bitmap = RoaringBitmap::new();
        b.iter(|| {
            bitmap.insert(black_box(1));
            bitmap.insert(black_box(10));
            bitmap.insert(black_box(100));
            bitmap.insert(black_box(1_000));
            bitmap.insert(black_box(10_000));
            bitmap.insert(black_box(100_000));
            bitmap.insert(black_box(1_000_000));
        });
    });
}

fn contains(c: &mut Criterion) {
    c.bench_function("contains true", |b| {
        let mut bitmap: RoaringBitmap = RoaringBitmap::new();
        bitmap.insert(1);

        b.iter(|| {
            bitmap.contains(black_box(1));
        });
    });

    c.bench_function("contains false", |b| {
        let bitmap: RoaringBitmap = RoaringBitmap::new();

        b.iter(|| {
            bitmap.contains(black_box(1));
        });
    });
}

fn len(c: &mut Criterion) {
    c.bench_function("len 100000", |b| {
        let bitmap: RoaringBitmap = (1..100_000).collect();

        b.iter(|| {
            black_box(bitmap.len());
        });
    });
    c.bench_function("len 1000000", |b| {
        let bitmap: RoaringBitmap = (1..1_000_000).collect();

        b.iter(|| {
            black_box(bitmap.len());
        });
    });
}

fn and(c: &mut Criterion) {
    c.bench_function("and", |b| {
        let bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| &bitmap1 & &bitmap2);
    });
}

fn intersect_with(c: &mut Criterion) {
    c.bench_function("intersect_with", |b| {
        let mut bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| {
            bitmap1.intersect_with(black_box(&bitmap2));
        });
    });
}

fn or(c: &mut Criterion) {
    c.bench_function("or", |b| {
        let bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| &bitmap1 | &bitmap2);
    });
}

fn union_with(c: &mut Criterion) {
    c.bench_function("union_with", |b| {
        let mut bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| {
            bitmap1.union_with(black_box(&bitmap2));
        });
    });
}

fn xor(c: &mut Criterion) {
    c.bench_function("xor", |b| {
        let bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| &bitmap1 ^ &bitmap2);
    });
}

fn symmetric_deference_with(c: &mut Criterion) {
    c.bench_function("symmetric_deference_with", |b| {
        let mut bitmap1: RoaringBitmap = (1..100).collect();
        let bitmap2: RoaringBitmap = (100..200).collect();

        b.iter(|| {
            bitmap1.symmetric_difference_with(black_box(&bitmap2));
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

fn remove(c: &mut Criterion) {
    c.bench_function("remove 1", |b| {
        let mut sub: RoaringBitmap = (0..65_536).collect();
        b.iter(|| {
            black_box(sub.remove(1000));
        });
    });
}

fn remove_range_bitmap(c: &mut Criterion) {
    c.bench_function("remove_range 1", |b| {
        let mut sub: RoaringBitmap = (0..65_536).collect();
        b.iter(|| {
            // carefully delete part of the bitmap
            // only the first iteration will actually change something
            // but the runtime remains identical afterwards
            black_box(sub.remove_range(4096 + 1..65_536));
            assert_eq!(sub.len(), 4096 + 1);
        });
    });

    c.bench_function("remove_range 2", |b| {
        // Slower bench that creates a new bitmap on each iteration so that can benchmark
        // bitmap to array conversion
        b.iter(|| {
            let mut sub: RoaringBitmap = (0..65_536).collect();
            black_box(sub.remove_range(100..65_536));
            assert_eq!(sub.len(), 100);
        });
    });
}

fn insert_range_bitmap(c: &mut Criterion) {
    for &size in &[10, 100, 1_000, 5_000, 10_000, 20_000] {
        let mut group = c.benchmark_group("insert_range");
        group.throughput(criterion::Throughput::Elements(size));
        group.bench_function(format!("from_empty_{}", size), |b| {
            let bm = RoaringBitmap::new();
            b.iter_batched(
                || bm.clone(),
                |mut bm| black_box(bm.insert_range(0..size)),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("pre_populated_{}", size), |b| {
            let mut bm = RoaringBitmap::new();
            bm.insert_range(0..size);
            b.iter_batched(
                || bm.clone(),
                |mut bm| black_box(bm.insert_range(0..size)),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

fn iter(c: &mut Criterion) {
    c.bench_function("iter", |b| {
        let bitmap: RoaringBitmap = (1..10_000).collect();

        b.iter(|| {
            let mut sum: u32 = 0;

            for (_, element) in bitmap.iter().enumerate() {
                sum += element;
            }

            assert_eq!(sum, 49_995_000);
        });
    });
}

fn is_empty(c: &mut Criterion) {
    c.bench_function("is_empty true", |b| {
        let bitmap = RoaringBitmap::new();
        b.iter(|| {
            bitmap.is_empty();
        });
    });
    c.bench_function("is_empty false", |b| {
        let mut bitmap = RoaringBitmap::new();
        bitmap.insert(1);
        b.iter(|| {
            bitmap.is_empty();
        });
    });
}

fn serialize(c: &mut Criterion) {
    c.bench_function("serialize 100000", |b| {
        let bitmap: RoaringBitmap = (1..100_000).collect();
        let mut buffer = Vec::with_capacity(bitmap.serialized_size());

        b.iter(|| {
            bitmap.serialize_into(&mut buffer).unwrap();
        });
    });
    c.bench_function("serialize 1000000", |b| {
        let bitmap: RoaringBitmap = (1..1_000_000).collect();
        let mut buffer = Vec::with_capacity(bitmap.serialized_size());

        b.iter(|| {
            bitmap.serialize_into(&mut buffer).unwrap();
        });
    });
}

fn deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialize");

    for n in [100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("Owned", n), n,
            |b, n| {
                let bitmap: RoaringBitmap = (1..*n).collect();
                let mut buffer = Vec::with_capacity(bitmap.serialized_size());
                bitmap.serialize_into(&mut buffer).unwrap();

                b.iter(|| {
                    RoaringBitmap::deserialize_from(&buffer[..]).unwrap();
                });
            });
        group.bench_with_input(BenchmarkId::new("Borrowed", n), n,
            |b, n| {
                let bitmap: RoaringBitmap = (1..*n).collect();
                let mut buffer = Vec::with_capacity(bitmap.serialized_size());
                bitmap.serialize_into(&mut buffer).unwrap();

                b.iter(|| {
                    BorrowedRoaringBitmap::deserialize_from_slice(&buffer[..]).unwrap();
                });
            });
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut random_numbers = vec![0; 10_000];
    rng.fill(&mut random_numbers[..]);

    group.bench_with_input(BenchmarkId::new("Owned", "random 10000"), &random_numbers,
        |b, numbers| {
            let bitmap: RoaringBitmap = numbers.iter().copied().collect();
            let mut buffer = Vec::with_capacity(bitmap.serialized_size());
            bitmap.serialize_into(&mut buffer).unwrap();

            b.iter(|| {
                RoaringBitmap::deserialize_from(&buffer[..]).unwrap();
            });
        });
    group.bench_with_input(BenchmarkId::new("Borrowed", "random 10000"), &random_numbers,
        |b, numbers| {
            let bitmap: RoaringBitmap = numbers.iter().copied().collect();
            let mut buffer = Vec::with_capacity(bitmap.serialized_size());
            bitmap.serialize_into(&mut buffer).unwrap();

            b.iter(|| {
                BorrowedRoaringBitmap::deserialize_from_slice(&buffer[..]).unwrap();
            });
        });
}

fn deserialize_big(c: &mut Criterion) {
    // there is 200100 values.
    let bytes = include_bytes!("../tests/bitmapwithoutruns.bin");

    let mut group = c.benchmark_group("Deserialize Big");

    group.bench_with_input("Owned", bytes,
        |b, bytes| b.iter(|| {
            RoaringBitmap::deserialize_from(&bytes[..]).unwrap();
        }));
    group.bench_with_input("Borrowed", bytes,
        |b, bytes| b.iter(|| {
            BorrowedRoaringBitmap::deserialize_from_slice(bytes).unwrap();
        }));
}

fn deserialize_intersect(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialize and Intersect");

    for n in [10_000, 100_000].iter() {
        let mut rna = vec![0; *n];
        let mut rnb = vec![0; *n];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        rng.fill(&mut rna[..]);
        rng.fill(&mut rnb[..]);

        group.bench_with_input(BenchmarkId::new("Owned random", n), &(&rna, &rnb),
            |b, (rna, rnb)| {
                let bitmap_a: RoaringBitmap = rna.iter().copied().collect();
                let bitmap_b: RoaringBitmap = rnb.iter().copied().collect();
                let mut buffer_a = Vec::with_capacity(bitmap_a.serialized_size());
                let mut buffer_b = Vec::with_capacity(bitmap_b.serialized_size());
                bitmap_a.serialize_into(&mut buffer_a).unwrap();
                bitmap_b.serialize_into(&mut buffer_b).unwrap();

                b.iter_batched(|| bitmap_a.clone(), |mut bitmap_a| {
                    let bitmap_b = RoaringBitmap::deserialize_from(&buffer_b[..]).unwrap();
                    bitmap_a.intersect_with(&bitmap_b)
                }, BatchSize::SmallInput);
            });
        group.bench_with_input(BenchmarkId::new("Borrowed random", n), &(&rna, &rnb),
            |b, (rna, rnb)| {
                let bitmap_a: RoaringBitmap = rna.iter().copied().collect();
                let bitmap_b: RoaringBitmap = rnb.iter().copied().collect();
                let mut buffer_b = Vec::with_capacity(bitmap_b.serialized_size());
                bitmap_b.serialize_into(&mut buffer_b).unwrap();

                b.iter_batched(|| bitmap_a.clone(), |mut bitmap_a| {
                    let bitmap_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer_b[..]).unwrap();
                    bitmap_a &= &bitmap_b;
                }, BatchSize::SmallInput);
            });
    }

}

fn deserialize_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("Deserialize and Union");

    for n in [10_000, 100_000].iter() {
        let mut rna = vec![0; *n];
        let mut rnb = vec![0; *n];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        rng.fill(&mut rna[..]);
        rng.fill(&mut rnb[..]);

        group.bench_with_input(BenchmarkId::new("Owned random", n), &(&rna, &rnb),
            |b, (rna, rnb)| {
                let bitmap_a: RoaringBitmap = rna.iter().copied().collect();
                let bitmap_b: RoaringBitmap = rnb.iter().copied().collect();
                let mut buffer_b = Vec::with_capacity(bitmap_b.serialized_size());
                bitmap_b.serialize_into(&mut buffer_b).unwrap();

                b.iter_batched(|| bitmap_a.clone(), |mut bitmap_a| {
                    let bitmap_b = RoaringBitmap::deserialize_from(&buffer_b[..]).unwrap();
                    bitmap_a.union_with(&bitmap_b)
                }, BatchSize::SmallInput);
            });
        group.bench_with_input(BenchmarkId::new("Borrowed random", n), &(&rna, &rnb),
            |b, (rna, rnb)| {
                let bitmap_a: RoaringBitmap = rna.iter().copied().collect();
                let bitmap_b: RoaringBitmap = rnb.iter().copied().collect();
                let mut buffer_b = Vec::with_capacity(bitmap_b.serialized_size());
                bitmap_b.serialize_into(&mut buffer_b).unwrap();

                b.iter_batched(|| bitmap_a.clone(), |mut bitmap_a| {
                    let bitmap_b = BorrowedRoaringBitmap::deserialize_from_slice(&buffer_b[..]).unwrap();
                    bitmap_a |= &bitmap_b;
                }, BatchSize::SmallInput);
            });
    }
}

fn serialized_size(c: &mut Criterion) {
    c.bench_function("serialized_size", |b| {
        let bitmap: RoaringBitmap = (1..100).collect();
        b.iter(|| bitmap.serialized_size());
    });
}

criterion_group!(
    benches,
    create,
    insert,
    contains,
    len,
    and,
    intersect_with,
    or,
    union_with,
    xor,
    symmetric_deference_with,
    is_subset,
    remove,
    remove_range_bitmap,
    insert_range_bitmap,
    iter,
    is_empty,
    serialize,
    deserialize,
    deserialize_big,
    deserialize_intersect,
    deserialize_union,
    serialized_size,
);
criterion_main!(benches);
