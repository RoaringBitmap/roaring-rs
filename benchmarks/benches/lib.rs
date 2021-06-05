mod datasets_paths;

use std::cmp::Reverse;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::{fs, io};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use roaring::RoaringBitmap;

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
            bitmap1 &= black_box(&bitmap2);
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
            bitmap1 |= black_box(&bitmap2);
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
            bitmap1 ^= black_box(&bitmap2);
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
    c.bench_function("deserialize 100000", |b| {
        let bitmap: RoaringBitmap = (1..100_000).collect();
        let mut buffer = Vec::with_capacity(bitmap.serialized_size());
        bitmap.serialize_into(&mut buffer).unwrap();

        b.iter(|| {
            RoaringBitmap::deserialize_from(&buffer[..]).unwrap();
        });
    });
    c.bench_function("deserialize 1000000", |b| {
        let bitmap: RoaringBitmap = (1..1_000_000).collect();
        let mut buffer = Vec::with_capacity(bitmap.serialized_size());
        bitmap.serialize_into(&mut buffer).unwrap();

        b.iter(|| {
            RoaringBitmap::deserialize_from(&buffer[..]).unwrap();
        });
    });
}

fn serialized_size(c: &mut Criterion) {
    c.bench_function("serialized_size", |b| {
        let bitmap: RoaringBitmap = (1..100).collect();
        b.iter(|| bitmap.serialized_size());
    });
}

fn extract_integers<A: AsRef<str>>(content: A) -> Result<Vec<u32>, ParseIntError> {
    content.as_ref().split(',').map(|s| s.trim().parse()).collect()
}

// Parse every file into a vector of integer.
fn parse_dir_files<A: AsRef<Path>>(
    files: A,
) -> io::Result<Vec<(PathBuf, Result<Vec<u32>, ParseIntError>)>> {
    fs::read_dir(files)?
        .map(|r| r.and_then(|e| fs::read_to_string(e.path()).map(|r| (e.path(), r))))
        .map(|r| r.map(|(p, c)| (p, extract_integers(c))))
        .collect()
}

fn from_sorted_iter(c: &mut Criterion) {
    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    c.bench_function("from_sorted_iter", |b| {
        b.iter(|| {
            for (_, numbers) in &parsed_numbers {
                let numbers = numbers.as_ref().unwrap();
                RoaringBitmap::from_sorted_iter(numbers.iter().copied());
            }
        })
    });
}

