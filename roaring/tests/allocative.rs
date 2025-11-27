#![cfg(feature = "allocative")]

use std::fs::File;
use std::io::Write;

use allocative::FlameGraphBuilder;
use roaring::RoaringBitmap;
use roaring::RoaringTreemap;

#[test]
fn flamegraph_bitmap() {
    let mut foo1 = RoaringBitmap::new();
    foo1.insert_range(0..1_000_000);
    foo1.insert(2_000_000);
    foo1.insert(9_000_000);

    let mut flamegraph = FlameGraphBuilder::default();
    flamegraph.visit_root(&foo1);
    let flamegraph_src = flamegraph.finish().flamegraph().write();
    let mut f = File::create("../target/bitmap.folded").unwrap();
    write!(f, "{}", flamegraph_src).unwrap();

    /*
    cargo test -p roaring --features allocative --test allocative
    inferno-flamegraph target/bitmap.folded > target/bitmap.flamegraph.svg
    open target/bitmap.flamegraph.svg
    */
}

#[test]
fn flamegraph_treemap() {
    let mut foo1 = RoaringTreemap::new();
    foo1.insert_range(0..1_000_000);
    foo1.insert(2_000_000);
    foo1.insert(9_000_000);
    foo1.insert((1 << 32) + 9000);
    foo1.insert((2 << 32) + 9000);

    let mut flamegraph = FlameGraphBuilder::default();
    flamegraph.visit_root(&foo1);
    let flamegraph_src = flamegraph.finish().flamegraph().write();
    let mut f = File::create("../target/treemap.folded").unwrap();
    write!(f, "{}", flamegraph_src).unwrap();

    /*
    cargo test -p roaring --features allocative --test allocative
    inferno-flamegraph target/treemap.folded > target/treemap.flamegraph.svg
    open target/treemap.flamegraph.svg
    */
}
