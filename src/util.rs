use std::{ u8, u16, u32 };
use std::num::Int;

pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}


pub trait Halveable {
    type HalfSize: ExtInt;
    fn split(self) -> (Self::HalfSize, Self::HalfSize);
    fn join(h1: Self::HalfSize, h2: Self::HalfSize) -> Self;
}

pub trait To64 { fn to64(self) -> u64; }
pub trait From { fn from<T: To64>(n: T) -> Self; }
pub trait BitLength { fn bits(self) -> usize; }
pub trait ExtInt: Int + To64 + From + BitLength { }


impl Halveable for u64 {
    type HalfSize = u32;
    fn split(self) -> (u32, u32) {
        ((self >> u32::BITS) as u32, self as u32)
    }
    fn join(h1: u32, h2: u32) -> u64 {
        ((h1 as u64) << u32::BITS as u64) + (h2 as u64)
    }
}
impl Halveable for u32 {
    type HalfSize = u16;
    fn split(self) -> (u16, u16) {
        ((self >> u16::BITS) as u16, self as u16)
    }
    fn join(h1: u16, h2: u16) -> u32 {
        ((h1 as u32) << u16::BITS as u32) + (h2 as u32)
    }
}
impl Halveable for u16 {
    type HalfSize = u8;
    fn split(self) -> (u8, u8) {
        ((self >> u8::BITS) as u8, self as u8)
    }
    fn join(h1: u8, h2: u8) -> u16 {
        ((h1 as u16) << u8::BITS as u16) + (h2 as u16)
    }
}

impl To64 for usize { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u64 { #[inline] fn to64(self) -> u64 { self } }
impl To64 for u32 { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u16 { #[inline] fn to64(self) -> u64 { self as u64 } }
impl To64 for u8 { #[inline] fn to64(self) -> u64 { self as u64 } }

impl From for usize { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as usize } }
impl From for u64 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() } }
impl From for u32 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u32 } }
impl From for u16 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u16 } }
impl From for u8 { #[inline] fn from<T: To64>(n: T) -> Self { n.to64() as u8 } }

impl BitLength for u64 { #[inline] fn bits(self) -> usize { 64us } }
impl BitLength for u32 { #[inline] fn bits(self) -> usize { 32us } }
impl BitLength for u16 { #[inline] fn bits(self) -> usize { 16us } }
impl BitLength for u8 { #[inline] fn bits(self) -> usize { 8us } }

impl ExtInt for u64 { }
impl ExtInt for u32 { }
impl ExtInt for u16 { }
impl ExtInt for u8 { }


pub fn cast<T: To64, U: From>(n: T) -> U {
    From::from(n)
}

pub fn bits<T: BitLength + Int>() -> usize {
    let zero: T = Int::zero();
    BitLength::bits(zero)
}