fn successive_and(c: &mut Criterion) {
    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    let mut bitmaps: Vec<_> = parsed_numbers
        .into_iter()
        .map(|(_, r)| r.map(RoaringBitmap::from_sorted_iter).unwrap())
        .collect();

    // biggest bitmaps first.
    bitmaps.sort_unstable_by_key(|b| Reverse(b.len()));

    let mut group = c.benchmark_group("Successive And");

    group.bench_function("Successive And Assign Ref", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps| {
                let mut iter = bitmaps.into_iter();
                let mut first = iter.next().unwrap().clone();
                for bitmap in iter {
                    first &= bitmap;
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("Successive And Assign Owned", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps| {
                black_box(bitmaps.into_iter().reduce(|a, b| a & b).unwrap());
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("Successive And Ref Ref", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps| {
                let mut iter = bitmaps.iter();
                let first = iter.next().unwrap().clone();
                black_box(iter.fold(first, |acc, x| (&acc) & x));
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

fn successive_or(c: &mut Criterion) {
    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    let bitmaps: Vec<_> = parsed_numbers
        .into_iter()
        .map(|(_, r)| r.map(RoaringBitmap::from_sorted_iter).unwrap())
        .collect();

    let mut group = c.benchmark_group("Successive Or");
    group.bench_function("Successive Or Assign Ref", |b| {
        b.iter(|| {
            let mut output = RoaringBitmap::new();
            for bitmap in &bitmaps {
                output |= bitmap;
            }
        });
    });

    group.bench_function("Successive Or Assign Owned", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                let mut output = RoaringBitmap::new();
                for bitmap in bitmaps {
                    output |= bitmap;
                }
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("Successive Or Ref Ref", |b| {
        b.iter(|| {
            let mut output = RoaringBitmap::new();
            for bitmap in &bitmaps {
                output = (&output) | bitmap;
            }
        });
    });

    group.finish();
}

fn multi_bitor(c: &mut Criterion) {
    use roaring::bitmap::MultiBitOr;

    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    let bitmaps: Vec<_> = parsed_numbers
        .into_iter()
        .map(|(_, r)| r.map(RoaringBitmap::from_sorted_iter).unwrap())
        .collect();

    let mut group = c.benchmark_group("Multi Or");

    group.bench_function("Multi Or Ref", |b| {
        b.iter(|| {
            black_box(bitmaps.as_slice().bitor());
        });
    });

    group.bench_function("Multi Or Ref By Hand", |b| {
        b.iter(|| {
            let mut base = RoaringBitmap::default();
            for bm in &bitmaps {
                black_box(base |= bm);
            }
        });
    });

    group.bench_function("Multi Or Owned", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                black_box(bitmaps.bitor());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("Multi Or Owned By Hand", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                let mut base = RoaringBitmap::default();
                for bm in bitmaps {
                    black_box(base |= bm);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn multi_bitand(c: &mut Criterion) {
    use roaring::bitmap::MultiBitAnd;

    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    let bitmaps: Vec<_> = parsed_numbers
        .into_iter()
        .map(|(_, r)| r.map(RoaringBitmap::from_sorted_iter).unwrap())
        .collect();

    let mut group = c.benchmark_group("Multi And");

    group.bench_function("Multi And Ref", |b| {
        b.iter(|| {
            black_box(bitmaps.as_slice().bitand());
        });
    });

    group.bench_function("Multi And Ref By Hand", |b| {
        b.iter(|| {
            let mut iter = bitmaps.iter();
            let mut base = iter.next().cloned().unwrap();
            for bm in iter {
                black_box(base &= bm);
            }
        });
    });

    group.bench_function("Multi And Owned", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                black_box(bitmaps.bitand());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("Multi And Owned By Hand", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                black_box(bitmaps.into_iter().reduce(|a, b| a & b).unwrap_or_default());
            },
            BatchSize::SmallInput,
        );
    });
}

fn multi_bitxor(c: &mut Criterion) {
    use roaring::bitmap::MultiBitXor;

    let files = self::datasets_paths::WIKILEAKS_NOQUOTES_SRT;
    let parsed_numbers = parse_dir_files(files).unwrap();

    let bitmaps: Vec<_> = parsed_numbers
        .into_iter()
        .map(|(_, r)| r.map(RoaringBitmap::from_sorted_iter).unwrap())
        .collect();

    let mut group = c.benchmark_group("Multi Xor");

    group.bench_function("Multi Xor Ref", |b| {
        b.iter(|| {
            black_box(bitmaps.as_slice().bitxor());
        });
    });

    group.bench_function("Multi Xor Ref By Hand", |b| {
        b.iter(|| {
            let mut iter = bitmaps.iter();
            let mut base = iter.next().cloned().unwrap();
            for bm in iter {
                black_box(base ^= bm);
            }
        });
    });

    group.bench_function("Multi Xor Owned", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                black_box(bitmaps.bitxor());
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("Multi Xor Owned By Hand", |b| {
        b.iter_batched(
            || bitmaps.clone(),
            |bitmaps: Vec<RoaringBitmap>| {
                black_box(bitmaps.into_iter().reduce(|a, b| a ^ b).unwrap_or_default());
            },
            BatchSize::SmallInput,
        );
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
    serialized_size,
    from_sorted_iter,
    successive_and,
    successive_or,
    multi_bitor,
    multi_bitand,
    multi_bitxor,
);
criterion_main!(benches);
