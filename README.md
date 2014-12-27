# RoaringBitmap [![Travis CI Build Status](https://img.shields.io/travis/Nemo157/roaring-rs.svg?style=flat-square)](https://travis-ci.org/Nemo157/roaring-rs)

> This is not yet production ready.

This is a [Rust](https://rust-lang.org) port of the [Roaring
bitmap](http://roaringbitmap.org) data structure, initially defined as a [Java
library](https://github.com/lemire/RoaringBitmap) and described in [Better bitmap
performance with Roaring bitmaps](http://arxiv.org/pdf/1402.6407v4).

## Example

```rust
let mut rr = roaring::RoaringBitmap::new();
for k in 4000..4255 {
  rr.set(k, true);
}
assert!(bitmap.get(4100))
```

## Developing

Take note of the [Collections
reform RFC](https://github.com/rust-lang/rfcs/pull/235) for the API.
