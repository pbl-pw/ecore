use core::{
    fmt::{Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex},
    hash::Hash,
    hint::assert_unchecked,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl,
        ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

use bytemuck::Zeroable;
use const_default::ConstDefault;

use crate::repr::{PlainUncheckable, Uncheckable, Unchecked};

use crate::int::{BasicInt, BasicSInt, BasicUInt, CInt, PrimaryInt, PrimarySInt, PrimaryUInt};

pub use self::alias::*;

/// The actual implementation of u1, u2, u3, ..., u127 (excluding u8, u16, u32, u64) and
/// i1, i2, i3, ..., i127 (excluding i8, i16, i32, i64).
///
/// * `uN` valid value range: `0..=2<sup>N</sup>-1`. Regardless of the operation performed,
///   the internal storage of the output `uN` value is guaranteed to be within this range;
///   otherwise it is a bug.
/// * `iN` valid value range: `-2<sup>N-1</sup>..=2<sup>N-1</sup>-1`. Regardless of the operation,
///   the internal storage of the output `iN` value is guaranteed to be within this range,
///   and for negative values all unused high bits are guaranteed to be 1; otherwise it is a bug.
///   * Special note: the value range of `i1` does not include 1; referencing `i1::ONE`
///     will fail to compile.
/// * The template parameter must satisfy `BITS < T::BITS`, otherwise compilation fails.
///   Standard `uN` and `iN` only support `BITS < T::BITS && BITS > T::BITS / 16 * 8`;
///   for example, standard [`BitInt<u16, BITS>`] only supports `9..=15` for BITS.
///   The following extensions are also supported:
///   * [`BitInt<T, 0>`] has a fixed value of 0 and all computation results are also 0.
///     It is primarily used for generic parameter or associated type placeholders.
///     Note: `!BitInt<T, 0>::ZERO` is still `BitInt<T, 0>::ZERO`; referencing
///     `BitInt<T, 0>::ONE` will fail to compile.
///   * [`BitInt<T, BITS>`] supports `BITS` in `0..=T::BITS-1`, useful when standard
///     `uN` and `iN` need extended alignment or a fixed internal type.
///   * For positive values of [`BitInt<T, BITS>`], the valid range is
///     `0..=2<sup>BITS</sup>-1`. For negative values, except when `BITS == 0`
///     (valid range fixed to 0), the valid range is
///     `-2<sup>BITS-1</sup>..=2<sup>BITS-1</sup>-1`.
/// * Based on the above rules, internal computations are optimized as follows:
///   * Minimum internal `T` range: for `uN` it is `0..=2<sup>N+1</sup>-1`, for `iN`
///     it is `-2<sup>N</sup>..=2<sup>N</sup>-1`. Larger integer types may be used
///     during actual computation.
///   * Internal `T + T` range: for `uN`, `0..=2<sup>N</sup>-1 + 2<sup>N</sup>-1 = 2<sup>N+1</sup>-2`;
///     for `iN`, `-2<sup>N-1</sup> + -2<sup>N-1</sup> = -2<sup>N</sup>` to
///     `2<sup>N-1</sup>-1 + 2<sup>N-1</sup>-1 = 2<sup>N</sup>-2`. Both fit within
///     the minimum range and will not overflow.
///   * Internal `T - T`: for `uN` uses `wrapping_sub` — when the result underflows,
///     all high bits become 1, otherwise no overflow. For `iN`,
///     `-2<sup>N-1</sup> - (2<sup>N-1</sup> - 1) = -2<sup>N</sup> + 1` to
///     `2<sup>N-1</sup>-1 - -2<sup>N-1</sup> = 2<sup>N</sup> - 1`, fitting within
///     the minimum range without overflow.
///   * Internal `T / T`: no overflow except division by zero, but for `iN`,
///     `-2<sup>N-1</sup> / -1 = 2<sup>N-1</sup>` exceeds the `iN` range.
#[derive(Copy, Clone, Default, ConstDefault, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BitInt<T, const BITS: u32>(T);

impl<T: PrimaryInt, const BITS: u32> BitInt<T, BITS> {
    pub const BITS: u32 = BITS;
    pub const MAX: Self = Self(CInt::unbounded_shr(T::MAX, Self::UNUSED_BITS));
    pub const MIN: Self = Self(if BITS == 0 { T::ZERO } else { CInt::unbounded_shr(T::MIN, Self::UNUSED_BITS) });
    pub const IS_NORMAL: bool = BITS < T::BITS && BITS > T::BITS / 16 * 8;
    const UNUSED_BITS: u32 = T::BITS - BITS;
    const UNUSED_CALC_BITS: u32 = T::CalcFitted::BITS - BITS;
    const ASSERT: () = assert!(BITS < T::BITS);

    /// The value must be guaranteed to be within the valid range, otherwise UB.
    ///
    /// # Safety
    ///
    /// The caller must ensure `value` fits within `BITS` bits.
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        let () = Self::ASSERT;
        debug_assert!(CInt::eq(if BITS == 0 { T::ZERO } else { CInt::shr(CInt::shl(value, Self::UNUSED_BITS), Self::UNUSED_BITS) }, value));
        Self(value)
    }

    #[inline(always)]
    pub const fn value(self) -> T {
        unsafe { assert_unchecked(CInt::le(Self::MIN.0, self.0) && CInt::le(self.0, Self::MAX.0)) };
        self.0
    }

    #[inline(always)]
    pub const fn new(value: T) -> Option<Self> {
        let () = Self::ASSERT;
        let v = Self::cast_new(value);
        if CInt::eq(v.0, value) { Some(v) } else { None }
    }

    #[inline(always)]
    pub const fn try_new(value: T) -> Result<Self, T> {
        let () = Self::ASSERT;
        let v = Self::cast_new(value);
        if CInt::eq(v.0, value) { Ok(v) } else { Err(value) }
    }

    #[inline(always)]
    pub const fn cast_new(value: T) -> Self {
        let () = Self::ASSERT;
        Self(if BITS == 0 { T::ZERO } else { CInt::shr(CInt::shl(value, Self::UNUSED_BITS), Self::UNUSED_BITS) })
    }

    #[inline(always)]
    fn calc_new(value: T::CalcFitted) -> Option<Self> {
        if Self::mask(value) == value { Some(Self(T::cast_from_calc(value))) } else { None }
    }

    #[inline(always)]
    fn mask_new(value: T::CalcFitted) -> Self {
        Self(T::cast_from_calc(Self::mask(value)))
    }

    /// Creates a value by direct truncation when the computed value already satisfies
    /// the storage range.
    #[inline(always)]
    unsafe fn unmask_new(value: T::CalcFitted) -> Self {
        debug_assert!(Self::mask(value) == value, "bitint create using invalid value pattern");
        Self(T::cast_from_calc(value))
    }

    /// Equivalent to `mask_new`; should normally be equivalent to `unmask_new`,
    /// but some values are invalid, and checking them is more expensive than
    /// the mask operation.
    #[track_caller]
    fn debug_mask_new(value: T::CalcFitted, err: &'static str) -> Self {
        let nv = Self::mask(value);
        debug_assert!(nv == value, "{}", err);
        Self(T::cast_from_calc(nv))
    }

    #[inline(always)]
    fn overflowing_new(value: T::CalcFitted) -> (Self, bool) {
        let nv = Self::mask(value);
        (Self(T::cast_from_calc(nv)), nv != value)
    }

    #[inline(always)]
    fn overflowing_new2(value: T::CalcFitted, overflow: bool) -> (Self, bool) {
        let nv = Self::mask(value);
        (Self(T::cast_from_calc(nv)), if nv == value { overflow } else { true })
    }

    fn saturating_new(value: T::CalcFitted) -> Self {
        if Self::mask(value) == value {
            Self(T::cast_from_calc(value))
        } else if T::SIGNED && value < T::CalcFitted::ZERO {
            Self::MIN
        } else {
            Self::MAX
        }
    }

    #[inline(always)]
    fn is_negative(self) -> bool {
        T::SIGNED && self.0 < T::ZERO
    }

    #[inline(always)]
    fn mask(value: T::CalcFitted) -> T::CalcFitted {
        if BITS == 0 { T::CalcFitted::ZERO } else { value << Self::UNUSED_CALC_BITS >> Self::UNUSED_CALC_BITS }
    }

    #[inline(always)]
    fn wrapping_shl(self, shift: u32) -> Self {
        if BITS == 0 {
            Self::ZERO
        } else {
            Self(T::cast_from_calc(self.cast_as_calc().wrapping_shl(Self::UNUSED_CALC_BITS.wrapping_add(shift)) >> Self::UNUSED_CALC_BITS))
        }
    }
}

