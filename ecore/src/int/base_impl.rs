use core::num::NonZero;

use super::{BasicInt, BasicSInt, BasicUInt, CalcFitted, MaybeZeroableInt, PrimaryInt, PrimarySInt, PrimaryUInt, sealed::Sealed};

#[cfg(feature = "varint")]
use super::VarInt;

macro_rules! basic_comm_methods {
    () => {
        #[inline(always)]
        fn abs_diff(self, other: Self) -> Self::Unsigned {
            Self::abs_diff(self, other)
        }

        #[inline(always)]
        fn checked_add(self, other: Self) -> Option<Self> {
            Self::checked_add(self, other)
        }

        #[inline(always)]
        fn checked_div(self, other: Self) -> Option<Self> {
            Self::checked_div(self, other)
        }

        #[inline(always)]
        fn checked_div_euclid(self, other: Self) -> Option<Self> {
            Self::checked_div_euclid(self, other)
        }

        #[inline(always)]
        fn checked_ilog(self, base: Self) -> Option<u32> {
            Self::checked_ilog(self, base)
        }

        #[inline(always)]
        fn checked_ilog2(self) -> Option<u32> {
            Self::checked_ilog2(self)
        }

        #[inline(always)]
        fn checked_ilog10(self) -> Option<u32> {
            Self::checked_ilog10(self)
        }

        #[inline(always)]
        fn checked_mul(self, other: Self) -> Option<Self> {
            Self::checked_mul(self, other)
        }

        #[inline(always)]
        fn checked_neg(self) -> Option<Self> {
            Self::checked_neg(self)
        }

        #[inline(always)]
        fn checked_pow(self, exp: u32) -> Option<Self> {
            Self::checked_pow(self, exp)
        }

        #[inline(always)]
        fn checked_rem(self, other: Self) -> Option<Self> {
            Self::checked_rem(self, other)
        }

        #[inline(always)]
        fn checked_rem_euclid(self, other: Self) -> Option<Self> {
            Self::checked_rem_euclid(self, other)
        }

        #[inline(always)]
        fn checked_shl(self, shift: u32) -> Option<Self> {
            Self::checked_shl(self, shift)
        }

        #[inline(always)]
        fn checked_shr(self, shift: u32) -> Option<Self> {
            Self::checked_shr(self, shift)
        }

        #[inline(always)]
        fn checked_sub(self, other: Self) -> Option<Self> {
            Self::checked_sub(self, other)
        }

        #[inline(always)]
        fn count_ones(self) -> u32 {
            Self::count_ones(self)
        }

        #[inline(always)]
        fn count_zeros(self) -> u32 {
            Self::count_zeros(self)
        }

        #[inline(always)]
        fn div_euclid(self, other: Self) -> Self {
            Self::div_euclid(self, other)
        }

        #[inline(always)]
        #[track_caller]
        fn ilog(self, base: Self) -> u32 {
            Self::ilog(self, base)
        }

        #[inline(always)]
        fn ilog2(self) -> u32 {
            Self::ilog2(self)
        }

        #[inline(always)]
        fn ilog10(self) -> u32 {
            Self::ilog10(self)
        }

        #[inline(always)]
        fn leading_zeros(self) -> u32 {
            Self::leading_zeros(self)
        }

        #[inline(always)]
        fn leading_ones(self) -> u32 {
            Self::leading_ones(self)
        }

        #[inline(always)]
        fn midpoint(self, other: Self) -> Self {
            Self::midpoint(self, other)
        }

        #[inline(always)]
        fn overflowing_add(self, other: Self) -> (Self, bool) {
            Self::overflowing_add(self, other)
        }

        #[inline(always)]
        fn overflowing_div(self, other: Self) -> (Self, bool) {
            Self::overflowing_div(self, other)
        }

        #[inline(always)]
        fn overflowing_div_euclid(self, other: Self) -> (Self, bool) {
            Self::overflowing_div_euclid(self, other)
        }

        #[inline(always)]
        fn overflowing_mul(self, other: Self) -> (Self, bool) {
            Self::overflowing_mul(self, other)
        }

        #[inline(always)]
        fn overflowing_neg(self) -> (Self, bool) {
            Self::overflowing_neg(self)
        }

        #[inline(always)]
        fn overflowing_pow(self, exp: u32) -> (Self, bool) {
            Self::overflowing_pow(self, exp)
        }

        #[inline(always)]
        fn overflowing_rem(self, other: Self) -> (Self, bool) {
            Self::overflowing_rem(self, other)
        }

        #[inline(always)]
        fn overflowing_rem_euclid(self, other: Self) -> (Self, bool) {
            Self::overflowing_rem_euclid(self, other)
        }

        #[inline(always)]
        fn overflowing_shl(self, shift: u32) -> (Self, bool) {
            Self::overflowing_shl(self, shift)
        }

        #[inline(always)]
        fn overflowing_shr(self, shift: u32) -> (Self, bool) {
            Self::overflowing_shr(self, shift)
        }

        #[inline(always)]
        fn overflowing_sub(self, other: Self) -> (Self, bool) {
            Self::overflowing_sub(self, other)
        }

        #[inline(always)]
        #[track_caller]
        fn pow(self, exp: u32) -> Self {
            Self::pow(self, exp)
        }

        #[inline(always)]
        fn rem_euclid(self, other: Self) -> Self {
            Self::rem_euclid(self, other)
        }

        #[inline(always)]
        fn reverse_bits(self) -> Self {
            Self::reverse_bits(self)
        }

        #[inline(always)]
        fn rotate_left(self, shift: u32) -> Self {
            Self::rotate_left(self, shift)
        }

        #[inline(always)]
        fn rotate_right(self, shift: u32) -> Self {
            Self::rotate_right(self, shift)
        }

        #[inline(always)]
        fn saturating_add(self, other: Self) -> Self {
            Self::saturating_add(self, other)
        }

        #[inline(always)]
        fn saturating_div(self, other: Self) -> Self {
            Self::saturating_div(self, other)
        }

        #[inline(always)]
        fn saturating_mul(self, other: Self) -> Self {
            Self::saturating_mul(self, other)
        }

        #[inline(always)]
        fn saturating_pow(self, exp: u32) -> Self {
            Self::saturating_pow(self, exp)
        }

        #[inline(always)]
        fn saturating_sub(self, other: Self) -> Self {
            Self::saturating_sub(self, other)
        }

        #[inline(always)]
        fn trailing_ones(self) -> u32 {
            Self::trailing_ones(self)
        }

        #[inline(always)]
        fn trailing_zeros(self) -> u32 {
            Self::trailing_zeros(self)
        }

        #[inline(always)]
        fn unbounded_shl(self, shift: u32) -> Self {
            Self::unbounded_shl(self, shift)
        }

        #[inline(always)]
        fn unbounded_shr(self, shift: u32) -> Self {
            Self::unbounded_shr(self, shift)
        }

        #[inline(always)]
        unsafe fn unchecked_add(self, other: Self) -> Self {
            unsafe { Self::unchecked_add(self, other) }
        }

        #[inline(always)]
        unsafe fn unchecked_mul(self, other: Self) -> Self {
            unsafe { Self::unchecked_mul(self, other) }
        }

        #[inline(always)]
        unsafe fn unchecked_sub(self, other: Self) -> Self {
            unsafe { Self::unchecked_sub(self, other) }
        }

        #[inline(always)]
        fn wrapping_add(self, other: Self) -> Self {
            Self::wrapping_add(self, other)
        }

        #[inline(always)]
        fn wrapping_div(self, other: Self) -> Self {
            Self::wrapping_div(self, other)
        }

        #[inline(always)]
        fn wrapping_div_euclid(self, other: Self) -> Self {
            Self::wrapping_div_euclid(self, other)
        }

        #[inline(always)]
        fn wrapping_mul(self, other: Self) -> Self {
            Self::wrapping_mul(self, other)
        }

        #[inline(always)]
        fn wrapping_neg(self) -> Self {
            Self::wrapping_neg(self)
        }

        #[inline(always)]
        fn wrapping_pow(self, exp: u32) -> Self {
            Self::wrapping_pow(self, exp)
        }

        #[inline(always)]
        fn wrapping_rem(self, other: Self) -> Self {
            Self::wrapping_rem(self, other)
        }

        #[inline(always)]
        fn wrapping_rem_euclid(self, other: Self) -> Self {
            Self::wrapping_rem_euclid(self, other)
        }

        #[inline(always)]
        fn wrapping_shl(self, shift: u32) -> Self {
            Self::wrapping_shl(self, shift)
        }

        #[inline(always)]
        fn wrapping_shr(self, shift: u32) -> Self {
            Self::wrapping_shr(self, shift)
        }

        #[inline(always)]
        fn wrapping_sub(self, other: Self) -> Self {
            Self::wrapping_sub(self, other)
        }

        #[inline(always)]
        fn cast_as_primary(self) -> Self::Primary {
            self
        }

        #[inline(always)]
        fn cast_from_primary(src: Self::Primary) -> Self {
            src
        }

        #[inline(always)]
        unsafe fn unchecked_from_primary(src: Self::Primary) -> Self {
            src
        }

        #[inline(always)]
        fn cast_as_signed(self) -> Self::Signed {
            self as Self::Signed
        }

        #[inline(always)]
        fn cast_from_signed(src: Self::Signed) -> Self {
            src as Self
        }

        #[inline(always)]
        fn cast_as_unsigned(self) -> Self::Unsigned {
            self as Self::Unsigned
        }

        #[inline(always)]
        fn cast_from_unsigned(src: Self::Unsigned) -> Self {
            src as Self
        }

        #[inline(always)]
        fn cast_as_calc(self) -> Self::CalcFitted {
            self as Self::CalcFitted
        }

        #[inline(always)]
        fn cast_from_calc(src: Self::CalcFitted) -> Self {
            src as Self
        }
    };
}

