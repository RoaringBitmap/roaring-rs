use std::num::Int;

pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}


pub trait Halveable { type HalfSize: ExtInt; }
pub trait To64 { fn to64(self) -> u64; }
pub trait From { fn from<T: To64>(n: T) -> Self; }
pub trait BitLength { fn bits(self) -> uint; }
pub trait ExtInt: Int + To64 + From + BitLength { }


impl Halveable for u64 { type HalfSize = u32; }
impl Halveable for u32 { type HalfSize = u16; }
impl Halveable for u16 { type HalfSize = u8; }

impl To64 for uint { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u64 { #[inline] fn to64(self) -> u64 { self } }
impl To64 for u32 { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u16 { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u8 { #[inline] fn to64(self) -> u64 { self as u64 } }

impl From for uint { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as uint } }
impl From for u64 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() } }
impl From for u32 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u32 } }
impl From for u16 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u16 } }
impl From for u8 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u8 } }

impl BitLength for u64 { #[inline] fn bits(self) -> uint { 64u } }
impl BitLength for u32 { #[inline] fn bits(self) -> uint { 32u } }
impl BitLength for u16 { #[inline] fn bits(self) -> uint { 16u } }
impl BitLength for u8 { #[inline] fn bits(self) -> uint { 8u } }

impl ExtInt for u64 { }
impl ExtInt for u32 { }
impl ExtInt for u16 { }
impl ExtInt for u8 { }


pub fn cast<T: To64, U: From>(n: T) -> U {
    From::from(n)
}

pub fn bits<T: BitLength + Int>() -> uint {
    let zero: T = Int::zero();
    BitLength::bits(zero)
}
