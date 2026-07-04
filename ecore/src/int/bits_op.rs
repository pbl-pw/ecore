use core::{hint::unreachable_unchecked, marker::PhantomData};

use crate::{
    int::CInt,
    range::{GetIndexRange, IndexRange},
};

use super::{BasicInt, BasicUInt, PrimaryInt};

macro_rules! impl_primary {
    ($type:ty, $bits:ty) => {
        impl BitsOp for $type {
            type Bits = $bits;

            #[inline(always)]
            fn get_raw_int(src: &Self) -> <Self::Bits as BasicInt>::Primary {
                *src as $bits
            }

            #[inline(always)]
            unsafe fn map_raw_int(src: &Self, f: impl FnOnce(<Self::Bits as BasicInt>::Primary) -> <Self::Bits as BasicInt>::Primary) -> Self {
                f(*src as $bits) as Self
            }
        }
    };
}
impl_primary!(u8, u8);
impl_primary!(i8, u8);
impl_primary!(u16, u16);
impl_primary!(i16, u16);
impl_primary!(u32, u32);
impl_primary!(i32, u32);
impl_primary!(u64, u64);
impl_primary!(i64, u64);
impl_primary!(u128, u128);
impl_primary!(i128, u128);
impl_primary!(usize, usize);
impl_primary!(isize, usize);

#[cfg(feature = "bitint")]
impl<T: PrimaryInt, const BITS: u32> BitsOp for crate::bitint::BitInt<T, BITS> {
    type Bits = <Self as BasicInt>::Unsigned;

    #[inline(always)]
    fn get_raw_int(src: &Self) -> <Self::Bits as BasicInt>::Primary {
        src.cast_as_primary().cast_as_unsigned()
    }

    #[inline(always)]
    unsafe fn map_raw_int(src: &Self, f: impl FnOnce(<Self::Bits as BasicInt>::Primary) -> <Self::Bits as BasicInt>::Primary) -> Self {
        unsafe { Self::unchecked_from_primary(BasicInt::cast_from_unsigned(f(src.cast_as_primary().cast_as_unsigned()))) }
    }
}

/// Bit read and write operations.
pub trait BitsOp: Sized {
    type Bits: BasicUInt;

    /// Gets the raw value, including the invariant high bits.
    fn get_raw_int(src: &Self) -> <Self::Bits as BasicInt>::Primary;

    /// Maps the raw value to a new value; the invariant high bits must not be mutated
    /// within the closure.
    ///
    /// # Safety
    ///
    /// The caller must ensure the closure `f` preserves the invariant bits of the raw value.
    unsafe fn map_raw_int(src: &Self, f: impl FnOnce(<Self::Bits as BasicInt>::Primary) -> <Self::Bits as BasicInt>::Primary) -> Self;

    #[inline(always)]
    fn read_raw_bits(src: &Self) -> Self::Bits {
        Self::Bits::cast_from_primary(Self::get_raw_int(src))
    }

    #[inline(always)]
    fn with_raw_bits(src: &Self, bits: Self::Bits) -> Self {
        unsafe { Self::map_raw_int(src, move |sf| sf & !Self::Bits::ALL_ONE.cast_as_primary() | bits.cast_as_primary()) }
    }

    #[inline(always)]
    fn write_raw_bits(src: &mut Self, bits: Self::Bits) {
        *src = Self::with_raw_bits(src, bits);
    }

    #[inline(always)]
    fn clear_raw_bits(src: &mut Self) {
        *src = unsafe { Self::map_raw_int(src, move |sf| sf & !Self::Bits::ALL_ONE.cast_as_primary()) };
    }

    /// Reads the integer value in the specified start-to-end bit range, inclusive.
    /// `<3,4>` and `<4,3>` refer to the same range and produce the same result.
    #[inline(always)]
    fn read_bits<const BITS_BEGIN: u32, const BITS_LAST: u32>(&self) -> Self::Bits {
        Op::<BITS_BEGIN, BITS_LAST, Self>::read_bits(self)
    }

