# RoaringBitmap [![travis-badge][]][travis] [![release-badge][]][cargo] [![docs-badge][]][docs] [![rust-version-badge][]][rust-version]

> This is not yet production ready. The API should be mostly complete now.

This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
defined as a [Java library][roaring-java] and described in [_Better bitmap
performance with Roaring bitmaps_][roaring-paper].

## Rust version policy

This crate only supports the current stable version of Rust, patch releases may
use new features at any time.

## Developing

This project uses [clippy][] and denies warnings in CI builds. To ensure your
changes will be accepted please check them with `cargo clippy` (available via
`cargo install clippy` on nightly rust) before submitting a pull request (along
with `cargo test` as usual).

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
[clippy]: https://github.com/Manishearth/rust-clippy