macro_rules! impl_basic {
    ($u:ty, $s:ty, $uptype:ty, $sptype:ty) => {
        impl Sealed for $s {}
        impl BasicInt for $s {
            const BITS: u32 = <$s>::BITS;
            const MIN: Self = <$s>::MIN;
            const MAX: Self = <$s>::MAX;
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const ALL_ONE: Self = Self::MIN | Self::MAX;
            const SIGNED: bool = true;
            type Primary = Self;
            type Signed = $s;
            type Unsigned = $u;
            type CalcFitted = $sptype;

            basic_comm_methods!();
        }
        impl BasicSInt for $s {
            type SignedCalcFitted = $sptype;

            #[inline(always)]
            fn abs(self) -> Self {
                Self::abs(self)
            }

            #[inline(always)]
            fn cast_unsigned(self) -> Self::Unsigned {
                Self::cast_unsigned(self)
            }

            #[inline(always)]
            fn checked_abs(self) -> Option<Self> {
                Self::checked_abs(self)
            }

            #[inline(always)]
            fn checked_add_unsigned(self, other: Self::Unsigned) -> Option<Self> {
                Self::checked_add_unsigned(self, other)
            }

            #[inline(always)]
            fn checked_sub_unsigned(self, other: Self::Unsigned) -> Option<Self> {
                Self::checked_sub_unsigned(self, other)
            }

            #[inline(always)]
            fn is_negative(self) -> bool {
                Self::is_negative(self)
            }

            #[inline(always)]
            fn is_positive(self) -> bool {
                Self::is_positive(self)
            }

            #[inline(always)]
            fn overflowing_abs(self) -> (Self, bool) {
                Self::overflowing_abs(self)
            }

            #[inline(always)]
            fn overflowing_add_unsigned(self, other: Self::Unsigned) -> (Self, bool) {
                Self::overflowing_add_unsigned(self, other)
            }

            #[inline(always)]
            fn overflowing_sub_unsigned(self, other: Self::Unsigned) -> (Self, bool) {
                Self::overflowing_sub_unsigned(self, other)
            }

            #[inline(always)]
            fn saturating_abs(self) -> Self {
                Self::saturating_abs(self)
            }

            #[inline(always)]
            fn saturating_add_unsigned(self, other: Self::Unsigned) -> Self {
                Self::saturating_add_unsigned(self, other)
            }

            #[inline(always)]
            fn saturating_neg(self) -> Self {
                Self::saturating_neg(self)
            }

            #[inline(always)]
            fn saturating_sub_unsigned(self, other: Self::Unsigned) -> Self {
                Self::saturating_sub_unsigned(self, other)
            }

            #[inline(always)]
            fn signum(self) -> Self {
                Self::signum(self)
            }

            #[inline(always)]
            fn unsigned_abs(self) -> Self::Unsigned {
                Self::unsigned_abs(self)
            }

            #[inline(always)]
            fn wrapping_abs(self) -> Self {
                Self::wrapping_abs(self)
            }

            #[inline(always)]
            fn wrapping_add_unsigned(self, other: Self::Unsigned) -> Self {
                Self::wrapping_add_unsigned(self, other)
            }

            #[inline(always)]
            fn wrapping_sub_unsigned(self, other: Self::Unsigned) -> Self {
                Self::wrapping_sub_unsigned(self, other)
            }
        }
        impl PrimarySInt for $s {}

        impl Sealed for $u {}
        impl BasicInt for $u {
            const BITS: u32 = <$u>::BITS;
            const MIN: Self = <$u>::MIN;
            const MAX: Self = <$u>::MAX;
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const ALL_ONE: Self = Self::MAX;
            const SIGNED: bool = false;
            type Primary = Self;
            type Signed = $s;
            type Unsigned = $u;
            type CalcFitted = $uptype;

            basic_comm_methods!();
        }
        impl BasicUInt for $u {
            type UnsignedCalcFitted = $uptype;

            #[inline(always)]
            fn cast_signed(self) -> Self::Signed {
                Self::cast_signed(self)
            }

            #[inline(always)]
            fn checked_add_signed(self, other: Self::Signed) -> Option<Self> {
                Self::checked_add_signed(self, other)
            }

            #[inline(always)]
            fn checked_next_multiple_of(self, other: Self) -> Option<Self> {
                Self::checked_next_multiple_of(self, other)
            }

            #[inline(always)]
            fn checked_next_power_of_two(self) -> Option<Self> {
                Self::checked_next_power_of_two(self)
            }

            #[inline(always)]
            fn div_ceil(self, other: Self) -> Self {
                Self::div_ceil(self, other)
            }

            #[inline(always)]
            fn is_power_of_two(self) -> bool {
                Self::is_power_of_two(self)
            }

            #[inline(always)]
            fn next_multiple_of(self, other: Self) -> Self {
                Self::next_multiple_of(self, other)
            }

            #[inline(always)]
            fn next_power_of_two(self) -> Self {
                Self::next_power_of_two(self)
            }

            #[inline(always)]
            fn overflowing_add_signed(self, other: Self::Signed) -> (Self, bool) {
                Self::overflowing_add_signed(self, other)
            }

            #[inline(always)]
            fn saturating_add_signed(self, other: Self::Signed) -> Self {
                Self::saturating_add_signed(self, other)
            }

            #[inline(always)]
            fn wrapping_add_signed(self, other: Self::Signed) -> Self {
                Self::wrapping_add_signed(self, other)
            }
        }
        impl PrimaryUInt for $u {}
    };
}
impl_basic!(u8, i8, usize, isize);
impl_basic!(u16, i16, usize, isize);
#[cfg(target_pointer_width = "16")]
impl_basic!(u32, i32, u32, i32);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_basic!(u32, i32, usize, isize);
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
impl_basic!(u64, i64, u64, i64);
#[cfg(target_pointer_width = "64")]
impl_basic!(u64, i64, usize, isize);
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))]
impl_basic!(u128, i128, u128, i128);
impl_basic!(usize, isize, usize, isize);