    /// Returns a new value with the specified start-to-end bit range replaced by `bv`,
    /// inclusive. `<3,4>` and `<4,3>` refer to the same range and produce the same result.
    #[inline(always)]
    fn with_bits<const BITS_BEGIN: u32, const BITS_LAST: u32>(&self, bv: Self::Bits) -> Self {
        Op::<BITS_BEGIN, BITS_LAST, Self>::with_bits(self, bv)
    }

    /// Reads the value of the specified bit.
    #[inline(always)]
    fn read_bit<const BIT_NUMBER: u32>(&self) -> bool {
        const { assert!(BIT_NUMBER < Self::Bits::BITS) }
        Self::get_raw_int(self) & Const::<Self>::ONE << BIT_NUMBER != Const::<Self>::ZERO
    }

    /// Returns a new value with the specified bit set to `bv`.
    #[inline(always)]
    fn with_bit<const BIT_NUMBER: u32>(&self, bv: bool) -> Self {
        Op::<BIT_NUMBER, BIT_NUMBER, Self>::with_bits(self, from_bool(bv))
    }

    /// Reads the value of the bit at a runtime-specified position.
    #[inline(always)]
    fn read_bit_i(&self, bitid: impl BasicInt) -> bool {
        (Self::get_raw_int(self) >> valid_bitid::<Self>(bitid)) & Const::<Self>::ONE != Const::<Self>::ZERO
    }

    /// Returns a new value with the bit at a runtime-specified position set to `v`.
    #[inline(always)]
    fn with_bit_i(&self, bitid: impl BasicInt, v: bool) -> Self {
        let bitid = valid_bitid::<Self>(bitid);
        unsafe { Self::map_raw_int(self, move |sf| sf & !(Const::<Self>::ONE << bitid) | from_bool::<<Self::Bits as BasicInt>::Primary>(v) << bitid) }
    }

    /// Reads the bit value in reverse order at a runtime-specified position.
    #[inline(always)]
    fn reverse_read_bit_i(src: &Self, bitid: impl BasicInt) -> bool {
        (Self::get_raw_int(src) << valid_bitid::<Self>(bitid)) & Const::<Self>::MOST_BIT != Const::<Self>::ZERO
    }

    /// Reads the integer value in the specified index range.
    #[inline(always)]
    fn read_bits_i(src: &Self, bitrng: impl GetIndexRange<u32>) -> <Self::Bits as BasicInt>::Primary {
        read_bits_i(Self::get_raw_int(src), GetIndexRange::get_index_range(&bitrng, Self::Bits::BITS))
    }

    /// Returns a new value with the specified index range replaced by `bv`.
    #[inline(always)]
    fn with_bits_i(src: &Self, bitrng: impl GetIndexRange<u32>, bv: <Self::Bits as BasicInt>::Primary) -> Self {
        unsafe { Self::map_raw_int(src, move |sf| with_bits_i(sf, GetIndexRange::get_index_range(&bitrng, Self::Bits::BITS), bv)) }
    }
}

fn read_bits_i<T: PrimaryInt>(src: T, bitrng: IndexRange<u32>) -> T {
    let bits_len = bitrng.len();
    debug_assert!(bits_len != 0);
    let remain_bits = T::BITS - bits_len;
    let left_remain_bits = remain_bits - bitrng.start();
    src << left_remain_bits >> remain_bits
}

fn with_bits_i<T: PrimaryInt>(src: T, bitrng: IndexRange<u32>, bv: T) -> T {
    let bits_len = bitrng.len();
    debug_assert!(bits_len != 0);
    let remain_bits = T::Primary::BITS - bits_len;
    let mask = T::Primary::ALL_ONE >> remain_bits << bitrng.start();
    src & !mask | bv << bitrng.start() & mask
}

#[inline(always)]
const fn from_bool<T: BasicInt>(bv: bool) -> T {
    if bv { T::ONE } else { T::ZERO }
}

#[inline(always)]
fn valid_bitid<T: BitsOp>(bitid: impl BasicInt) -> u32 {
    let Some(bitid) = bitid.cast_as_primary().checked_cast_as::<u32>() else { unsafe { unreachable_unchecked() } };
    debug_assert!(bitid < T::Bits::BITS);
    bitid
}

