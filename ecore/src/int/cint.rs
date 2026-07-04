use core::hint::unreachable_unchecked;

use super::{BasicInt, PrimaryInt, PrimarySInt, PrimaryUInt};

/// Collection of const-function operations for integer types.
/// * Operations with a `PrimaryInt` generic parameter only support native integer types,
///   while those requiring `BasicInt` support basic integer types (extending support to
///   non-native types like u1, u2, u3, ...).
/// * For operations that only support native integer types (`op`), if `ex_op` exists,
///   it represents the same operation extended to support basic integer types.
pub struct CInt();

macro_rules! dispatch {
    ($imp:ident $(, $op:tt)*) => {
        if Src::SIGNED {
            match Src::Primary::BITS {
                8 => $imp!(i8 $(, $op)*),
                16 => $imp!(i16 $(, $op)*),
                32 => $imp!(i32 $(, $op)*),
                64 => $imp!(i64 $(, $op)*),
                128 => $imp!(i128 $(, $op)*),
                _ => unsafe { unreachable_unchecked() },
            }
        } else {
            match Src::Primary::BITS {
                8 => $imp!(u8 $(, $op)*),
                16 => $imp!(u16 $(, $op)*),
                32 => $imp!(u32 $(, $op)*),
                64 => $imp!(u64 $(, $op)*),
                128 => $imp!(u128 $(, $op)*),
                _ => unsafe { unreachable_unchecked() },
            }
        }
    };
}
macro_rules! dispatch_signed {
    ($imp:ident $(, $op:tt)*) => {
        match Src::Primary::BITS {
            8 => $imp!(i8 $(, $op)*),
            16 => $imp!(i16 $(, $op)*),
            32 => $imp!(i32 $(, $op)*),
            64 => $imp!(i64 $(, $op)*),
            128 => $imp!(i128 $(, $op)*),
            _ => unsafe { unreachable_unchecked() },
        }
    };
}
macro_rules! dispatch_unsigned {
    ($imp:ident $(, $op:tt)*) => {
        match Src::Primary::BITS {
            8 => $imp!(u8 $(, $op)*),
            16 => $imp!(u16 $(, $op)*),
            32 => $imp!(u32 $(, $op)*),
            64 => $imp!(u64 $(, $op)*),
            128 => $imp!(u128 $(, $op)*),
            _ => unsafe { unreachable_unchecked() },
        }
    };
}

macro_rules! imp_unary {
    ($type:ty, $op:tt, $src:ident) => {
        unsafe { transmute(transmute::<_, $type>($src).$op()) }
    };
}

macro_rules! imp_unary_fixed_return {
    ($type:ty, $op:tt, $src:ident) => {
        unsafe { transmute::<_, $type>($src).$op() }
    };
}

/// Unary operation returning `$result<Src>`.
macro_rules! unary {
    ($op:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryInt>(src: Src) -> $result!(Src) {
            dispatch!($imp, $op, src)
        }
    };
    ($op:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryInt>(src: Src) -> $result!(Src) {
            dispatch!(imp_unary, $op, src)
        }
    };
}

/// Signed unary operation returning `$result<Src>`.
macro_rules! unary_signed {
    ($op:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src) -> $result!(Src) {
            dispatch_signed!($imp, $op, src)
        }
    };
    ($op:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src) -> $result!(Src) {
            dispatch_signed!(imp_unary, $op, src)
        }
    };
}

/// Unsigned unary operation returning `$result<Src>`.
macro_rules! unary_unsigned {
    ($op:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryUInt>(src: Src) -> $result!(Src) {
            dispatch_unsigned!($imp, $op, src)
        }
    };
    ($op:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryUInt>(src: Src) -> $result!(Src) {
            dispatch_unsigned!(imp_unary, $op, src)
        }
    };
}

macro_rules! imp_binary {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute(transmute::<_, $type>($src).$op(transmute($right))) }
    };
}

macro_rules! imp_binary_fixed_right {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute(transmute::<_, $type>($src).$op($right)) }
    };
}

macro_rules! imp_binary_fixed_return {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute::<_, $type>($src).$op(transmute($right)) }
    };
}

/// Binary operation returning `$result<Src>`.
macro_rules! binary {
    ($op:ident, $right:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch!($imp, $op, src, right)
        }
    };
    ($op:ident, $right:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimaryInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch!(imp_binary, $op, src, right)
        }
    };
}