impl<T: PrimaryInt, const BITS: u32> Add for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::debug_mask_new(self.cast_as_calc() + rhs.cast_as_calc(), "bitint add with overflow")
    }
}

impl<T: PrimaryInt, const BITS: u32> AddAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> BitAnd for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<T: PrimaryInt, const BITS: u32> BitAndAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> BitOr for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl<T: PrimaryInt, const BITS: u32> BitOrAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> BitXor for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl<T: PrimaryInt, const BITS: u32> BitXorAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> Div for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    #[track_caller]
    fn div(self, rhs: Self) -> Self::Output {
        if T::SIGNED { Self::debug_mask_new(self.cast_as_calc() / rhs.cast_as_calc(), "bitint div with overflow") } else { Self(self.0 / rhs.0) }
    }
}

impl<T: PrimaryInt, const BITS: u32> DivAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> Mul for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::debug_mask_new(self.cast_as_calc() * rhs.cast_as_calc(), "bitint mul with overflow")
    }
}

impl<T: PrimaryInt, const BITS: u32> MulAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> Not for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        if T::SIGNED && BITS != 0 { Self(!self.0) } else { Self::mask_new(!self.cast_as_calc()) }
    }
}

impl<T: PrimaryInt, const BITS: u32> Rem for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl<T: PrimaryInt, const BITS: u32> RemAssign for BitInt<T, BITS> {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

impl<T: PrimaryInt, const BITS: u32, Rhs: PrimaryInt> Shl<Rhs> for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    #[track_caller]
    fn shl(self, rhs: Rhs) -> Self::Output {
        debug_assert!(
            Rhs::ZERO <= rhs && {
                let bits = BITS.checked_cast_as();
                bits.is_some_and(|bits| rhs < bits) || bits.is_none()
            },
            "bitint shl with overflow"
        );
        self.wrapping_shl(rhs.cast_as())
    }
}

