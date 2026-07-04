use core::{fmt, marker::PhantomData};

use crate::{
    int::{BasicInt, BasicUInt, CInt, PrimaryInt},
    repr::{PlainUncheckable, Uncheckable, Unchecked},
};

pub use ecore_macro::rint;

/// provide range for RInt
pub trait RIntRange: Copy {
    type Value: PrimaryInt;
    const MIN: Self::Value;
    const MAX: Self::Value;
    const DEFAULT: Self::Value;
    const STEP: Self::Value;
}

/// RInt which exhaustive stored value
pub trait RIntExhaustive {}

macro_rules! gen_range {
    ($name:ident, $type:ty) => {
        #[doc = "range of RInt"]
        #[derive(Clone, Copy, Default)]
        pub struct $name<const MIN: $type, const MAX: $type, const DEFAULT: $type = MIN, const STEP: $type = 1>;

        impl<const MIN: $type, const MAX: $type, const DEFAULT: $type, const STEP: $type> RIntRange for $name<MIN, MAX, DEFAULT, STEP> {
            type Value = $type;
            const MIN: Self::Value = MIN;
            const MAX: Self::Value = MAX;
            const DEFAULT: Self::Value = DEFAULT;
            const STEP: Self::Value = STEP;
        }
        impl<const DEFAULT: $type> RIntExhaustive for RInt<<$type as BasicInt>::Unsigned, $name<{ <$type>::MIN }, { <$type>::MAX }, DEFAULT>> {}
    };
}
gen_range!(RRU8, u8);
gen_range!(RRU16, u16);
gen_range!(RRU32, u32);
gen_range!(RRU64, u64);
gen_range!(RRU128, u128);
gen_range!(RRUsize, usize);
gen_range!(RRI8, i8);
gen_range!(RRI16, i16);
gen_range!(RRI32, i32);
gen_range!(RRI64, i64);
gen_range!(RRI128, i128);
gen_range!(RRIsize, isize);

/// int value type limit in range MIN..=MAX, but memory size narrowed to STORE type, when memory raw value is 0, then represent value is DEFAULT
#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct RInt<STORE, RANGE> {
    store: STORE,
    range: PhantomData<RANGE>,
}

impl<STORE: BasicUInt, RANGE: RIntRange> RInt<STORE, RANGE> {
    pub const BITS: u32 = STORE::BITS;
    /// Minimum valid value.
    pub const MIN: Self = unsafe { Self::new_unchecked(RANGE::MIN) };
    /// Maximum valid value.
    pub const MAX: Self = unsafe { Self::new_unchecked(RANGE::MAX) };
    /// Default value (typically `RANGE::MIN` unless specified).
    pub const DEFAULT: Self = unsafe { Self::new_unchecked(RANGE::DEFAULT) };

    const MAX_DIFF: RANGE::Value = CInt::wrapping_sub(RANGE::MAX, RANGE::MIN);
    /// Whether the value range exhaustively occupies all possible bit patterns of `STORE`.
    pub const EXHAUSTIVE: bool = CInt::eq(Self::MAX_DIFF, CInt::ex_cast_as(STORE::MAX));
    /// Number of unused bits after narrowing.
    pub const REMAIN_BITS: u32 = RANGE::Value::BITS.checked_sub(Self::BITS).unwrap();
    /// Whether the stored type is narrower than the logical value type.
    pub const NARROWED: bool = Self::REMAIN_BITS != 0;
    /// Whether the step size has been trimmed (step > 1 and constrained by remaining bits).
    pub const STEP_TRIMED: bool = CInt::gt(RANGE::STEP, RANGE::Value::ONE) && CInt::leading_zeros(Self::MAX_DIFF) < Self::REMAIN_BITS;

    pub const VALUE_OFFSET: RANGE::Value = if Self::STEP_TRIMED {
        CInt::div(RANGE::MIN, RANGE::STEP)
    } else if Self::NARROWED {
        RANGE::MIN
    } else {
        RANGE::DEFAULT
    };

