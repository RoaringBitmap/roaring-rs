# RoaringBitmap [![Travis CI Build Status][]][travis]

> This is not yet production ready. The API should be mostly complete now.

This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
defined as a [Java library][roaring-java] and described in [_Better bitmap
performance with Roaring bitmaps_][roaring-paper].

## Developing

Take note of the [Collections reform RFC][collections-rfc] for the API.  Mostly aiming to
duplicate the [BitvSet][] API.

[Travis CI Build Status]: https://img.shields.io/travis/Nemo157/roaring-rs.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/roaring-rs
[Rust]: https://rust-lang.org
[Roaring bitmap]: http://roaringbitmap.org
[roaring-java]: https://github.com/lemire/RoaringBitmap
[roaring-paper]: http://arxiv.org/pdf/1402.6407v4
[collections-rfc]: https://github.com/rust-lang/rfcs/pull/235
[BitvSet]: http://doc.rust-lang.org/std/collections/bitv_set/struct.BitvSet.html