impl<T: PrimaryInt, const BITS: u32, Rhs: PrimaryInt> ShlAssign<Rhs> for BitInt<T, BITS> {
    #[inline(always)]
    #[track_caller]
    fn shl_assign(&mut self, rhs: Rhs) {
        *self = *self << rhs
    }
}

impl<T: PrimaryInt, const BITS: u32, Rhs: PrimaryInt> Shr<Rhs> for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    #[track_caller]
    fn shr(self, rhs: Rhs) -> Self::Output {
        debug_assert!(
            Rhs::ZERO <= rhs && {
                let bits = BITS.checked_cast_as();
                bits.is_some_and(|bits| rhs < bits) || bits.is_none()
            },
            "bitint shr with overflow"
        );
        self.wrapping_shr(rhs.cast_as())
    }
}

impl<T: PrimaryInt, const BITS: u32, Rhs: PrimaryInt> ShrAssign<Rhs> for BitInt<T, BITS> {
    #[inline(always)]
    #[track_caller]
    fn shr_assign(&mut self, rhs: Rhs) {
        *self = *self >> rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> Sub for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::debug_mask_new(self.cast_as_calc().wrapping_sub(rhs.cast_as_calc()), "bitint sub with overflow")
    }
}

impl<T: PrimaryInt, const BITS: u32> SubAssign for BitInt<T, BITS> {
    #[inline(always)]
    #[track_caller]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T: PrimaryInt, const BITS: u32> Binary for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Binary>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> Debug for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Debug>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> Display for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Display>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> Hash for BitInt<T, BITS> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: PrimaryInt, const BITS: u32> Octal for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as Octal>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> UpperHex for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as UpperHex>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> LowerHex for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as LowerHex>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> UpperExp for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as UpperExp>::fmt(&self.0, f)
    }
}

impl<T: PrimaryInt, const BITS: u32> LowerExp for BitInt<T, BITS> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as LowerExp>::fmt(&self.0, f)
    }
}

impl<T: PrimarySInt, const BITS: u32> Neg for BitInt<T, BITS> {
    type Output = Self;

    #[inline(always)]
    #[track_caller]
    fn neg(self) -> Self::Output {
        Self::debug_mask_new(-self.cast_as_calc(), "bitint neg with overflow")
    }
}

