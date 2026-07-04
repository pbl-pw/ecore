use bytemuck::Pod;
use const_default::ConstDefault;

use crate::{Array, repr::ConvertEndian};

use core::{
    fmt::{Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex},
    hash::Hash,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl,
        ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

pub use self::{bits_op::BitsOp, cint::CInt};

#[cfg(feature = "varint")]
pub use self::varint::VarInt;

#[cfg(feature = "ranged-int")]
pub mod ranged;

/// A potentially non-zero integer type, such as [i8], [NonZero\<i8>], [u8], [NonZero\<u8>], ...
pub trait MaybeZeroableInt: Copy + sealed::Sealed {
    type Underlying: PrimaryInt;
    const ZEROABLE: bool = true;
}

/// A type that fits the machine's computation width after adapting the actual type,
/// used to reduce code bloat from generics. Includes `usize` and `isize`, plus:
/// * When `usize::BITS == 32`, also includes `u64`, `i64`, `u128`, and `i128`.
/// * When `usize::BITS == 64`, also includes `u128` and `i128`.
pub trait CalcFitted: sealed::Sealed {
    #[cfg(feature = "varint")]
    type VarIntBuf: Copy;
}

/// Basic integer type, including u1, u2, u3, ..., u128, usize and i1, i2, i3, ..., i128, isize.
/// Extended to include `bitint::BitInt<T, BITS>` (`BITS in 0..=T::BITS-1`); the non-native
/// `uN` and `iN` series are actually aliases for `bitint::BitInt`.
/// In generic functions, the integer category can be determined by checking associated constants:
/// * `Self::BITS == Self::Primary::BITS` indicates a native integer type.
/// * `Self::BITS < Self::Primary::BITS` indicates `bitint::BitInt`; if further
///   `Self::BITS > Self::Primary::BITS / 16 * 8`, it indicates a non-native `uN` or `iN` series.
pub trait BasicInt:
    'static
    + sealed::Sealed
    + Add<Output = Self>
    + AddAssign
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Default
    + Div<Output = Self>
    + DivAssign
    + Eq
    + Mul<Output = Self>
    + MulAssign
    + Not<Output = Self>
    + Ord
    + Rem<Output = Self>
    + RemAssign
    + Shl<u32, Output = Self>
    + ShlAssign<u32>
    + Shr<u32, Output = Self>
    + ShrAssign<u32>
    + Shl<usize, Output = Self>
    + ShlAssign<usize>
    + Shr<usize, Output = Self>
    + ShrAssign<usize>
    + Sub<Output = Self>
    + SubAssign
    + BitsOp
    + Binary
    + Copy
    + Debug
    + Display
    + Hash
    + Octal
    + UpperHex
    + LowerHex
    + LowerExp
    + UpperExp
    + Unpin
    + ConstDefault
{
    const BITS: u32;
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const ALL_ONE: Self;
    const SIGNED: bool;
    type Primary: PrimaryInt<SignedPrimary = <Self::Signed as BasicInt>::Primary, UnsignedPrimary = <Self::Unsigned as BasicInt>::Primary, CalcFitted = Self::CalcFitted>;
    type Signed: BasicSInt<Signed = Self::Signed, Unsigned = Self::Unsigned>;
    type Unsigned: BasicUInt<Signed = Self::Signed, Unsigned = Self::Unsigned>;
    /// The type after adapting to the machine's computation width.
    type CalcFitted: PrimaryInt<CalcFitted = Self::CalcFitted> + CalcFitted;

    fn abs_diff(self, other: Self) -> Self::Unsigned;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn checked_div_euclid(self, other: Self) -> Option<Self>;
    fn checked_ilog(self, base: Self) -> Option<u32>;
    fn checked_ilog2(self) -> Option<u32>;
    fn checked_ilog10(self) -> Option<u32>;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_neg(self) -> Option<Self>;
    fn checked_pow(self, exp: u32) -> Option<Self>;
    fn checked_rem(self, other: Self) -> Option<Self>;
    fn checked_rem_euclid(self, other: Self) -> Option<Self>;
    fn checked_shl(self, shift: u32) -> Option<Self>;
    fn checked_shr(self, shift: u32) -> Option<Self>;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn count_ones(self) -> u32;
    fn count_zeros(self) -> u32;
    fn div_euclid(self, other: Self) -> Self;
    fn ilog(self, base: Self) -> u32;
    fn ilog2(self) -> u32;
    fn ilog10(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn leading_ones(self) -> u32;
    fn midpoint(self, other: Self) -> Self;
    fn overflowing_add(self, other: Self) -> (Self, bool);
    fn overflowing_div(self, other: Self) -> (Self, bool);
    fn overflowing_div_euclid(self, other: Self) -> (Self, bool);
    fn overflowing_mul(self, other: Self) -> (Self, bool);
    fn overflowing_neg(self) -> (Self, bool);
    fn overflowing_pow(self, exp: u32) -> (Self, bool);
    fn overflowing_rem(self, other: Self) -> (Self, bool);
    fn overflowing_rem_euclid(self, other: Self) -> (Self, bool);
    fn overflowing_shl(self, shift: u32) -> (Self, bool);
    fn overflowing_shr(self, shift: u32) -> (Self, bool);
    fn overflowing_sub(self, other: Self) -> (Self, bool);
    fn pow(self, exp: u32) -> Self;
    fn rem_euclid(self, other: Self) -> Self;
    fn reverse_bits(self) -> Self;
    fn rotate_left(self, shift: u32) -> Self;
    fn rotate_right(self, shift: u32) -> Self;
    fn saturating_add(self, other: Self) -> Self;
    fn saturating_div(self, other: Self) -> Self;
    fn saturating_mul(self, other: Self) -> Self;
    fn saturating_pow(self, exp: u32) -> Self;
    fn saturating_sub(self, other: Self) -> Self;
    fn trailing_ones(self) -> u32;
    fn trailing_zeros(self) -> u32;
    fn unbounded_shl(self, shift: u32) -> Self;
    fn unbounded_shr(self, shift: u32) -> Self;
    /// # Safety
    ///
    /// The caller must ensure the operation does not overflow.
    unsafe fn unchecked_add(self, other: Self) -> Self;
    /// # Safety
    ///
    /// The caller must ensure the operation does not overflow.
    unsafe fn unchecked_mul(self, other: Self) -> Self;
    /// # Safety
    ///
    /// The caller must ensure the operation does not overflow.
    unsafe fn unchecked_sub(self, other: Self) -> Self;
    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_div(self, other: Self) -> Self;
    fn wrapping_div_euclid(self, other: Self) -> Self;
    fn wrapping_mul(self, other: Self) -> Self;
    fn wrapping_neg(self) -> Self;
    fn wrapping_pow(self, exp: u32) -> Self;
    fn wrapping_rem(self, other: Self) -> Self;
    fn wrapping_rem_euclid(self, other: Self) -> Self;
    fn wrapping_shl(self, shift: u32) -> Self;
    fn wrapping_shr(self, shift: u32) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;

    #[inline(always)]
    fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    fn cast_as_primary(self) -> Self::Primary;
    fn cast_from_primary(src: Self::Primary) -> Self;
    /// # Safety
    ///
    /// The caller must ensure `src` represents a valid value of `Self`.
    unsafe fn unchecked_from_primary(src: Self::Primary) -> Self;

    fn cast_as_signed(self) -> Self::Signed;
    fn cast_from_signed(src: Self::Signed) -> Self;

    fn cast_as_unsigned(self) -> Self::Unsigned;
    fn cast_from_unsigned(src: Self::Unsigned) -> Self;

    fn cast_as_calc(self) -> Self::CalcFitted;
    fn cast_from_calc(src: Self::CalcFitted) -> Self;

    #[inline(always)]
    /// Creates a basic integer value from a `bool` using `as` rules.
    /// **Except for `i1` (`true` maps to `-1`) or `Self::BITS == 0` (result is always `0`),
    /// in all other cases `true` maps to `1` and `false` maps to `0`.**
    fn cast_from_bool(src: bool) -> Self {
        if Self::BITS != 0 && src { if Self::SIGNED && Self::BITS == 1 { Self::MIN } else { Self::ONE } } else { Self::ZERO }
    }

    /// Converts between basic integer types using `as` casting rules.
    /// When converting to/from an associated type, prefer `cast_as_<name>` or
    /// `cast_from_<name>` for optimal performance.
    #[inline(always)]
    fn cast_as<Tgt: BasicInt>(self) -> Tgt {
        base_impl::cast_as(self)
    }

    #[inline(always)]
    /// Safely converts a basic integer type to another basic integer type.
    fn checked_cast_as<Tgt: BasicInt>(self) -> Option<Tgt> {
        base_impl::checked_cast_as(self)
    }

    #[inline(always)]
    fn is_multiple_of(self, other: Self) -> bool {
        other.cast_as_primary() > Self::Primary::ZERO && self.cast_as_primary() % other.cast_as_primary() == Self::Primary::ZERO
    }
}

/// Basic unsigned integer type, including u1, u2, u3, ..., u128, usize.
/// Extended to include `bitint::BitInt<u8, BITS>`, `bitint::BitInt<u16, BITS>`,
/// ..., `bitint::BitInt<usize, BITS>`.
pub trait BasicUInt: BasicInt<Unsigned = Self, CalcFitted = Self::UnsignedCalcFitted> {
    type UnsignedCalcFitted: PrimaryUInt<UnsignedCalcFitted = Self::UnsignedCalcFitted> + CalcFitted;

    fn cast_signed(self) -> Self::Signed;
    fn checked_add_signed(self, other: Self::Signed) -> Option<Self>;
    fn checked_next_multiple_of(self, other: Self) -> Option<Self>;
    fn checked_next_power_of_two(self) -> Option<Self>;
    fn div_ceil(self, other: Self) -> Self;
    fn is_power_of_two(self) -> bool;
    fn next_multiple_of(self, other: Self) -> Self;
    fn next_power_of_two(self) -> Self;
    fn overflowing_add_signed(self, other: Self::Signed) -> (Self, bool);
    fn saturating_add_signed(self, other: Self::Signed) -> Self;
    fn wrapping_add_signed(self, other: Self::Signed) -> Self;
}

/// Basic signed integer type, including i1, i2, i3, ..., i128, isize.
/// Extended to include `bitint::BitInt<i8, BITS>`, `bitint::BitInt<i16, BITS>`,
/// ..., `bitint::BitInt<isize, BITS>`.
pub trait BasicSInt: BasicInt<Signed = Self, CalcFitted = Self::SignedCalcFitted> + Neg<Output = Self> {
    type SignedCalcFitted: PrimarySInt<SignedCalcFitted = Self::SignedCalcFitted> + CalcFitted;

    fn abs(self) -> Self;
    fn cast_unsigned(self) -> Self::Unsigned;
    fn checked_abs(self) -> Option<Self>;
    fn checked_add_unsigned(self, other: Self::Unsigned) -> Option<Self>;
    fn checked_sub_unsigned(self, other: Self::Unsigned) -> Option<Self>;
    fn is_negative(self) -> bool;
    fn is_positive(self) -> bool;
    fn overflowing_abs(self) -> (Self, bool);
    fn overflowing_add_unsigned(self, other: Self::Unsigned) -> (Self, bool);
    fn overflowing_sub_unsigned(self, other: Self::Unsigned) -> (Self, bool);
    fn saturating_abs(self) -> Self;
    fn saturating_add_unsigned(self, other: Self::Unsigned) -> Self;
    fn saturating_neg(self) -> Self;
    fn saturating_sub_unsigned(self, other: Self::Unsigned) -> Self;
    fn signum(self) -> Self::Primary;
    fn unsigned_abs(self) -> Self::Unsigned;
    fn wrapping_abs(self) -> Self;
    fn wrapping_add_unsigned(self, other: Self::Unsigned) -> Self;
    fn wrapping_sub_unsigned(self, other: Self::Unsigned) -> Self;
}

/// Native integer type, including u8, u16, u32, u64, u128, usize and
/// i8, i16, i32, i64, i128, isize.
pub trait PrimaryInt:
    BasicInt<Primary = Self, Signed = Self::SignedPrimary, Unsigned = Self::UnsignedPrimary> + Pod + MaybeZeroableInt<Underlying = Self> + ConvertEndian
{
    /// `[u8; size_of::<Self>()]`
    type BytesArray: Pod + Array<E = u8>;
    /// Effectively `Self::Signed`, defined to resolve the cyclic trait bound
    /// `trait PrimaryInt: BasicInt<Primary = Self, Signed: PrimarySInt, Unsigned: PrimaryUInt>{}`.
    type SignedPrimary: PrimarySInt<SignedPrimary = Self::SignedPrimary, UnsignedPrimary = Self::UnsignedPrimary>;
    /// Effectively `Self::Unsigned`, defined to resolve the cyclic trait bound
    /// `trait PrimaryInt: BasicInt<Primary = Self, Signed: PrimarySInt, Unsigned: PrimaryUInt>{}`.
    type UnsignedPrimary: PrimaryUInt<SignedPrimary = Self::SignedPrimary, UnsignedPrimary = Self::UnsignedPrimary>;

    fn swap_bytes(self) -> Self;
    fn from_be(src: Self) -> Self;
    fn from_be_bytes(bytes: Self::BytesArray) -> Self;
    fn from_le(src: Self) -> Self;
    fn from_le_bytes(bytes: Self::BytesArray) -> Self;
    fn from_ne_bytes(bytes: Self::BytesArray) -> Self;
    fn to_be(self) -> Self;
    fn to_be_bytes(self) -> Self::BytesArray;
    fn to_le(self) -> Self;
    fn to_le_bytes(self) -> Self::BytesArray;
    fn to_ne_bytes(self) -> Self::BytesArray;
}

/// Native unsigned integer type, including u8, u16, u32, u64, u128, usize.
pub trait PrimaryUInt: PrimaryInt<UnsignedPrimary = Self> + BasicUInt {}

/// Native signed integer type, including i8, i16, i32, i64, i128, isize.
pub trait PrimarySInt: PrimaryInt<SignedPrimary = Self> + BasicSInt {}

mod base_impl;
mod bits_op;
mod cint;
#[cfg(feature = "varint")]
mod varint;

mod sealed {
    pub trait Sealed {}

    impl<T: super::PrimaryInt, const BITS: u32> Sealed for crate::bitint::BitInt<T, BITS> {}
}