/// Signed binary operation returning `$result<Src>`.
macro_rules! binary_signed {
    ($op:ident, $right:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch_signed!($imp, $op, src, right)
        }
    };
    ($op:ident, $right:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch_signed!(imp_binary, $op, src, right)
        }
    };
}

/// Unsigned binary operation returning `$result<Src>`.
macro_rules! binary_unsigned {
    ($op:ident, $right:ident, $result:ident, $imp:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch_unsigned!($imp, $op, src, right)
        }
    };
    ($op:ident, $right:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $op<Src: PrimarySInt>(src: Src, right: $right!(Src)) -> $result!(Src) {
            dispatch_unsigned!(imp_binary, $op, src, right)
        }
    };
}

macro_rules! as_src {
    ($type:ty) => {
        $type
    };
}
macro_rules! map_option {
    ($type:ty) => {
        Option<$type>
    };
}
macro_rules! map_overflow {
    ($type:ty) => {
        ($type, bool)
    };
}
macro_rules! as_unsigned {
    ($type:ty) => {
        <$type>::Unsigned
    };
}
macro_rules! as_signed {
    ($type:ty) => {
        <$type>::Signed
    };
}
macro_rules! as_bytes_array {
    ($type:ty) => {
        <$type>::BytesArray
    };
}
macro_rules! as_u32 {
    ($type:ty) => {
        u32
    };
}
macro_rules! as_option_u32 {
    ($type:ty) => {
        Option<u32>
    };
}
macro_rules! as_bool {
    ($type:ty) => {
        bool
    };
}

impl CInt {
    /// Converts between native integer types using `as` casting rules.
    /// Any two of i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
    /// can be mutually converted.
    #[inline(always)]
    pub const fn cast_as<Src: PrimaryInt, Tgt: PrimaryInt>(src: Src) -> Tgt {
        if Src::BITS >= Tgt::BITS {
            unsafe { *(&src as *const Src as *const Tgt) }
        } else {
            let mut out = Tgt::ZERO;
            unsafe { *(&mut out as *mut Tgt as *mut Src) = src };
            if Src::SIGNED {
                let unused_bits = const { Tgt::BITS.wrapping_sub(Src::BITS) };
                match Tgt::BITS {
                    16 => unsafe { transmute(transmute::<_, i16>(out) << unused_bits >> unused_bits) },
                    32 => unsafe { transmute(transmute::<_, i32>(out) << unused_bits >> unused_bits) },
                    64 => unsafe { transmute(transmute::<_, i64>(out) << unused_bits >> unused_bits) },
                    128 => unsafe { transmute(transmute::<_, i128>(out) << unused_bits >> unused_bits) },
                    _ => unsafe { unreachable_unchecked() },
                }
            } else {
                out
            }
        }
    }

    /// Converts between basic integer types using `as` casting rules.
    /// Any two of i8, i16, i32, i64, i128, isize, u1, u2, u3, ..., u128, usize
    /// can be mutually converted.
    #[inline(always)]
    pub const fn ex_cast_as<Src: BasicInt, Tgt: BasicInt>(src: Src) -> Tgt {
        Self::cast_from_primary(Self::cast_as(Self::cast_as_primary(src)))
    }

    #[inline(always)]
    /// Safely converts a basic integer type to another basic integer type.
    pub const fn checked_cast_as<Src: BasicInt, Tgt: BasicInt>(src: Src) -> Option<Tgt> {
        let src = Self::cast_as_primary(src);
        let en = if Src::SIGNED == Tgt::SIGNED && Src::BITS == Tgt::BITS {
            true
        } else {
            let lz = Self::leading_zeros(src);
            if Tgt::BITS == 0 {
                lz == Src::Primary::BITS
            } else {
                let bits = const { Src::Primary::BITS as i32 - (Tgt::BITS as i32 - Tgt::SIGNED as i32) };
                if Src::SIGNED && lz == 0 { Tgt::SIGNED && Self::leading_ones(src) as i32 >= bits } else { lz as i32 >= bits }
            }
        };
        if en { Some(unsafe { transmute(Self::cast_as::<_, Tgt::Primary>(src)) }) } else { None }
    }

    #[inline(always)]
    pub const fn clamp<Src: PrimaryInt>(src: Src, min: Src, max: Src) -> Src {
        macro_rules! imp {
            ($type:ty) => {
                unsafe {
                    let (src, min, max) = (transmute::<_, $type>(src), transmute::<_, $type>(min), transmute::<_, $type>(max));
                    transmute(if src < min {
                        min
                    } else if src > max {
                        max
                    } else {
                        src
                    })
                }
            };
        }
        dispatch!(imp)
    }