impl<T: PrimaryInt, const BITS: u32> BasicInt for BitInt<T, BITS> {
    const BITS: u32 = BITS;
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const ZERO: Self = Self(T::ZERO);
    const ONE: Self = {
        assert!(!T::SIGNED || BITS >= 2, "i1 not has value 1");
        Self(T::ONE)
    };
    const ALL_ONE: Self = Self(CInt::unbounded_shr(T::ALL_ONE, Self::UNUSED_BITS));
    const SIGNED: bool = T::SIGNED;
    type Primary = T;
    type Signed = BitInt<T::Signed, BITS>;
    type Unsigned = BitInt<T::Unsigned, BITS>;
    type CalcFitted = T::CalcFitted;

    #[inline(always)]
    fn abs_diff(self, other: Self) -> Self::Unsigned {
        BitInt(self.0.abs_diff(other.0))
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_add(other.cast_as_calc()))
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let other = other.cast_as_calc();
        if other == T::CalcFitted::ZERO {
            None
        } else {
            let v = self.cast_as_calc().wrapping_div(other);
            if Self::SIGNED { Self::calc_new(v) } else { Some(unsafe { Self::unmask_new(v) }) }
        }
    }

    #[inline(always)]
    fn checked_div_euclid(self, other: Self) -> Option<Self> {
        let other = other.cast_as_calc();
        if other == T::CalcFitted::ZERO {
            None
        } else {
            let v = self.cast_as_calc().wrapping_div_euclid(other);
            if Self::SIGNED { Self::calc_new(v) } else { Some(unsafe { Self::unmask_new(v) }) }
        }
    }

    #[inline(always)]
    fn checked_ilog(self, base: Self) -> Option<u32> {
        self.0.checked_ilog(base.0)
    }

    #[inline(always)]
    fn checked_ilog2(self) -> Option<u32> {
        self.0.checked_ilog2()
    }

    #[inline(always)]
    fn checked_ilog10(self) -> Option<u32> {
        self.0.checked_ilog10()
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        if let Some(v) = self.cast_as_calc().checked_mul(other.cast_as_calc()) { Self::calc_new(v) } else { None }
    }

    #[inline(always)]
    fn checked_neg(self) -> Option<Self> {
        let v = self.cast_as_calc();
        if Self::SIGNED { Self::calc_new(v.wrapping_neg()) } else { if v == T::CalcFitted::ZERO { Some(Self::ZERO) } else { None } }
    }

    #[inline(always)]
    fn checked_pow(self, exp: u32) -> Option<Self> {
        if let Some(v) = self.cast_as_calc().checked_pow(exp) { Self::calc_new(v) } else { None }
    }

    #[inline(always)]
    fn checked_rem(self, other: Self) -> Option<Self> {
        let other = other.cast_as_calc();
        if other == T::CalcFitted::ZERO { None } else { Some(unsafe { Self::unmask_new(self.cast_as_calc().wrapping_rem(other)) }) }
    }

    #[inline(always)]
    fn checked_rem_euclid(self, other: Self) -> Option<Self> {
        let other = other.cast_as_calc();
        if other == T::CalcFitted::ZERO { None } else { Some(unsafe { Self::unmask_new(self.cast_as_calc().wrapping_rem_euclid(other)) }) }
    }

    #[inline(always)]
    fn checked_shl(self, shift: u32) -> Option<Self> {
        if shift < BITS { Some(self.wrapping_shl(shift)) } else { None }
    }

    #[inline(always)]
    fn checked_shr(self, shift: u32) -> Option<Self> {
        if shift < BITS { Some(Self(self.0 >> shift)) } else { None }
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_sub(other.cast_as_calc()))
    }

    #[inline(always)]
    fn count_ones(self) -> u32 {
        let num = self.cast_as_calc().count_ones();
        if self.is_negative() { num - Self::UNUSED_CALC_BITS } else { num }
    }

    #[inline(always)]
    fn count_zeros(self) -> u32 {
        let num = self.cast_as_calc().count_zeros();
        if self.is_negative() { num } else { num - Self::UNUSED_CALC_BITS }
    }

    #[inline(always)]
    fn div_euclid(self, other: Self) -> Self {
        Self(self.0.div_euclid(other.0))
    }

    #[inline(always)]
    #[track_caller]
    fn ilog(self, base: Self) -> u32 {
        self.0.ilog(base.0)
    }

    #[inline(always)]
    fn ilog2(self) -> u32 {
        self.0.ilog2()
    }

    #[inline(always)]
    fn ilog10(self) -> u32 {
        self.0.ilog10()
    }

    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        let lz = self.cast_as_calc().leading_zeros();
        if Self::SIGNED && lz == 0 { lz } else { lz - Self::UNUSED_CALC_BITS }
    }

    #[inline(always)]
    fn leading_ones(self) -> u32 {
        if BITS == 0 { 0 } else { (self.cast_as_calc() << Self::UNUSED_CALC_BITS).leading_ones() }
    }

    #[inline(always)]
    fn midpoint(self, other: Self) -> Self {
        Self(self.0.midpoint(other.0))
    }

    #[inline(always)]
    fn overflowing_add(self, other: Self) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc() + other.cast_as_calc())
    }

    #[inline(always)]
    fn overflowing_div(self, other: Self) -> (Self, bool) {
        if other.0 == T::ZERO { (Self::ZERO, true) } else { Self::overflowing_new(self.cast_as_calc() / other.cast_as_calc()) }
    }

    #[inline(always)]
    fn overflowing_div_euclid(self, other: Self) -> (Self, bool) {
        if other.0 == T::ZERO { (Self::ZERO, true) } else { Self::overflowing_new(self.cast_as_calc().div_euclid(other.cast_as_calc())) }
    }

    #[inline(always)]
    fn overflowing_mul(self, other: Self) -> (Self, bool) {
        let (v, overflow) = self.cast_as_calc().overflowing_mul(other.cast_as_calc());
        Self::overflowing_new2(v, overflow)
    }

    #[inline(always)]
    fn overflowing_neg(self) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_neg())
    }

    #[inline(always)]
    fn overflowing_pow(self, exp: u32) -> (Self, bool) {
        let (v, overflow) = self.cast_as_calc().overflowing_pow(exp);
        Self::overflowing_new2(v, overflow)
    }

    #[inline(always)]
    fn overflowing_rem(self, other: Self) -> (Self, bool) {
        if other.0 == T::ZERO { (Self::ZERO, true) } else { Self::overflowing_new(self.cast_as_calc() % other.cast_as_calc()) }
    }

    #[inline(always)]
    fn overflowing_rem_euclid(self, other: Self) -> (Self, bool) {
        if other.0 == T::ZERO { (Self::ZERO, true) } else { Self::overflowing_new(self.cast_as_calc().rem_euclid(other.cast_as_calc())) }
    }

    #[inline(always)]
    fn overflowing_shl(self, shift: u32) -> (Self, bool) {
        if shift < BITS { (self.wrapping_shl(shift), false) } else { (Self::ZERO, true) }
    }

    #[inline(always)]
    fn overflowing_shr(self, shift: u32) -> (Self, bool) {
        if shift < BITS { (Self(self.0 >> shift), false) } else { (Self(self.0 >> BITS), true) }
    }

    #[inline(always)]
    fn overflowing_sub(self, other: Self) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_sub(other.cast_as_calc()))
    }

    #[inline(always)]
    #[track_caller]
    fn pow(self, exp: u32) -> Self {
        Self::debug_mask_new(self.cast_as_calc().pow(exp), "bitint pow with overflow")
    }

    #[inline(always)]
    fn rem_euclid(self, other: Self) -> Self {
        Self(self.0.rem_euclid(other.0))
    }

    #[inline(always)]
    fn reverse_bits(self) -> Self {
        if BITS == 0 { Self::ZERO } else { unsafe { Self::unmask_new(self.cast_as_calc().reverse_bits() >> Self::UNUSED_CALC_BITS) } }
    }

    #[inline(always)]
    #[track_caller]
    fn rotate_left(self, shift: u32) -> Self {
        let true = BITS != 0 else { return Self::ZERO };
        let bits = shift % BITS;
        if Self::SIGNED && bits == 0 {
            return self;
        }
        let calc = self.cast_as_calc();
        let hi = calc << (Self::UNUSED_CALC_BITS + bits) >> Self::UNUSED_CALC_BITS;
        if Self::SIGNED {
            let lo = T::CalcFitted::cast_from_unsigned(calc.cast_as_unsigned() << Self::UNUSED_CALC_BITS >> (T::CalcFitted::BITS - bits));
            unsafe { Self::unmask_new(hi | lo) }
        } else {
            unsafe { Self::unmask_new(hi | calc >> (BITS - bits)) }
        }
    }

    #[inline(always)]
    fn rotate_right(self, shift: u32) -> Self {
        let true = BITS != 0 else { return Self::ZERO };
        let bits = shift % BITS;
        if bits == 0 {
            self
        } else {
            let calc = self.cast_as_calc();
            let hi = calc << (T::CalcFitted::BITS - bits) >> Self::UNUSED_CALC_BITS;
            let lo = if Self::SIGNED {
                T::CalcFitted::cast_from_unsigned(calc.cast_as_unsigned() << Self::UNUSED_CALC_BITS >> (Self::UNUSED_CALC_BITS + bits))
            } else {
                calc >> bits
            };
            unsafe { Self::unmask_new(hi | lo) }
        }
    }

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        Self::saturating_new(self.cast_as_calc() + other.cast_as_calc())
    }

    #[inline(always)]
    fn saturating_div(self, other: Self) -> Self {
        if other.0 == T::ZERO { Self::MAX } else { Self::saturating_new(self.cast_as_calc() / other.cast_as_calc()) }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        Self::saturating_new(self.cast_as_calc().saturating_mul(other.cast_as_calc()))
    }

    fn saturating_pow(self, exp: u32) -> Self {
        Self::saturating_new(self.cast_as_calc().saturating_pow(exp))
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        if Self::SIGNED {
            Self::saturating_new(self.cast_as_calc().wrapping_sub(other.cast_as_calc()))
        } else {
            unsafe { Self::unmask_new(self.cast_as_calc().saturating_sub(other.cast_as_calc())) }
        }
    }

    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        let num = self.0.trailing_ones();
        if Self::SIGNED && num > BITS { BITS } else { num }
    }

    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        let num = self.0.trailing_zeros();
        if num > BITS { BITS } else { num }
    }

    #[inline(always)]
    fn unbounded_shl(self, shift: u32) -> Self {
        if shift < BITS { self.wrapping_shl(shift) } else { Self::ZERO }
    }

    #[inline(always)]
    fn unbounded_shr(self, shift: u32) -> Self {
        if shift < BITS {
            Self(self.0 >> shift)
        } else if Self::SIGNED
            && let Some(shift) = const { BITS.checked_sub(1) }
        {
            Self(self.0 >> shift)
        } else {
            Self::ZERO
        }
    }

    #[inline(always)]
    unsafe fn unchecked_add(self, other: Self) -> Self {
        unsafe { Self::unmask_new(self.cast_as_calc().wrapping_add(other.cast_as_calc())) }
    }

    #[inline(always)]
    unsafe fn unchecked_mul(self, other: Self) -> Self {
        unsafe { Self::unmask_new(self.cast_as_calc().unchecked_mul(other.cast_as_calc())) }
    }

    #[inline(always)]
    unsafe fn unchecked_sub(self, other: Self) -> Self {
        unsafe { Self::unmask_new(self.cast_as_calc().wrapping_sub(other.cast_as_calc())) }
    }

    #[inline(always)]
    fn wrapping_add(self, other: Self) -> Self {
        Self::mask_new(self.cast_as_calc() + other.cast_as_calc())
    }

    #[inline(always)]
    #[track_caller]
    fn wrapping_div(self, other: Self) -> Self {
        let v = self.cast_as_calc().wrapping_div(other.cast_as_calc());
        if Self::SIGNED { Self::mask_new(v) } else { unsafe { Self::unmask_new(v) } }
    }

    #[inline(always)]
    fn wrapping_div_euclid(self, other: Self) -> Self {
        let v = self.cast_as_calc().wrapping_div_euclid(other.cast_as_calc());
        if Self::SIGNED { Self::mask_new(v) } else { unsafe { Self::unmask_new(v) } }
    }

    #[inline(always)]
    fn wrapping_mul(self, other: Self) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_mul(other.cast_as_calc()))
    }

    #[inline(always)]
    fn wrapping_neg(self) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_neg())
    }

    #[inline(always)]
    fn wrapping_pow(self, exp: u32) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_pow(exp))
    }

    #[inline(always)]
    fn wrapping_rem(self, other: Self) -> Self {
        unsafe { Self::unmask_new(self.cast_as_calc().wrapping_rem(other.cast_as_calc())) }
    }

    #[inline(always)]
    fn wrapping_rem_euclid(self, other: Self) -> Self {
        unsafe { Self::unmask_new(self.cast_as_calc().wrapping_rem_euclid(other.cast_as_calc())) }
    }

    #[inline(always)]
    fn wrapping_shl(self, shift: u32) -> Self {
        Self::wrapping_shl(self, shift)
    }

    #[inline(always)]
    fn wrapping_shr(self, shift: u32) -> Self {
        Self(self.0.wrapping_shr(shift))
    }

    #[inline(always)]
    fn wrapping_sub(self, other: Self) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_sub(other.cast_as_calc()))
    }

    #[inline(always)]
    fn cast_as_primary(self) -> Self::Primary {
        unsafe { assert_unchecked(Self::MIN.0 <= self.0 && self.0 <= Self::MAX.0) };
        self.0
    }

    #[inline(always)]
    fn cast_from_primary(src: Self::Primary) -> Self {
        Self::cast_new(src)
    }

    #[inline(always)]
    unsafe fn unchecked_from_primary(src: Self::Primary) -> Self {
        unsafe { Self::new_unchecked(src) }
    }

    #[inline(always)]
    fn cast_as_signed(self) -> Self::Signed {
        if Self::SIGNED { BitInt(self.0.cast_as_signed()) } else { BitInt::cast_new(self.0.cast_as_signed()) }
    }

    #[inline(always)]
    fn cast_from_signed(src: Self::Signed) -> Self {
        if Self::SIGNED { Self(T::cast_from_signed(src.0)) } else { Self::cast_new(T::cast_from_signed(src.0)) }
    }

    #[inline(always)]
    fn cast_as_unsigned(self) -> Self::Unsigned {
        if Self::SIGNED { BitInt::cast_new(self.0.cast_as_unsigned()) } else { BitInt(self.0.cast_as_unsigned()) }
    }

    #[inline(always)]
    fn cast_from_unsigned(src: Self::Unsigned) -> Self {
        if Self::SIGNED { Self::cast_new(T::cast_from_unsigned(src.0)) } else { Self(T::cast_from_unsigned(src.0)) }
    }

    #[inline(always)]
    fn cast_as_calc(self) -> Self::CalcFitted {
        unsafe { assert_unchecked(Self::MIN.0 <= self.0 && self.0 <= Self::MAX.0) };
        self.0.cast_as_calc()
    }

    #[inline(always)]
    fn cast_from_calc(src: Self::CalcFitted) -> Self {
        let () = Self::ASSERT;
        Self::mask_new(src)
    }
}

