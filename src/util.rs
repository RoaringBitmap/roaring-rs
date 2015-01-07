use std::num::Int;

pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

pub trait Halveable { type HalfSize: Int + To64 + From + BitLength; }
pub trait To64 { fn to64(self) -> u64; }
pub trait From { fn from<T: To64>(n: T) -> Self; }
pub trait BitLength { fn bits() -> uint; }
pub trait ExtInt: Int + Halveable + To64 + From + BitLength { }


pub impl Halveable for u64 { type HalfSize = u32; }
pub impl Halveable for u32 { type HalfSize = u16; }
pub impl Halveable for u16 { type HalfSize = u8; }

pub impl To64 for u64 { #[inline] fn to64(self) -> u64 { self } }
pub impl To64 for u32 { #[inline] fn to64(self) -> u64 { self as u64 } }
pub impl To64 for u16 { #[inline] fn to64(self) -> u64 { self as u64 } }
pub impl To64 for u8 { #[inline] fn to64(self) -> u64 { self as u64 } }

pub impl From for uint { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as uint } }
pub impl From for u64 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() } }
pub impl From for u32 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u32 } }
pub impl From for u16 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u16 } }
pub impl From for u8 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u8 } }

pub impl BitLength for u64 { #[inline] fn bits() -> uint { 64u } }
pub impl BitLength for u32 { #[inline] fn bits() -> uint { 32u } }
pub impl BitLength for u16 { #[inline] fn bits() -> uint { 16u } }
pub impl BitLength for u8 { #[inline] fn bits() -> uint { 8u } }

pub impl ExtInt for u64 { }
pub impl ExtInt for u32 { }
pub impl ExtInt for u16 { }

pub fn cast<T: To64, U: From>(n: T) -> U {
    From::from(n)
}

pub fn bits<T: BitLength>() -> uint {
    BitLength::bits()
}
