# RoaringBitmap

> This is not yet production ready.

This is a [Rust](https://rust-lang.org) port of the Roaring bitmap data
structure. The data structure was initially defined as a [Java
library](https://github.com/lemire/RoaringBitmap) and is described at its
[homepage](http://roaringbitmap.org) and associated
[paper](http://arxiv.org/pdf/1402.6407v4).

## Example

```rust
let mut rr = roaring::RoaringBitmap::new();
for k in 4000..4255 {
  rr.set(k, true);
}
assert!(bitmap.get(4100))
```