macro_rules! impl_primary {
    ($type:ty) => {
        impl PrimaryInt for $type {
            type BytesArray = [u8; size_of::<Self>()];
            type SignedPrimary = Self::Signed;
            type UnsignedPrimary = Self::Unsigned;

            #[inline(always)]
            fn swap_bytes(self) -> Self {
                Self::swap_bytes(self)
            }

            #[inline(always)]
            fn from_be(src: Self) -> Self {
                Self::from_be(src)
            }

            #[inline(always)]
            fn from_be_bytes(bytes: Self::BytesArray) -> Self {
                Self::from_be_bytes(bytes)
            }

            #[inline(always)]
            fn from_le(src: Self) -> Self {
                Self::from_le(src)
            }

            #[inline(always)]
            fn from_le_bytes(bytes: Self::BytesArray) -> Self {
                Self::from_le_bytes(bytes)
            }

            #[inline(always)]
            fn from_ne_bytes(bytes: Self::BytesArray) -> Self {
                Self::from_ne_bytes(bytes)
            }

            #[inline(always)]
            fn to_be(self) -> Self {
                Self::to_be(self)
            }

            #[inline(always)]
            fn to_be_bytes(self) -> Self::BytesArray {
                Self::to_be_bytes(self)
            }

            #[inline(always)]
            fn to_le(self) -> Self {
                Self::to_le(self)
            }

            #[inline(always)]
            fn to_le_bytes(self) -> Self::BytesArray {
                Self::to_le_bytes(self)
            }

            #[inline(always)]
            fn to_ne_bytes(self) -> Self::BytesArray {
                Self::to_ne_bytes(self)
            }
        }

        impl MaybeZeroableInt for $type {
            type Underlying = $type;
        }

        impl Sealed for NonZero<$type> {}

        impl MaybeZeroableInt for NonZero<$type> {
            type Underlying = $type;
            const ZEROABLE: bool = false;
        }
    };
}
impl_primary!(u8);
impl_primary!(i8);
impl_primary!(u16);
impl_primary!(i16);
impl_primary!(u32);
impl_primary!(i32);
impl_primary!(u64);
impl_primary!(i64);
impl_primary!(u128);
impl_primary!(i128);
impl_primary!(usize);
impl_primary!(isize);

