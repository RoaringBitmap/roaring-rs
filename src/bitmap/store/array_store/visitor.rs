/// This visitor pattern allows multiple different algorithms to be written over the same data
/// For example: vectorized algorithms can pass a visitor off to a scalar algorithm to finish off
/// a tail that is not a multiple of the vector width.
///
/// Perhaps more importantly: it separates the set algorithms from the operations performed on
/// their results. Future work can utilize the exiting algorithms to trivially implement
/// computing the cardinality of an operation without materializng a new bitmap.
pub trait BinaryOperationVisitor {
    fn visit_scalar(&mut self, value: u16);
    fn visit_slice(&mut self, values: &[u16]);
}

/// A simple visitor that stores the computation result to a Vec
/// accessible by calling `into_inner()`
pub struct VecWriter {
    vec: Vec<u16>,
}

impl VecWriter {
    pub fn new(capacity: usize) -> VecWriter {
        let vec = Vec::with_capacity(capacity);
        VecWriter { vec }
    }

    pub fn into_inner(self) -> Vec<u16> {
        // Consider shrinking the vec here.
        // Exacty len could be too aggressive. Len rounded up to next power of 2?
        // Related, but not exact issue: https://github.com/RoaringBitmap/roaring-rs/issues/136
        self.vec
    }
}

impl BinaryOperationVisitor for VecWriter {
    fn visit_scalar(&mut self, value: u16) {
        self.vec.push(value)
    }

    fn visit_slice(&mut self, values: &[u16]) {
        self.vec.extend_from_slice(values);
    }
}
