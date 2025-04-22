#![no_main]

mod arbitrary_ops;

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;

use crate::arbitrary_ops::{check_equal, Operation};

#[derive(Arbitrary, Debug)]
struct FuzzInput<'a> {
    ops: Vec<Operation>,
    initial_input: &'a [u8],
}

fuzz_target!(|input: FuzzInput| {
    let lhs_c = croaring::Bitmap::try_deserialize::<croaring::Portable>(input.initial_input);
    let lhs_r = roaring::RoaringBitmap::deserialize_from(input.initial_input).ok();

    let (mut lhs_c, mut lhs_r) = match (lhs_c, lhs_r) {
        (Some(lhs_c), Some(lhs_r)) => {
            check_equal(&lhs_c, &lhs_r);
            (lhs_c, lhs_r)
        }
        (None, None) => Default::default(),
        (Some(_), None) => panic!("croaring deserialized, but roaring failed"),
        (None, Some(_)) => panic!("roaring deserialized, but croaring failed"),
    };

    let mut rhs_c = croaring::Bitmap::new();
    let mut rhs_r = roaring::RoaringBitmap::new();

    for op in input.ops {
        op.apply(&mut lhs_c, &mut rhs_c, &mut lhs_r, &mut rhs_r);
    }
    check_equal(&lhs_c, &lhs_r);
    check_equal(&rhs_c, &rhs_r);
});
