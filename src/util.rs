#![allow(missing_docs)]

use std::fmt::Debug;
use std::num::ParseIntError;

use num::traits::{ PrimInt, Num };

pub enum Either<Left, Right> { Left(Left), Right(Right) }

pub trait Halveable {
    type HalfSize: ExtInt;

    fn split(self) -> (Self::HalfSize, Self::HalfSize);
    fn join(h1: Self::HalfSize, h2: Self::HalfSize) -> Self;
}

pub trait To64 { fn to64(self) -> u64; }
pub trait From { fn from<T: To64>(n: T) -> Self; }
pub trait BitLength { fn bits(self) -> usize; }
pub trait ExtInt:
    PrimInt + Num<FromStrRadixErr=ParseIntError>
    + To64 + From + BitLength + Debug { }


impl Halveable for u64 {
    type HalfSize = u32;

    fn split(self) -> (u32, u32) { ((self / 0x1_00_00_00_00u64) as u32, self as u32) }
    fn join(h1: u32, h2: u32) -> u64 { ((h1 as u64) * 0x1_00_00_00_00u64) + (h2 as u64) }
}
impl Halveable for u32 {
    type HalfSize = u16;

    fn split(self) -> (u16, u16) { ((self / 0x1_00_00u32) as u16, self as u16) }
    fn join(h1: u16, h2: u16) -> u32 { ((h1 as u32) * 0x1_00_00u32) + (h2 as u32) }
}
impl Halveable for u16 {
    type HalfSize = u8;

    fn split(self) -> (u8, u8) { ((self / 0x1_00u16) as u8, self as u8) }
    fn join(h1: u8, h2: u8) -> u16 { ((h1 as u16) * 0x1_00u16) + (h2 as u16) }
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

impl BitLength for u64 { #[inline] fn bits(self) -> usize { 64usize } }
impl BitLength for u32 { #[inline] fn bits(self) -> usize { 32usize } }
impl BitLength for u16 { #[inline] fn bits(self) -> usize { 16usize } }
impl BitLength for u8 { #[inline] fn bits(self) -> usize { 8usize } }

impl ExtInt for u64 { }
impl ExtInt for u32 { }
impl ExtInt for u16 { }
impl ExtInt for u8 { }


pub fn cast<T: To64, U: From>(n: T) -> U {
    From::from(n)
}

#[cfg(test)]
mod test {
    use super::{ Halveable };

    #[test]
    fn test_split_u16() {
        assert_eq!((0x00u8, 0x00u8), Halveable::split(0x0000u16));
        assert_eq!((0x00u8, 0x01u8), Halveable::split(0x0001u16));
        assert_eq!((0x00u8, 0xFEu8), Halveable::split(0x00FEu16));
        assert_eq!((0x00u8, 0xFFu8), Halveable::split(0x00FFu16));
        assert_eq!((0x01u8, 0x00u8), Halveable::split(0x0100u16));
        assert_eq!((0x01u8, 0x01u8), Halveable::split(0x0101u16));
        assert_eq!((0xFFu8, 0xFEu8), Halveable::split(0xFFFEu16));
        assert_eq!((0xFFu8, 0xFFu8), Halveable::split(0xFFFFu16));
    }

    #[test]
    fn test_split_u32() {
        assert_eq!((0x0000u16, 0x0000u16), Halveable::split(0x00000000u32));
        assert_eq!((0x0000u16, 0x0001u16), Halveable::split(0x00000001u32));
        assert_eq!((0x0000u16, 0xFFFEu16), Halveable::split(0x0000FFFEu32));
        assert_eq!((0x0000u16, 0xFFFFu16), Halveable::split(0x0000FFFFu32));
        assert_eq!((0x0001u16, 0x0000u16), Halveable::split(0x00010000u32));
        assert_eq!((0x0001u16, 0x0001u16), Halveable::split(0x00010001u32));
        assert_eq!((0xFFFFu16, 0xFFFEu16), Halveable::split(0xFFFFFFFEu32));
        assert_eq!((0xFFFFu16, 0xFFFFu16), Halveable::split(0xFFFFFFFFu32));
    }

