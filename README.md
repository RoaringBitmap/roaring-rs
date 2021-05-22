# RoaringBitmap [![travis-badge][]][travis] [![release-badge][]][cargo] [![docs-badge][]][docs] [![rust-version-badge][]][rust-version]

This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
defined as a [Java library][roaring-java] and described in [_Better bitmap
performance with Roaring bitmaps_][roaring-paper].

## Rust version policy

This crate only supports the current stable version of Rust, patch releases may
use new features at any time.

## Developing

This project uses [clippy][], [rustfmt][], and denies warnings in CI builds. To ensure your
changes will be accepted please check them with `cargo clippy` (available via
`cargo install clippy` on nightly rust) before submitting a pull request (along
with `cargo test` as usual).

### Benchmarking

It is recommended to run the `cargo bench` command inside of the `benchmarks` directory.
This directory contains a library that is dedicated to benchmarking the roaring library
by using a set of [real-world datasets][]. It is also advised to run the benchmarks on
a bare-metal machine, running them on the base branch and then on the contribution PR
branch to better see the changes.

Those benchmarks are designed on top of the criterion library,
you can read more about it [on the User guide][].

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

[travis-badge]: https://img.shields.io/travis/Nemo157/roaring-rs/master.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/roaring-rs
[release-badge]: https://img.shields.io/github/release/Nemo157/roaring-rs.svg?style=flat-square
[cargo]: https://crates.io/crates/roaring
[docs-badge]: https://img.shields.io/badge/API-docs-blue.svg?style=flat-square
[docs]: https://nemo157.com/roaring-rs/
[rust-version-badge]: https://img.shields.io/badge/rust-latest%20stable-blue.svg?style=flat-square
[rust-version]: .travis.yml#L5

[Rust]: https://rust-lang.org
[Roaring bitmap]: http://roaringbitmap.org
[roaring-java]: https://github.com/lemire/RoaringBitmap
[roaring-paper]: https://arxiv.org/pdf/1402.6407v4
[clippy]: https://github.com/rust-lang/rust-clippy
[rustfmt]: https://github.com/rust-lang/rustfmt

[real-world datasets]: https://github.com/RoaringBitmap/real-roaring-datasets
[on the User guide]: https://bheisler.github.io/criterion.rs/book/user_guide/user_guide.html
