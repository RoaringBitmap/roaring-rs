use itertools::Itertools;
use std::cmp::Reverse;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

use criterion::measurement::Measurement;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
    Throughput,
};

use roaring::{MultiOps, RoaringBitmap, RoaringTreemap};

use crate::datasets::Datasets;

mod datasets;

#[allow(clippy::too_many_arguments)]
fn pairwise_binary_op_matrix(
    c: &mut Criterion,
    op_name: &str,
    op_own_own: impl Fn(RoaringBitmap, RoaringBitmap) -> RoaringBitmap,
    op_own_ref: impl Fn(RoaringBitmap, &RoaringBitmap) -> RoaringBitmap,
    op_ref_own: impl Fn(&RoaringBitmap, RoaringBitmap) -> RoaringBitmap,
    op_ref_ref: impl Fn(&RoaringBitmap, &RoaringBitmap) -> RoaringBitmap,
    mut op_assign_owned: impl FnMut(&mut RoaringBitmap, RoaringBitmap),
    mut op_assign_ref: impl FnMut(&mut RoaringBitmap, &RoaringBitmap),
    op_len: impl Fn(&RoaringBitmap, &RoaringBitmap) -> u64,
) {
    let mut group = c.benchmark_group(format!("pairwise_{}", op_name));

    for dataset in Datasets {
        let pairs = dataset.bitmaps.iter().cloned().tuple_windows::<(_, _)>().collect::<Vec<_>>();

        group.bench_function(BenchmarkId::new("own_own", &dataset.name), |b| {
            b.iter_batched(
                || pairs.clone(),
                |bitmaps| {
                    for (a, b) in bitmaps {
                        black_box(op_own_own(a, b));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("own_ref", &dataset.name), |b| {
            b.iter_batched(
                || pairs.clone(),
                |bitmaps| {
                    for (a, b) in bitmaps {
                        black_box(op_own_ref(a, &b));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("ref_own", &dataset.name), |b| {
            b.iter_batched(
                || pairs.clone(),
                |bitmaps| {
                    for (a, b) in bitmaps {
                        black_box(op_ref_own(&a, b));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("ref_ref", &dataset.name), |b| {
            b.iter_batched(
                || pairs.clone(),
                |bitmaps| {
                    for (a, b) in bitmaps {
                        black_box(op_ref_ref(&a, &b));
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("assign_own", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.iter().cloned().tuple_windows::<(_, _)>().collect::<Vec<_>>(),
                |bitmaps| {
                    for (mut a, b) in bitmaps {
                        op_assign_owned(&mut a, b);
                        black_box(a);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("assign_ref", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.iter().cloned().tuple_windows::<(_, _)>().collect::<Vec<_>>(),
                |bitmaps| {
                    for (mut a, b) in bitmaps {
                        op_assign_ref(&mut a, &b);
                        black_box(a);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("len", &dataset.name), |b| {
            b.iter(|| {
                for (a, b) in pairs.iter() {
                    black_box(op_len(a, b));
                }
            })
        });
    }

    group.finish();
}

fn pairwise_binary_op<R, M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    op_name: &str,
    op: impl Fn(RoaringBitmap, RoaringBitmap) -> R,
) {
    for dataset in Datasets {
        group.bench_function(BenchmarkId::new(op_name, &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.iter().cloned().tuple_windows::<(_, _)>().collect::<Vec<_>>(),
                |bitmaps| {
                    for (a, b) in bitmaps {
                        black_box(op(a, b));
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
}

fn creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("creation");

    for dataset in Datasets {
        let dataset_numbers = dataset
            .bitmaps
            .iter()
            .map(|bitmap| bitmap.iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        group.throughput(Throughput::Elements(dataset.bitmaps.iter().map(|rb| rb.len()).sum()));

        group.bench_function(BenchmarkId::new("from_bitmap_bytes", &dataset.name), |b| {
            let bitmap_bytes = dataset_numbers
                .iter()
                .map(|bitmap_numbers| {
                    let max_number = *bitmap_numbers.iter().max().unwrap() as usize;
                    let mut buf = vec![0u8; max_number / 8 + 1];
                    for n in bitmap_numbers {
                        let byte = (n / 8) as usize;
                        let bit = n % 8;
                        buf[byte] |= 1 << bit;
                    }
                    buf
                })
                .collect::<Vec<_>>();
            b.iter(|| {
                for bitmap_bytes in &bitmap_bytes {
                    black_box(RoaringBitmap::from_bitmap_bytes(0, bitmap_bytes));
                }
            })
        });

        group.bench_function(BenchmarkId::new("from_sorted_iter", &dataset.name), |b| {
            b.iter(|| {
                for bitmap_numbers in &dataset_numbers {
                    black_box(
                        RoaringBitmap::from_sorted_iter(bitmap_numbers.iter().copied()).unwrap(),
                    );
                }
            })
        });

        group.bench_function(BenchmarkId::new("collect", &dataset.name), |b| {
            b.iter(|| {
                for bitmap_numbers in &dataset_numbers {
                    black_box(bitmap_numbers.iter().copied().collect::<RoaringBitmap>());
                }
            })
        });
    }

    group.finish();
}

fn len(c: &mut Criterion) {
    let mut group = c.benchmark_group("len");

    for dataset in Datasets {
        group.throughput(Throughput::Elements(dataset.bitmaps.iter().map(|rb| rb.len()).sum()));
        group.bench_function(BenchmarkId::new("len", &dataset.name), |b| {
            b.iter(|| {
                for bitmap in &dataset.bitmaps {
                    black_box(bitmap.len());
                }
            });
        });
    }

    group.finish();
}

fn rank(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank");
    for dataset in Datasets {
        let bitmaps =
            dataset.bitmaps.iter().map(|bitmap| (bitmap, bitmap.len() as u32)).collect::<Vec<_>>();

        // Rank all multiples of 100 < bitmap.len()
        // Mupliplier chosen arbitrarily, but should be sure not to rank many values > len()
        // Doing so would degenerate into benchmarking len()
        group.bench_function(BenchmarkId::new("rank", &dataset.name), |b| {
            b.iter(|| {
                for (bitmap, len) in bitmaps.iter() {
                    for i in (0..*len).step_by(100) {
                        black_box(bitmap.rank(i));
                    }
                }
            });
        });
    }
}

fn select(c: &mut Criterion) {
    let mut group = c.benchmark_group("select");
    for dataset in Datasets {
        let bitmaps = dataset
            .bitmaps
            .iter()
            .map(|bitmap| (bitmap, bitmap.max().unwrap()))
            .collect::<Vec<_>>();

        // Select all multiples of 100 < bitmap.max()
        // Mupliplier chosen arbitrarily, but should be sure not to select many values > max()
        // Doing so would degenerate into benchmarking len()
        group.bench_function(BenchmarkId::new("select", &dataset.name), |b| {
            b.iter(|| {
                for (bitmap, max) in bitmaps.iter() {
                    for i in (0..*max).step_by(100) {
                        black_box(bitmap.select(i));
                    }
                }
            });
        });
    }
}

#[allow(clippy::redundant_closure)]
fn and(c: &mut Criterion) {
    pairwise_binary_op_matrix(
        c,
        "and",
        |a, b| BitAnd::bitand(a, b),
        |a, b| BitAnd::bitand(a, b),
        |a, b| BitAnd::bitand(a, b),
        |a, b| BitAnd::bitand(a, b),
        |a, b| BitAndAssign::bitand_assign(a, b),
        |a, b| BitAndAssign::bitand_assign(a, b),
        |a, b| a.intersection_len(b),
    )
}

#[allow(clippy::redundant_closure)]
fn or(c: &mut Criterion) {
    pairwise_binary_op_matrix(
        c,
        "or",
        |a, b| BitOr::bitor(a, b),
        |a, b| BitOr::bitor(a, b),
        |a, b| BitOr::bitor(a, b),
        |a, b| BitOr::bitor(a, b),
        |a, b| BitOrAssign::bitor_assign(a, b),
        |a, b| BitOrAssign::bitor_assign(a, b),
        |a, b| a.union_len(b),
    )
}

#[allow(clippy::redundant_closure)]
fn sub(c: &mut Criterion) {
    pairwise_binary_op_matrix(
        c,
        "sub",
        |a, b| Sub::sub(a, b),
        |a, b| Sub::sub(a, b),
        |a, b| Sub::sub(a, b),
        |a, b| Sub::sub(a, b),
        |a, b| SubAssign::sub_assign(a, b),
        |a, b| SubAssign::sub_assign(a, b),
        |a, b| a.difference_len(b),
    )
}

#[allow(clippy::redundant_closure)]
fn xor(c: &mut Criterion) {
    pairwise_binary_op_matrix(
        c,
        "xor",
        BitXor::bitxor,
        |a, b| BitXor::bitxor(a, b),
        |a, b| BitXor::bitxor(a, b),
        |a, b| BitXor::bitxor(a, b),
        |a, b| BitXorAssign::bitxor_assign(a, b),
        |a, b| BitXorAssign::bitxor_assign(a, b),
        |a, b| a.symmetric_difference_len(b),
    )
}

fn subset(c: &mut Criterion) {
    let mut group = c.benchmark_group("pairwise_subset");
    pairwise_binary_op(&mut group, "is_subset", |a, b| a.is_subset(&b));
    group.finish();
}

fn disjoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("pairwise_disjoint");
    pairwise_binary_op(&mut group, "is_disjoint", |a, b| a.is_disjoint(&b));
    group.finish();
}

fn iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration");

    for dataset in Datasets {
        group.throughput(Throughput::Elements(dataset.bitmaps.iter().map(|rb| rb.len()).sum()));

        group.bench_function(BenchmarkId::new("iter", &dataset.name), |b| {
            b.iter(|| {
                for i in dataset.bitmaps.iter().flat_map(|bitmap| bitmap.iter()) {
                    black_box(i);
                }
            });
        });

        group.bench_function(BenchmarkId::new("into_iter", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.clone(),
                |bitmaps| {
                    for i in bitmaps.into_iter().flat_map(|bitmap| bitmap.into_iter()) {
                        black_box(i);
                    }
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("iter rev", &dataset.name), |b| {
            b.iter(|| {
                for i in dataset.bitmaps.iter().flat_map(|bitmap| bitmap.iter().rev()) {
                    black_box(i);
                }
            });
        });

        group.bench_function(BenchmarkId::new("into_iter rev", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.clone(),
                |bitmaps| {
                    for i in bitmaps.into_iter().flat_map(|bitmap| bitmap.into_iter().rev()) {
                        black_box(i);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    for dataset in Datasets {
        let sizes = dataset.bitmaps.iter().map(|rb| rb.serialized_size()).collect::<Vec<_>>();

        let max_size = sizes.iter().copied().max().unwrap();
        let mut buf = Vec::with_capacity(max_size);

        group.throughput(Throughput::Bytes(sizes.iter().copied().sum::<usize>() as u64));

        group.bench_function(BenchmarkId::new("serialized_size", &dataset.name), |b| {
            b.iter(|| {
                for bitmap in &dataset.bitmaps {
                    black_box(bitmap.serialized_size());
                }
            });
        });

        group.bench_function(BenchmarkId::new("serialize_into", &dataset.name), |b| {
            b.iter(|| {
                for bitmap in &dataset.bitmaps {
                    buf.clear();
                    bitmap.serialize_into(&mut buf).unwrap();
                    black_box(&buf);
                }
            });
        });
    }

    group.finish();
}

fn deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");

    for dataset in Datasets {
        let input = dataset
            .bitmaps
            .iter()
            .map(|rb| {
                let size = rb.serialized_size();
                let mut buf = Vec::with_capacity(size);
                rb.serialize_into(&mut buf).unwrap();
                buf
            })
            .collect::<Vec<_>>();

        group.throughput(Throughput::Bytes(input.iter().map(|buf| buf.len() as u64).sum()));

        group.bench_function(BenchmarkId::new("deserialize_from", &dataset.name), |b| {
            b.iter(|| {
                for buf in input.iter() {
                    black_box(RoaringBitmap::deserialize_from(buf.as_slice()).unwrap());
                }
            });
        });

        group.bench_function(BenchmarkId::new("deserialize_unchecked_from", &dataset.name), |b| {
            b.iter(|| {
                for buf in input.iter() {
                    black_box(RoaringBitmap::deserialize_unchecked_from(buf.as_slice()).unwrap());
                }
            });
        });
    }

    group.finish();
}

fn successive_and(c: &mut Criterion) {
    let mut group = c.benchmark_group("Successive And");

    for dataset in Datasets {
        // biggest bitmaps first.
        let mut sorted_bitmaps = dataset.bitmaps.clone();
        sorted_bitmaps.sort_unstable_by_key(|b| Reverse(b.len()));

        group.bench_function(BenchmarkId::new("Successive And Assign Ref", &dataset.name), |b| {
            b.iter_batched(
                || sorted_bitmaps.clone(),
                |bitmaps| {
                    let mut iter = bitmaps.into_iter();
                    let mut first = iter.next().unwrap();
                    for bitmap in iter {
                        first &= bitmap;
                    }
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("Successive And Assign Owned", &dataset.name), |b| {
            b.iter_batched(
                || sorted_bitmaps.clone(),
                |bitmaps| {
                    black_box(bitmaps.into_iter().reduce(|a, b| a & b).unwrap());
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("Successive And Ref Ref", &dataset.name), |b| {
            b.iter_batched(
                || sorted_bitmaps.clone(),
                |bitmaps| {
                    let mut iter = bitmaps.iter();
                    let first = iter.next().unwrap().clone();
                    black_box(iter.fold(first, |acc, x| (&acc) & x));
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("Multi And Ref", &dataset.name), |b| {
            b.iter(|| black_box(dataset.bitmaps.iter().intersection()));
        });

        group.bench_function(BenchmarkId::new("Multi And Owned", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.clone(),
                |bitmaps: Vec<RoaringBitmap>| black_box(bitmaps.intersection()),
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn successive_or(c: &mut Criterion) {
    let mut group = c.benchmark_group("Successive Or");

    for dataset in Datasets {
        group.bench_function(BenchmarkId::new("Successive Or Assign Ref", &dataset.name), |b| {
            b.iter(|| {
                let mut output = RoaringBitmap::new();
                for bitmap in &dataset.bitmaps {
                    output |= bitmap;
                }
            });
        });

        group.bench_function(BenchmarkId::new("Successive Or Assign Owned", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.clone(),
                |bitmaps: Vec<RoaringBitmap>| {
                    let mut output = RoaringBitmap::new();
                    for bitmap in bitmaps {
                        output |= bitmap;
                    }
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("Successive Or Ref Ref", &dataset.name), |b| {
            b.iter(|| {
                let mut output = RoaringBitmap::new();
                for bitmap in &dataset.bitmaps {
                    output = (&output) | bitmap;
                }
            });
        });

        group.bench_function(BenchmarkId::new("Multi Or Ref", &dataset.name), |b| {
            b.iter(|| black_box(dataset.bitmaps.iter().union()));
        });

        group.bench_function(BenchmarkId::new("Multi Or Owned", &dataset.name), |b| {
            b.iter_batched(
                || dataset.bitmaps.clone(),
                |bitmaps: Vec<RoaringBitmap>| black_box(bitmaps.union()),
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// LEGACY BENCHMARKS
// =================

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
        group.throughput(criterion::Throughput::Elements(size as u64));
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

fn insert_range_treemap(c: &mut Criterion) {
    for &size in &[1_000_u64, 10_000u64, 2 * (u32::MAX as u64)] {
        let mut group = c.benchmark_group("insert_range_treemap");
        group.throughput(criterion::Throughput::Elements(size));
        group.bench_function(format!("from_empty_{}", size), |b| {
            let bm = RoaringTreemap::new();
            b.iter_batched(
                || bm.clone(),
                |mut bm| black_box(bm.insert_range(0..size)),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_function(format!("pre_populated_{}", size), |b| {
            let mut bm = RoaringTreemap::new();
            bm.insert_range(0..size);
            b.iter_batched(
                || bm.clone(),
                |mut bm| black_box(bm.insert_range(0..size)),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(
    benches,
    creation,
    insert,
    contains,
    len,
    rank,
    select,
    and,
    or,
    sub,
    xor,
    subset,
    disjoint,
    remove,
    remove_range_bitmap,
    insert_range_bitmap,
    insert_range_treemap,
    iteration,
    is_empty,
    serialization,
    deserialization,
    successive_and,
    successive_or
);
criterion_main!(benches);