struct Op<const BITS_BEGIN: u32, const BITS_LAST: u32, T>(PhantomData<T>);

impl<const BITS_BEGIN: u32, const BITS_LAST: u32, T: BitsOp> Op<BITS_BEGIN, BITS_LAST, T> {
    const BEGIN: u32 = if BITS_BEGIN > BITS_LAST { BITS_LAST } else { BITS_BEGIN };
    const LAST: u32 = if BITS_BEGIN > BITS_LAST { BITS_BEGIN } else { BITS_LAST };
    const BITS: u32 = Self::LAST - Self::BEGIN + 1;
    const REMAIN_BITS: u32 = <T::Bits as BasicInt>::Primary::BITS.checked_sub(Self::BITS).expect("bits range overflow");
    const LEFT_REMAIN_BITS: u32 = Self::REMAIN_BITS.checked_sub(Self::BEGIN).expect("bits range overflow");
    const MASK: <T::Bits as BasicInt>::Primary = CInt::shl(CInt::shr(<T::Bits as BasicInt>::Primary::ALL_ONE, Self::REMAIN_BITS), Self::BEGIN);

    #[inline(always)]
    /// todo! T::Bits should be `BitInt<<T::Bits as BasicInt>::Primary, Self::BITS>`
    fn read_bits(v: &T) -> T::Bits {
        unsafe { T::Bits::unchecked_from_primary(BitsOp::get_raw_int(v) << Self::LEFT_REMAIN_BITS >> Self::REMAIN_BITS) }
    }

    #[inline(always)]
    /// todo! T::Bits should be `BitInt<<T::Bits as BasicInt>::Primary, Self::BITS>`
    fn with_bits(v: &T, bv: T::Bits) -> T {
        unsafe { T::map_raw_int(v, move |v| v & !Self::MASK | (bv.cast_as_primary() << Self::BEGIN) & Self::MASK) }
    }
}

struct Const<T>(PhantomData<T>);

impl<T: BitsOp> Const<T> {
    const ONE: <T::Bits as BasicInt>::Primary = BasicInt::ONE;
    const ZERO: <T::Bits as BasicInt>::Primary = BasicInt::ZERO;
    const MOST_BIT: <T::Bits as BasicInt>::Primary = CInt::shl(Self::ONE, T::Bits::BITS - 1);
}

#[cfg(test)]
#[test]
fn test_u32() {
    let a = 0x12345678u32;
    assert_eq!(a.read_bits::<0, 3>(), 8);
    assert_eq!(a.read_bits::<3, 0>(), 8);
    assert_eq!(a.read_bits::<4, 7>(), 7);
    assert_eq!(a.read_bits::<7, 4>(), 7);
    assert_eq!(a.read_bits::<24, 27>(), 2);
    assert_eq!(a.read_bits::<27, 24>(), 2);
    assert_eq!(a.with_bits::<0, 3>(0xa), 0x1234567a);
    assert_eq!(a.with_bits::<3, 0>(0xa), 0x1234567a);
    assert_eq!(a.with_bits::<24, 31>(0xcd), 0xcd345678);
    assert_eq!(a.with_bits::<31, 24>(0xcd), 0xcd345678);

    let b = 0b1001_1010_u8;
    assert!(!b.read_bit::<0>());
    assert!(b.read_bit::<1>());
    assert!(!b.read_bit::<2>());
    assert!(b.read_bit::<3>());
    assert!(b.read_bit::<4>());
    assert!(!b.read_bit::<5>());
    assert!(!b.read_bit::<6>());
    assert!(b.read_bit::<7>());

    assert!(BitsOp::reverse_read_bit_i(&b, 0));
    assert!(!BitsOp::reverse_read_bit_i(&b, 1));
    assert!(!BitsOp::reverse_read_bit_i(&b, 2));
    assert!(BitsOp::reverse_read_bit_i(&b, 3));
    assert!(BitsOp::reverse_read_bit_i(&b, 4));
    assert!(!BitsOp::reverse_read_bit_i(&b, 5));
    assert!(BitsOp::reverse_read_bit_i(&b, 6));
    assert!(!BitsOp::reverse_read_bit_i(&b, 7));
}