    pub const STORE_OFFSET: RANGE::Value = if Self::STEP_TRIMED {
        CInt::div(CInt::sub(RANGE::DEFAULT, RANGE::MIN), RANGE::STEP)
    } else if Self::NARROWED {
        CInt::sub(RANGE::DEFAULT, RANGE::MIN)
    } else {
        RANGE::Value::ZERO
    };

    const MASK: RANGE::Value = CInt::ex_cast_as(STORE::ALL_ONE);
    const VALID: bool = STORE::BITS <= RANGE::Value::BITS
        && CInt::le(RANGE::MIN, RANGE::DEFAULT)
        && CInt::le(RANGE::DEFAULT, RANGE::MAX)
        && CInt::ge(RANGE::STEP, RANGE::Value::ONE)
        && CInt::is_multiple_of(RANGE::MIN, RANGE::STEP)
        && CInt::is_multiple_of(RANGE::MAX, RANGE::STEP)
        && CInt::is_multiple_of(RANGE::DEFAULT, RANGE::STEP);
    const ASSERT: () = assert!(Self::VALID && CInt::leading_zeros(CInt::div(Self::MAX_DIFF, RANGE::STEP)) >= Self::REMAIN_BITS);

    const fn is_valid(value: RANGE::Value) -> bool {
        if CInt::gt(RANGE::STEP, RANGE::Value::ONE) {
            CInt::le(RANGE::MIN, value) && CInt::le(value, RANGE::MAX) && CInt::is_multiple_of(value, RANGE::STEP)
        } else {
            CInt::le(RANGE::MIN, value) && CInt::le(value, RANGE::MAX)
        }
    }

    /// Returns the raw stored value without any range validation.
    pub const fn raw_value(self) -> STORE {
        self.store
    }

    /// Constructs an `RInt` from a raw store value without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure `store` represents a valid value within the range.
    pub const unsafe fn new_with_raw_value(store: STORE) -> Self {
        let () = Self::ASSERT;
        Self { store, range: PhantomData }
    }

    /// # Safety
    ///
    /// The caller must ensure `value` is within the valid range `[RANGE::MIN, RANGE::MAX]`
    /// and is a multiple of `RANGE::STEP`.
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: RANGE::Value) -> Self {
        let () = Self::ASSERT;
        let value = if Self::STEP_TRIMED { CInt::div(value, RANGE::STEP) } else { value };
        let value = if CInt::is_zero(RANGE::DEFAULT) {
            value
        } else {
            CInt::wrapping_sub(value, const { if Self::STEP_TRIMED { CInt::div(RANGE::DEFAULT, RANGE::STEP) } else { RANGE::DEFAULT } })
        };
        Self { store: CInt::ex_cast_as(value), range: PhantomData }
    }

    #[inline(always)]
    pub const fn value(self) -> RANGE::Value {
        let v = CInt::ex_cast_as(self.store);
        let v = if CInt::is_zero(Self::VALUE_OFFSET) && CInt::is_zero(Self::STORE_OFFSET) {
            v
        } else if CInt::is_zero(Self::STORE_OFFSET) {
            CInt::wrapping_add(v, Self::VALUE_OFFSET)
        } else {
            CInt::wrapping_add(CInt::bitand(CInt::wrapping_add(v, Self::STORE_OFFSET), Self::MASK), Self::VALUE_OFFSET)
        };
        if Self::STEP_TRIMED { CInt::mul(v, RANGE::STEP) } else { v }
    }

    pub const fn new(value: RANGE::Value) -> Option<Self> {
        if Self::is_valid(value) { unsafe { Some(Self::new_unchecked(value)) } } else { None }
    }

    pub const fn new_clamped(value: RANGE::Value) -> Option<Self> {
        let value = CInt::clamp(value, RANGE::MIN, RANGE::MAX);
        if const { CInt::gt(RANGE::STEP, RANGE::Value::ONE) } && !CInt::is_multiple_of(value, RANGE::STEP) {
            None
        } else {
            Some(unsafe { Self::new_unchecked(value) })
        }
    }

    #[inline(always)]
    pub const fn into_unchecked(self) -> Unchecked<Self> {
        Unchecked::new_with_raw_value(self.store)
    }
}