    #[inline(always)]
    pub const fn cast_as_primary<Src: BasicInt>(src: Src) -> Src::Primary {
        unsafe { transmute(src) }
    }

    #[inline(always)]
    pub const fn cast_from_primary<Src: BasicInt>(src: Src::Primary) -> Src {
        if Src::BITS != Src::Primary::BITS {
            if Src::BITS == 0 {
                Src::ZERO
            } else {
                let unused_bits = const { Src::Primary::BITS - Src::BITS };
                macro_rules! imp {
                    ($type:ty) => {
                        unsafe { transmute(transmute::<_, $type>(src) << unused_bits >> unused_bits) }
                    };
                }
                dispatch!(imp)
            }
        } else {
            unsafe { transmute(src) }
        }
    }

    #[inline(always)]
    pub const fn is_zero<Src: BasicInt>(src: Src) -> bool {
        Self::eq(src, Src::ZERO)
    }

    #[inline(always)]
    pub const fn cast_as_signed<Src: PrimaryInt>(src: Src) -> Src::Signed {
        unsafe { transmute(src) }
    }

    #[inline(always)]
    pub const fn cast_from_signed<Tgt: PrimaryInt>(src: Tgt::Signed) -> Tgt {
        unsafe { transmute(src) }
    }

    #[inline(always)]
    pub const fn cast_as_unsigned<Src: PrimaryInt>(src: Src) -> Src::Unsigned {
        unsafe { transmute(src) }
    }

    #[inline(always)]
    pub const fn cast_from_unsigned<Tgt: PrimaryInt>(src: Tgt::Unsigned) -> Tgt {
        unsafe { transmute(src) }
    }

    #[inline(always)]
    pub const fn cast_as_calc<Src: PrimaryInt>(src: Src) -> Src::CalcFitted {
        Self::cast_as(src)
    }

    binary!(abs_diff, as_src, as_unsigned);

    binary!(checked_add, as_src, map_option);

    binary!(checked_div, as_src, map_option);

    binary!(checked_div_euclid, as_src, map_option);

    binary!(checked_ilog, as_src, as_option_u32, imp_binary_fixed_return);

    unary!(checked_ilog2, as_option_u32, imp_unary_fixed_return);

    unary!(checked_ilog10, as_option_u32, imp_unary_fixed_return);

    binary!(checked_mul, as_src, map_option);

    unary!(checked_neg, map_option);

    binary!(checked_pow, as_u32, map_option, imp_binary_fixed_right);

    binary!(checked_rem, as_src, map_option);

    binary!(checked_rem_euclid, as_src, map_option);

    binary!(checked_shl, as_u32, map_option, imp_binary_fixed_right);

    binary!(checked_shr, as_u32, map_option, imp_binary_fixed_right);

    binary!(checked_sub, as_src, map_option);

    unary!(count_ones, as_u32, imp_unary_fixed_return);

    unary!(count_zeros, as_u32, imp_unary_fixed_return);

    binary!(div_euclid, as_src, as_src);

    binary!(ilog, as_src, as_u32, imp_binary_fixed_return);

    unary!(ilog2, as_u32, imp_unary_fixed_return);

    unary!(ilog10, as_u32, imp_unary_fixed_return);

    unary!(leading_ones, as_u32, imp_unary_fixed_return);

    unary!(leading_zeros, as_u32, imp_unary_fixed_return);

    binary!(midpoint, as_src, as_src);

    binary!(overflowing_add, as_src, map_overflow);

    binary!(overflowing_div, as_src, map_overflow);

    binary!(overflowing_div_euclid, as_src, map_overflow);

    binary!(overflowing_mul, as_src, map_overflow);

    unary!(overflowing_neg, map_overflow);

    binary!(overflowing_pow, as_u32, map_overflow, imp_binary_fixed_right);

    binary!(overflowing_rem, as_src, map_overflow);

    binary!(overflowing_rem_euclid, as_src, map_overflow);

    binary!(overflowing_shl, as_u32, map_overflow, imp_binary_fixed_right);

    binary!(overflowing_shr, as_u32, map_overflow, imp_binary_fixed_right);

    binary!(overflowing_sub, as_src, map_overflow);

    binary!(pow, as_u32, as_src, imp_binary_fixed_right);

    binary!(rem_euclid, as_src, as_src);

    unary!(reverse_bits, as_src);

