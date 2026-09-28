use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use roaring::RoaringBitmap;

fn equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("equality");

    // Patterns 0x01, 0x55, and 0x7f have densities of 12.5%, 50%, and 87.5%, respectively.
    for (containers, pattern) in [(1, 0x55), (64, 0x01), (64, 0x55), (64, 0x7f), (16384, 0x55)] {
        let bitmap = RoaringBitmap::from_lsb0_bytes(0, &vec![pattern; containers * 8192]);
        assert_eq!(bitmap.statistics().n_bitset_containers, containers as u32);
        let other = bitmap.clone();
        group.bench_function(
            BenchmarkId::new(format!("bitmap_equal_{pattern:02x}"), containers),
            |b| {
                b.iter(|| black_box(black_box(&bitmap) == black_box(&other)));
            },
        );
    }

    let bitmap = RoaringBitmap::from_lsb0_bytes(0, &vec![0x55; 64 * 8192]);
    for (position, offset) in [("first", 0), ("last", 64 * 65536 - 2)] {
        let mut other = bitmap.clone();
        assert!(other.remove(offset));
        assert_ne!(bitmap.len(), other.len());
        group.bench_function(format!("bitmap_unequal_length_{position}"), |b| {
            b.iter(|| black_box(black_box(&bitmap) == black_box(&other)));
        });

        assert!(other.insert(offset + 1));
        assert_eq!(bitmap.len(), other.len());
        assert_ne!(bitmap, other);
        group.bench_function(format!("bitmap_unequal_content_{position}"), |b| {
            b.iter(|| black_box(black_box(&bitmap) == black_box(&other)));
        });
    }

    let array: RoaringBitmap = (0..64 * 65536).step_by(32).collect();
    assert_eq!(array.statistics().n_array_containers, 64);
    let other = array.clone();
    group.bench_function("array_equal_64", |b| {
        b.iter(|| black_box(black_box(&array) == black_box(&other)));
    });

    let mut runs: RoaringBitmap = (0..64 * 65536).collect();
    runs.optimize();
    assert_eq!(runs.statistics().n_run_containers, 64);
    let other = runs.clone();
    group.bench_function("run_equal_64", |b| {
        b.iter(|| black_box(black_box(&runs) == black_box(&other)));
    });

    group.finish();
}

criterion_group!(benches, equality);
criterion_main!(benches);