    #[test]
    fn test_split_u64() {
        assert_eq!((0x00000000u32, 0x00000000u32), Halveable::split(0x0000000000000000u64));
        assert_eq!((0x00000000u32, 0x00000001u32), Halveable::split(0x0000000000000001u64));
        assert_eq!((0x00000000u32, 0xFFFFFFFEu32), Halveable::split(0x00000000FFFFFFFEu64));
        assert_eq!((0x00000000u32, 0xFFFFFFFFu32), Halveable::split(0x00000000FFFFFFFFu64));
        assert_eq!((0x00000001u32, 0x00000000u32), Halveable::split(0x0000000100000000u64));
        assert_eq!((0x00000001u32, 0x00000001u32), Halveable::split(0x0000000100000001u64));
        assert_eq!((0xFFFFFFFFu32, 0xFFFFFFFEu32), Halveable::split(0xFFFFFFFFFFFFFFFEu64));
        assert_eq!((0xFFFFFFFFu32, 0xFFFFFFFFu32), Halveable::split(0xFFFFFFFFFFFFFFFFu64));
    }

    #[test]
    fn test_join_u16() {
        assert_eq!(0x0000u16, Halveable::join(0x00u8, 0x00u8));
        assert_eq!(0x0001u16, Halveable::join(0x00u8, 0x01u8));
        assert_eq!(0x00FEu16, Halveable::join(0x00u8, 0xFEu8));
        assert_eq!(0x00FFu16, Halveable::join(0x00u8, 0xFFu8));
        assert_eq!(0x0100u16, Halveable::join(0x01u8, 0x00u8));
        assert_eq!(0x0101u16, Halveable::join(0x01u8, 0x01u8));
        assert_eq!(0xFFFEu16, Halveable::join(0xFFu8, 0xFEu8));
        assert_eq!(0xFFFFu16, Halveable::join(0xFFu8, 0xFFu8));
    }

    #[test]
    fn test_join_u32() {
        assert_eq!(0x00000000u32, Halveable::join(0x0000u16, 0x0000u16));
        assert_eq!(0x00000001u32, Halveable::join(0x0000u16, 0x0001u16));
        assert_eq!(0x0000FFFEu32, Halveable::join(0x0000u16, 0xFFFEu16));
        assert_eq!(0x0000FFFFu32, Halveable::join(0x0000u16, 0xFFFFu16));
        assert_eq!(0x00010000u32, Halveable::join(0x0001u16, 0x0000u16));
        assert_eq!(0x00010001u32, Halveable::join(0x0001u16, 0x0001u16));
        assert_eq!(0xFFFFFFFEu32, Halveable::join(0xFFFFu16, 0xFFFEu16));
        assert_eq!(0xFFFFFFFFu32, Halveable::join(0xFFFFu16, 0xFFFFu16));
    }

    #[test]
    fn test_join_u64() {
        assert_eq!(0x0000000000000000u64, Halveable::join(0x00000000u32, 0x00000000u32));
        assert_eq!(0x0000000000000001u64, Halveable::join(0x00000000u32, 0x00000001u32));
        assert_eq!(0x00000000FFFFFFFEu64, Halveable::join(0x00000000u32, 0xFFFFFFFEu32));
        assert_eq!(0x00000000FFFFFFFFu64, Halveable::join(0x00000000u32, 0xFFFFFFFFu32));
        assert_eq!(0x0000000100000000u64, Halveable::join(0x00000001u32, 0x00000000u32));
        assert_eq!(0x0000000100000001u64, Halveable::join(0x00000001u32, 0x00000001u32));
        assert_eq!(0xFFFFFFFFFFFFFFFEu64, Halveable::join(0xFFFFFFFFu32, 0xFFFFFFFEu32));
        assert_eq!(0xFFFFFFFFFFFFFFFFu64, Halveable::join(0xFFFFFFFFu32, 0xFFFFFFFFu32));
    }
}