    binary!(rotate_left, as_u32, as_src, imp_binary_fixed_right);

    binary!(rotate_right, as_u32, as_src, imp_binary_fixed_right);

    binary!(saturating_add, as_src, as_src);

    binary!(saturating_div, as_src, as_src);

    binary!(saturating_mul, as_src, as_src);

    binary!(saturating_pow, as_u32, as_src, imp_binary_fixed_right);

    binary!(saturating_sub, as_src, as_src);

    unary!(trailing_ones, as_u32, imp_unary_fixed_return);

    unary!(trailing_zeros, as_u32, imp_unary_fixed_return);

    binary!(unbounded_shl, as_u32, as_src);

    binary!(unbounded_shr, as_u32, as_src);

    binary!(unchecked_add, as_src, as_src);

    binary!(unchecked_mul, as_src, as_src);

    binary!(unchecked_sub, as_src, as_src);

    binary!(wrapping_add, as_src, as_src);

    binary!(wrapping_div, as_src, as_src);

    binary!(wrapping_div_euclid, as_src, as_src);

    binary!(wrapping_mul, as_src, as_src);

    unary!(wrapping_neg, as_src);

    binary!(wrapping_pow, as_u32, as_src, imp_binary_fixed_right);

    binary!(wrapping_rem, as_src, as_src);

    binary!(wrapping_rem_euclid, as_src, as_src);

    binary!(wrapping_shl, as_u32, as_src, imp_binary_fixed_right);

    binary!(wrapping_shr, as_u32, as_src, imp_binary_fixed_right);

    binary!(wrapping_sub, as_src, as_src);

    #[inline(always)]
    pub const fn is_multiple_of<Src: BasicInt>(src: Src, right: Src) -> bool {
        macro_rules! imp {
            ($type:ty) => {
                unsafe {
                    let (src, right) = (transmute::<Src, $type>(src), transmute::<Src, $type>(right));
                    right > 0 && src % right == 0
                }
            };
        }
        dispatch!(imp)
    }
}

impl CInt {
    unary_signed!(abs, as_src);

    unary_signed!(cast_unsigned, as_unsigned);

    unary_signed!(checked_abs, map_option);

    binary_signed!(checked_add_unsigned, as_unsigned, map_option);

    binary_signed!(checked_sub_unsigned, as_unsigned, map_option);

    unary_signed!(is_negative, as_bool, imp_unary_fixed_return);

    unary_signed!(is_positive, as_bool, imp_unary_fixed_return);

    unary_signed!(overflowing_abs, map_overflow);

    binary_signed!(overflowing_add_unsigned, as_unsigned, map_overflow);

    binary_signed!(overflowing_sub_unsigned, as_unsigned, map_overflow);

    unary_signed!(saturating_abs, as_src);

    binary_signed!(saturating_add_unsigned, as_unsigned, as_src);

    unary_signed!(saturating_neg, as_src);

    binary_signed!(saturating_sub_unsigned, as_unsigned, as_src);

    unary_signed!(signum, as_src);

    unary_signed!(unsigned_abs, as_unsigned);

    unary_signed!(wrapping_abs, as_src);

    binary_signed!(wrapping_add_unsigned, as_unsigned, as_src);

    binary_signed!(wrapping_sub_unsigned, as_unsigned, as_src);
}

impl CInt {
    unary_unsigned!(cast_signed, as_signed);

    binary_unsigned!(checked_add_signed, as_signed, map_option);

    binary_unsigned!(checked_next_multiple_of, as_src, map_option);

    unary_unsigned!(checked_next_power_of_two, map_option);

    binary_unsigned!(div_ceil, as_src, as_src);

    unary_unsigned!(is_power_of_two, as_bool, imp_unary_fixed_return);

    binary_unsigned!(next_multiple_of, as_src, as_src);

    unary_unsigned!(next_power_of_two, as_src);

    binary_unsigned!(overflowing_add_signed, as_signed, map_overflow);

    binary_unsigned!(saturating_add_signed, as_signed, as_src);

    binary_unsigned!(wrapping_add_signed, as_signed, as_src);
}

macro_rules! imp_operator {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute(transmute::<_,$type>($src) $op transmute::<_,$type>($right)) }
    };
}

macro_rules! operator {
    ($catalog:ident, $name:ident, $op:tt) => {
        #[inline(always)]
        pub const fn $name<Src: $catalog>(src: Src, right: Src) -> Src {
            dispatch!(imp_operator, $op, src, right)
        }
    };
}

