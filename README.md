# RoaringBitmap [![Travis CI Build Status][]][travis]

> This is not yet production ready.

This is a [Rust][] port of the [Roaring bitmap][] data structure, initially
defined as a [Java library][roaring-java] and described in [_Better bitmap
performance with Roaring bitmaps_][roaring-paper].

## Developing

Take note of the [Collections reform RFC][collections-rfc] for the API.  Mostly aiming to
duplicate the [BitvSet][] API.

### TODO

  - [ ] Bounded Iterators ([§ in the RFC][bounded-iterators])
    - [ ] `fn range(&self, min: Bound<&T>, max: Bound<&T>) -> RangedItems<'a, T>;`
  - [ ] Set Operations ([§ in the RFC][set-operations])
    - [ ] Comparisons
      - [X] `fn is_disjoint(&self, other: &Self) -> bool;`
      - [ ] `fn is_subset(&self, other: &Self) -> bool;`
      - [ ] `fn is_superset(&self, other: &Self) -> bool;`
    - [ ] Combinations
      - [ ] Iterated Functions
        - [ ] `fn union<'a>(&'a self, other: &'a Self) -> I;`
        - [ ] `fn intersection<'a>(&'a self, other: &'a Self) -> I;`
        - [ ] `fn difference<'a>(&'a self, other: &'a Self) -> I;`
        - [ ] `fn symmetric_difference<'a>(&'a self, other: &'a Self) -> I;`
      - [ ] Operator Traits
        - [ ] `std::ops::BitOr`
        - [ ] `std::ops::BitAnd`
        - [ ] `std::ops::BitXor`
        - [ ] `std::ops::Sub`
      - [ ] Inplace Functions
        - [ ] `fn union_with(&mut self, other: &BitvSet)`
        - [ ] `fn intersect_with(&mut self, other: &BitvSet)`
        - [ ] `fn difference_with(&mut self, other: &BitvSet)`
        - [ ] `fn symmetric_difference_with(&mut self, other: &BitvSet)`

[Travis CI Build Status]: https://img.shields.io/travis/Nemo157/roaring-rs.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/roaring-rs
[Rust]: https://rust-lang.org
[Roaring bitmap]: http://roaringbitmap.org
[roaring-java]: https://github.com/lemire/RoaringBitmap
[roaring-paper]: http://arxiv.org/pdf/1402.6407v4
[collections-rfc]: https://github.com/rust-lang/rfcs/pull/235
[BitvSet]: http://doc.rust-lang.org/std/collections/bitv_set/struct.BitvSet.html
[bounded-iterators]: https://github.com/aturon/rfcs/blob/collections-conventions/text/0000-collection-conventions.md#bounded-iterators
[set-operations]: https://github.com/aturon/rfcs/blob/collections-conventions/text/0000-collection-conventions.md#set-operations