impl CalcFitted for usize {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}
impl CalcFitted for isize {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}

#[cfg(target_pointer_width = "16")]
impl CalcFitted for u32 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}
#[cfg(target_pointer_width = "16")]
impl CalcFitted for i32 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}

#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
impl CalcFitted for u64 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
impl CalcFitted for i64 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}

#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))]
impl CalcFitted for u128 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))]
impl CalcFitted for i128 {
    #[cfg(feature = "varint")]
    type VarIntBuf = [u8; VarInt::max_bytes_of::<Self>()];
}

/// Converts between basic integer types using `as` casting rules.
/// Any two of i1, i2, i3, ..., i128, isize, u1, u2, u3, ..., u128, usize can be
/// mutually converted.
#[inline(always)]
pub fn cast_as<Src: BasicInt, Tgt: BasicInt>(src: Src) -> Tgt {
    Tgt::cast_from_primary(cast_as_imp(src.cast_as_primary()))
}

#[inline(always)]
fn cast_as_imp<Src: PrimaryInt, Tgt: PrimaryInt>(src: Src) -> Tgt {
    if const { Src::BITS >= Tgt::BITS } {
        unsafe { *(&raw const src as *const Tgt) }
    } else {
        let mut tgt = Tgt::ZERO;
        unsafe { *(&raw mut tgt as *mut Src) = src };
        if Src::SIGNED {
            let unused_bits = const { Tgt::BITS.wrapping_sub(Src::BITS) };
            Tgt::cast_from_signed(tgt.cast_as_signed() << unused_bits >> unused_bits)
        } else {
            tgt
        }
    }
}

#[inline(always)]
/// Safely converts a basic integer type to another basic integer type.
pub fn checked_cast_as<Src: BasicInt, Tgt: BasicInt>(src: Src) -> Option<Tgt> {
    if Tgt::BITS == 0 {
        if src == Src::ZERO { Some(Tgt::ZERO) } else { None }
    } else if const { Src::SIGNED == Tgt::SIGNED } {
        let tgt: Tgt = cast_as(src);
        if const { Src::BITS <= Tgt::BITS } || src == cast_as(tgt) { Some(tgt) } else { None }
    } else if const { Src::BITS < Tgt::BITS } {
        if Tgt::SIGNED || src >= Src::ZERO { Some(cast_as(src)) } else { None }
    } else {
        // Src::SIGNED != Tgt::SIGNED && Src::BITS >= Tgt::BITS, only positive num is valid
        let zbits = const {
            let zbits = Src::CalcFitted::BITS.wrapping_sub(Tgt::BITS.wrapping_sub(Tgt::SIGNED as u32));
            if Src::SIGNED && zbits == 0 { 1 } else { zbits }
        };
        if src.cast_as_calc().leading_zeros() >= zbits { Some(cast_as(src)) } else { None }
    }
}