macro_rules! imp_operator_f {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute(transmute::<_,$type>($src) $op $right) }
    };
}

macro_rules! operator_f {
    ($catalog:ident, $name:ident, $op:tt, $right:ty) => {
        #[inline(always)]
        pub const fn $name<Src: $catalog>(src: Src, right: $right) -> Src {
            dispatch!(imp_operator_f, $op, src, right)
        }
    };
}

macro_rules! imp_operator_r {
    ($type:ty, $op:tt, $src:ident, $right:ident) => {
        unsafe { transmute::<_,$type>($src) $op transmute::<_,$type>($right) }
    };
}

macro_rules! operator_r {
    ($name:ident, $op:tt, $result:ty) => {
        #[inline(always)]
        pub const fn $name<Src: BasicInt>(src: Src, right: Src) -> $result {
            dispatch!(imp_operator_r, $op, src, right)
        }
    };
}

macro_rules! ex_operator {
    ($name:ident, $origin:ident, $right:ty) => {
        #[inline(always)]
        pub const fn $name<Src: BasicInt>(src: Src, right: $right) -> Src {
            Self::cast_from_primary(Self::$origin(Self::cast_as_primary(src), right))
        }
    };
    ($name:ident, $origin:ident) => {
        #[inline(always)]
        pub const fn $name<Src: BasicInt>(src: Src, right: Src) -> Src {
            Self::cast_from_primary(Self::$origin(Self::cast_as_primary(src), Self::cast_as_primary(right)))
        }
    };
}

/// const trait
impl CInt {
    operator!(PrimaryInt, add, +);
    ex_operator!(ex_add, add);

    operator!(PrimaryInt, mul, *);
    ex_operator!(ex_mul, mul);

    operator!(PrimaryInt, sub, -);
    ex_operator!(ex_sub, sub);

    operator!(BasicInt, bitand, &);
    operator!(BasicInt, bitor, |);
    operator!(BasicInt, bitxor, ^);
    operator!(BasicInt, div, /);
    operator!(BasicInt, rem, %);
    operator_r!(lt, <, bool);
    operator_r!(le, <=, bool);
    operator_r!(gt, >, bool);
    operator_r!(ge, >=, bool);

    operator_f!(PrimaryInt, shl, <<, u32);
    ex_operator!(ex_shl, shl, u32);

    operator_f!(BasicInt, shr, >>, u32);

    #[inline(always)]
    pub const fn eq<Src: BasicInt>(src: Src, right: Src) -> bool {
        dispatch_unsigned!(imp_operator_r, ==, src, right)
    }

    #[inline(always)]
    pub const fn not<Src: PrimaryInt>(src: Src) -> Src {
        macro_rules! imp {
            ($type:ty) => {
                unsafe { transmute(!transmute::<Src, $type>(src)) }
            };
        }
        dispatch_unsigned!(imp)
    }

    #[inline(always)]
    pub const fn ex_not<Src: BasicInt>(src: Src) -> Src {
        Self::cast_from_primary(Self::not(Self::cast_as_primary(src)))
    }
}

macro_rules! imp_bytes {
    ($type:ty, $op:ident, $src:ident) => {
        unsafe { transmute(<$type>::$op(transmute($src))) }
    };
}
macro_rules! def_bytes {
    ($name:ident, $src:ident, $result:ident) => {
        #[inline(always)]
        pub const fn $name<Src: PrimaryInt>(src: $src!(Src)) -> $result!(Src) {
            dispatch_unsigned!(imp_bytes, $name, src)
        }
    };
}

impl CInt {
    def_bytes!(swap_bytes, as_src, as_src);

    def_bytes!(from_be, as_src, as_src);

    def_bytes!(from_be_bytes, as_bytes_array, as_src);

    def_bytes!(from_le, as_src, as_src);

    def_bytes!(from_le_bytes, as_bytes_array, as_src);

    def_bytes!(from_ne_bytes, as_bytes_array, as_src);

    def_bytes!(to_be, as_src, as_src);

    def_bytes!(to_be_bytes, as_src, as_bytes_array);

    def_bytes!(to_le, as_src, as_src);

    def_bytes!(to_le_bytes, as_src, as_bytes_array);

    def_bytes!(to_ne_bytes, as_src, as_bytes_array);
}

#[inline(always)]
pub(super) const unsafe fn transmute<Src: Copy, Tgt: Copy>(src: Src) -> Tgt {
    unsafe { *(&src as *const Src as *const Tgt) }
}