impl<T: PrimarySInt, const BITS: u32> BasicSInt for BitInt<T, BITS> {
    type SignedCalcFitted = T::SignedCalcFitted;

    #[inline(always)]
    fn abs(self) -> Self {
        Self::debug_mask_new(self.cast_as_calc().abs(), "bitint abs with overflow")
    }

    #[inline(always)]
    fn cast_unsigned(self) -> Self::Unsigned {
        self.cast_as_unsigned()
    }

    #[inline(always)]
    fn checked_abs(self) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_abs())
    }

    #[inline(always)]
    fn checked_add_unsigned(self, other: Self::Unsigned) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_add_unsigned(other.cast_as()))
    }

    #[inline(always)]
    fn checked_sub_unsigned(self, other: Self::Unsigned) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_sub_unsigned(other.cast_as()))
    }

    #[inline(always)]
    fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    #[inline(always)]
    fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    #[inline(always)]
    fn overflowing_abs(self) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_abs())
    }

    #[inline(always)]
    fn overflowing_add_unsigned(self, other: Self::Unsigned) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_add_unsigned(other.cast_as()))
    }

    #[inline(always)]
    fn overflowing_sub_unsigned(self, other: Self::Unsigned) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_sub_unsigned(other.cast_as()))
    }

    #[inline(always)]
    fn saturating_abs(self) -> Self {
        if let Some(v) = self.checked_abs() { v } else { Self::MAX }
    }

    #[inline(always)]
    fn saturating_add_unsigned(self, other: Self::Unsigned) -> Self {
        if let Some(v) = self.checked_add_unsigned(other) { v } else { Self::MAX }
    }

    #[inline(always)]
    fn saturating_neg(self) -> Self {
        if let Some(v) = self.checked_neg() { v } else { Self::MAX }
    }

    #[inline(always)]
    fn saturating_sub_unsigned(self, other: Self::Unsigned) -> Self {
        if let Some(v) = self.checked_sub_unsigned(other) { v } else { Self::MIN }
    }

    #[inline(always)]
    fn signum(self) -> Self::Primary {
        self.0.signum()
    }

    #[inline(always)]
    fn unsigned_abs(self) -> Self::Unsigned {
        BitInt(self.0.unsigned_abs())
    }

    #[inline(always)]
    fn wrapping_abs(self) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_abs())
    }

    #[inline(always)]
    fn wrapping_add_unsigned(self, other: Self::Unsigned) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_add_unsigned(other.cast_as()))
    }

    #[inline(always)]
    fn wrapping_sub_unsigned(self, other: Self::Unsigned) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_sub_unsigned(other.cast_as()))
    }
}