impl<STORE: BasicUInt, RANGE: RIntRange> Uncheckable for RInt<STORE, RANGE> {
    type UncheckedRaw = STORE;
    fn raw_value(sf: Self) -> Self::UncheckedRaw {
        sf.store
    }
    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
        let v = unsafe { Self::new_with_raw_value(raw) };
        if Self::is_valid(v.value()) { Ok(v) } else { Err(raw) }
    }
}
unsafe impl<STORE: BasicUInt, RANGE: RIntRange> PlainUncheckable for RInt<STORE, RANGE> {}

impl<STORE: BasicUInt, RANGE: RIntRange> fmt::Debug for RInt<STORE, RANGE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?})", self.value(), self.store)
    }
}

impl<Value: PrimaryInt, STORE: BasicUInt, RANGE: RIntRange<Value = Value>> From<Value> for Unchecked<RInt<STORE, RANGE>> {
    fn from(value: Value) -> Self {
        Self::new_with_raw_value(unsafe { RInt::<STORE, RANGE>::new_unchecked(value) }.raw_value())
    }
}

impl<STORE: BasicUInt, RANGE: RIntRange> TryFrom<Unchecked<RInt<STORE, RANGE>>> for RInt<STORE, RANGE> {
    type Error = ();
    fn try_from(value: Unchecked<RInt<STORE, RANGE>>) -> Result<Self, Self::Error> {
        let v = Self { store: value.raw_value(), range: PhantomData };
        if Self::is_valid(v.value()) { Ok(v) } else { Err(()) }
    }
}

impl<STORE: BasicUInt, RANGE: RIntRange> Unchecked<RInt<STORE, RANGE>> {
    pub const MIN: RInt<STORE, RANGE> = RInt::<STORE, RANGE>::MIN;
    pub const MAX: RInt<STORE, RANGE> = RInt::<STORE, RANGE>::MAX;

    pub const fn value(self) -> Option<RANGE::Value> {
        let v = unsafe { RInt::<STORE, RANGE>::new_with_raw_value(self.raw_value()) }.value();
        if RInt::<STORE, RANGE>::is_valid(v) { Some(v) } else { None }
    }

    /// # Safety
    ///
    /// The caller must ensure the contained raw value represents a valid value within the range.
    pub const unsafe fn value_unchecked(self) -> RANGE::Value {
        unsafe { RInt::<STORE, RANGE>::new_with_raw_value(self.raw_value()) }.value()
    }

    pub fn value_or_default(self) -> RANGE::Value {
        self.value().unwrap_or_default()
    }
}