impl<T: PrimaryUInt, const BITS: u32> BasicUInt for BitInt<T, BITS> {
    type UnsignedCalcFitted = T::UnsignedCalcFitted;

    #[inline(always)]
    fn cast_signed(self) -> Self::Signed {
        self.cast_as_signed()
    }

    #[inline(always)]
    fn checked_add_signed(self, other: Self::Signed) -> Option<Self> {
        Self::calc_new(self.cast_as_calc().wrapping_add_signed(other.cast_as()))
    }

    #[inline(always)]
    fn checked_next_multiple_of(self, other: Self) -> Option<Self> {
        if let Some(v) = self.cast_as_calc().checked_next_multiple_of(other.cast_as_calc()) { Self::calc_new(v) } else { None }
    }

    #[inline(always)]
    fn checked_next_power_of_two(self) -> Option<Self> {
        if let Some(v) = self.cast_as_calc().checked_next_power_of_two() { Self::calc_new(v) } else { None }
    }

    #[inline(always)]
    fn div_ceil(self, other: Self) -> Self {
        Self(self.0.div_ceil(other.0))
    }

    #[inline(always)]
    fn is_power_of_two(self) -> bool {
        self.0.is_power_of_two()
    }

    #[inline(always)]
    fn next_multiple_of(self, other: Self) -> Self {
        Self::debug_mask_new(self.cast_as_calc().next_multiple_of(other.cast_as_calc()), "bitint next multiple with overflow")
    }

    #[inline(always)]
    fn next_power_of_two(self) -> Self {
        Self::debug_mask_new(self.cast_as_calc().next_power_of_two(), "bitint next power of two with overflow")
    }

    #[inline(always)]
    fn overflowing_add_signed(self, other: Self::Signed) -> (Self, bool) {
        Self::overflowing_new(self.cast_as_calc().wrapping_add_signed(other.cast_as()))
    }

    #[inline(always)]
    fn saturating_add_signed(self, other: Self::Signed) -> Self {
        let v = self.cast_as_calc().wrapping_add_signed(other.cast_as());
        if Self::mask(v) == v {
            Self(T::cast_from_calc(v))
        } else if other.is_negative() {
            Self::MIN
        } else {
            Self::MAX
        }
    }

    #[inline(always)]
    fn wrapping_add_signed(self, other: Self::Signed) -> Self {
        Self::mask_new(self.cast_as_calc().wrapping_add_signed(other.cast_as()))
    }
}

unsafe impl<T: PrimaryInt, const BITS: u32> Zeroable for BitInt<T, BITS> {}

impl<T: PrimaryInt, const BITS: u32> Uncheckable for BitInt<T, BITS> {
    type UncheckedRaw = T;

    fn raw_value(sf: Self) -> Self::UncheckedRaw {
        sf.value()
    }

    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
        Self::try_new(raw)
    }
}
unsafe impl<T: PrimaryInt, const BITS: u32> PlainUncheckable for BitInt<T, BITS> {}

impl<T: PrimaryInt, const BITS: u32> Unchecked<BitInt<T, BITS>> {
    pub fn masked_value(self) -> BitInt<T, BITS> {
        BitInt::cast_new(self.raw_value())
    }
}

impl From<u1> for bool {
    #[inline(always)]
    #[allow(clippy::transmute_int_to_bool)]
    fn from(value: u1) -> Self {
        unsafe { core::mem::transmute(value.0) }
    }
}

impl From<bool> for u1 {
    #[inline(always)]
    fn from(value: bool) -> Self {
        Self(value as u8)
    }
}

mod alias;
mod compat;
mod non_zero;