impl<STORE: Eq, RANGE> Eq for RInt<STORE, RANGE> {}
impl<STORE: PartialEq, RANGE> PartialEq for RInt<STORE, RANGE> {
    fn eq(&self, other: &Self) -> bool {
        self.store == other.store
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod test {
    use super::*;
    use crate::{
        bitfld::prelude::*,
        bitint::{u7, u13},
    };

    /// ```compile_fail
    /// let _ = RInt::<u8,RRU8<3, 4, 2, 0>>::new(0);
    /// ```
    #[test]
    fn i8_value() {
        let v = RInt::<u8, RRI8<-128, 127>>::new(-128).unwrap();
        assert_eq!(v.store, 0);
        assert_eq!(v.value(), -128);
        let v = RInt::<u8, RRI8<-128, 127, 127>>::new(-128).unwrap();
        assert_eq!(v.store, 1);
        assert_eq!(v.value(), -128);
        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u8, RRI8<-128, 127, -11>>>, 5..=12),
        }

        for i in -128i8..=127i8 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(v.ek().get().unwrap().value(), i);
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), -11);
    }
    #[test]
    fn u8_value() {
        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u8, RRU8<0, 255, 11>>>, 5..=12),
        }

        for i in 0..=255u8 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(v.ek().get().unwrap().value(), i);
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), 11);
    }
    #[test]
    fn i7_value() {
        let v = RInt::<u7, RRI8<-64, 63>>::default();
        assert_eq!(v.store.value(), 0);
        assert_eq!(v.value(), -64);

        let v = RInt::<u7, RRI8<-64, 63, -64>>::new(-64).unwrap();
        assert_eq!(v.store.value(), 0);
        assert_eq!(v.value(), -64);

        let v = RInt::<u7, RRI8<-64, 63, 63>>::default();
        assert_eq!(v.store.value(), 0);
        assert_eq!(v.value(), 63);

        let mut v = RInt::<u7, RRI8<-64, 63, 63>>::new(-64).unwrap();
        assert_eq!(v.value(), -64);
        for i in -64..=63 {
            v = RInt::new(i).unwrap();
            assert_eq!(v.value(), i);
        }

        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u7, RRI8<-64, 63, -10>>>, 5..=11),
        }

        for i in -64i8..=63i8 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(v.ek().get().unwrap().value(), i);
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), -10);
    }

    #[test]
    fn i7_value_ex() {
        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u7, RRI16<-127, 0, -10>>>, 5..=11),
        }

        for i in -127..=0 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(v.ek().get().unwrap().value(), i);
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), -10);
    }

    #[test]
    fn u7_value() {
        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u7, RRU8<0, 127, 11>>>, 5..=11),
        }

        for i in 0u8..=127u8 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(v.ek().get().unwrap().value(), i);
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), 11);
    }

    #[test]
    fn u7_value_ex() {
        #[bitfld(u32)]
        pub struct STe {
            ek: bitfld!(Unchecked<RInt<u7, RRU16<1000, 1127, 1010>>>, 5..=11),
        }

        for i in 1000..=1127 {
            let v = STe(0).ek().with(RInt::new(i).unwrap());
            assert_eq!(i, v.ek().get().unwrap().value());
        }
        assert_eq!(STe(0).ek().get().unwrap().value(), 1010);
    }

    #[test]
    fn step_i16() {
        {
            type Ptype = RInt<u8, RRI16<-1280, 1270, -10, 10>>;
            assert!(Ptype::NARROWED);
            assert!(Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(-10).unwrap().store, 0);
            for i in -1280i16..=1270i16 {
                if i % 10 != 0 {
                    continue;
                }
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
        {
            type Ptype = RInt<u13, RRI16<-1280, 1270, -10, 10>>;
            assert!(Ptype::NARROWED);
            assert!(!Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(-10).unwrap().store, u13::new(0).unwrap());
            for i in -1280i16..=1270i16 {
                if i % 10 != 0 {
                    continue;
                }
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
        {
            type Ptype = RInt<u16, RRI16<-1280, 1270, -10, 10>>;
            assert!(!Ptype::NARROWED);
            assert!(!Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(-10).unwrap().store, 0);
            for i in -1280i16..=1270i16 {
                if i % 10 != 0 {
                    continue;
                }
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
    }

    #[test]
    fn step_u16() {
        {
            type Ptype = RInt<u8, RRU16<10, 2560, 50, 10>>;
            assert!(Ptype::NARROWED);
            assert!(Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(50).unwrap().store, 0);
            for i in 10..=2560 {
                let 0 = i % 10 else { continue };
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
        {
            type Ptype = RInt<u13, RRU16<10, 2560, 50, 10>>;
            assert!(Ptype::NARROWED);
            assert!(!Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(50).unwrap().store, u13::new(0).unwrap());
            for i in 10..=2560 {
                let 0 = i % 10 else { continue };
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
        {
            type Ptype = RInt<u16, RRU16<10, 2560, 50, 10>>;
            assert!(!Ptype::NARROWED);
            assert!(!Ptype::STEP_TRIMED);
            assert_eq!(Ptype::new(50).unwrap().store, 0);
            for i in 10..=2560 {
                let 0 = i % 10 else { continue };
                assert_eq!(Ptype::new(i).unwrap().value(), i);
            }
        }
    }
}
