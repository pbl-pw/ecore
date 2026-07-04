use core::{hash::Hash, mem::transmute_copy};

use bytemuck::{Pod, Zeroable};
use const_default::ConstDefault;

use crate::int::{CInt, PrimaryInt, PrimaryUInt};

pub use self::unchecked::{PlainUncheckable, Uncheckable, Unchecked};

mod unchecked;

/// Type which be store as primary int in underlying, so maybe has different endian
///
/// **unsafe**: implementer must ensure `transmute(transmute(Self)->Self::Store)->Self` is ok
///
/// # Safety
///
/// Implementors must ensure that converting to `Store` and back via `from_store`/`into_store` is sound.
pub unsafe trait ConvertEndian: Pod {
    type Store: PrimaryUInt;

    /// store must get from `into_store` and recovery endian
    ///
    /// # Safety
    ///
    /// The caller must ensure `store` was produced by `into_store` on a valid `Self` value.
    unsafe fn from_store(store: Self::Store) -> Self;
    fn into_store(src: Self) -> Self::Store;
}

/// unalign (packed) T type
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash, ConstDefault, Pod, Zeroable)]
pub struct Unalign<T>(pub T);

/// fixed little endian T type, no matter target endian
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default, ConstDefault)]
pub struct LEndian<T: ConvertEndian> {
    raw: T::Store,
}
#[allow(non_snake_case)]
#[inline(always)]
pub const fn LEndian<T: ConvertEndian>(v: T) -> LEndian<T> {
    LEndian { raw: CInt::to_le(unsafe { transmute_copy(&v) }) }
}

/// fixed big endian T type, no matter target endian
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default, ConstDefault)]
pub struct BEndian<T: ConvertEndian> {
    raw: T::Store,
}
#[allow(non_snake_case)]
#[inline(always)]
pub const fn BEndian<T: ConvertEndian>(v: T) -> BEndian<T> {
    BEndian { raw: CInt::to_be(unsafe { transmute_copy(&v) }) }
}

/// type T value but using alter memory repr, impl for T, [`Unalign<T>`], [`struct@LEndian<T>`], [`struct@BEndian<T>`], [`Unalign<LEndian<T>>`], [`Unalign<BEndian<T>>`]
pub trait AlterRepr<T>: sealed::Sealed<T> {
    const IS_SWEAP_BYTES: bool;
    fn from_std_repr(v: T) -> Self;
    fn into_std_repr(src: Self) -> T;
}

impl<T> AlterRepr<T> for T {
    const IS_SWEAP_BYTES: bool = false;

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        v
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        src
    }
}

impl<T> AlterRepr<T> for Unalign<T> {
    const IS_SWEAP_BYTES: bool = false;

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        Unalign(v)
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        src.0
    }
}

impl<T: ConvertEndian> AlterRepr<T> for LEndian<T> {
    const IS_SWEAP_BYTES: bool = cfg!(target_endian = "big");

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        LEndian(v)
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        unsafe { ConvertEndian::from_store(src.raw.to_le()) }
    }
}

impl<T: ConvertEndian> AlterRepr<T> for BEndian<T> {
    const IS_SWEAP_BYTES: bool = cfg!(target_endian = "little");

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        BEndian(v)
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        unsafe { ConvertEndian::from_store(src.raw.to_be()) }
    }
}

impl<T: ConvertEndian> AlterRepr<T> for Unalign<LEndian<T>> {
    const IS_SWEAP_BYTES: bool = <LEndian<T> as AlterRepr<T>>::IS_SWEAP_BYTES;

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        Self(LEndian::from_std_repr(v))
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        LEndian::into_std_repr(src.0)
    }
}

impl<T: ConvertEndian> AlterRepr<T> for Unalign<BEndian<T>> {
    const IS_SWEAP_BYTES: bool = <BEndian<T> as AlterRepr<T>>::IS_SWEAP_BYTES;

    #[inline(always)]
    fn from_std_repr(v: T) -> Self {
        Self(BEndian::from_std_repr(v))
    }

    #[inline(always)]
    fn into_std_repr(src: Self) -> T {
        BEndian::into_std_repr(src.0)
    }
}

/// fixed endian T type alternative repr, impl for [`struct@LEndian<T>`], [`struct@BEndian<T>`], [`Unalign<LEndian<T>>`], [`Unalign<BEndian<T>>`]
pub trait FixedEndian: AlterRepr<Self::Native> + Pod {
    type Native: ConvertEndian;
    const IS_LITTLE_ENDIAN: bool;
}

impl<T: ConvertEndian> FixedEndian for LEndian<T> {
    type Native = T;
    const IS_LITTLE_ENDIAN: bool = true;
}

impl<T: ConvertEndian> FixedEndian for BEndian<T> {
    type Native = T;
    const IS_LITTLE_ENDIAN: bool = false;
}

impl<T: ConvertEndian> FixedEndian for Unalign<LEndian<T>> {
    type Native = T;
    const IS_LITTLE_ENDIAN: bool = true;
}

impl<T: ConvertEndian> FixedEndian for Unalign<BEndian<T>> {
    type Native = T;
    const IS_LITTLE_ENDIAN: bool = false;
}

/// unalign and fixed endian T type alternative repr, impl for [`Unalign<LEndian<T>>`], [`Unalign<BEndian<T>>`]
pub trait UnalignFixedEndian: FixedEndian {}

impl<T: ConvertEndian> UnalignFixedEndian for Unalign<LEndian<T>> {}

impl<T: ConvertEndian> UnalignFixedEndian for Unalign<BEndian<T>> {}

/// aligned T type alternative repr, impl for T, [`struct@LEndian<T>`], [`struct@BEndian<T>`]
pub trait AlignedRepr<T>: AlterRepr<T> {}

impl<T> AlignedRepr<T> for T {}

impl<T: ConvertEndian> AlignedRepr<T> for LEndian<T> {}

impl<T: ConvertEndian> AlignedRepr<T> for BEndian<T> {}

impl<T: ConvertEndian> LEndian<T> {
    /// Creates a new little-endian wrapper from a native-endian value.
    pub const fn new(v: T) -> Self {
        LEndian(v)
    }

    /// Reads the stored value, converting it back to native endianness.
    pub const fn get(self) -> T {
        unsafe { transmute_copy(&CInt::from_le(self.raw)) }
    }

    /// Constructs a `LEndian` from its raw bytes (in native byte order).
    pub const fn from_bytes(bytes: <T::Store as PrimaryInt>::BytesArray) -> Self {
        Self { raw: CInt::from_ne_bytes(bytes) }
    }

    /// Serializes the stored value to raw bytes (in native byte order).
    pub const fn to_bytes(self) -> <T::Store as PrimaryInt>::BytesArray {
        CInt::to_ne_bytes(self.raw)
    }

    /// Reinterprets the little-endian storage as a different type `U` with the same underlying integer.
    pub const fn into<U: ConvertEndian<Store = T::Store>>(self) -> LEndian<U> {
        LEndian { raw: self.raw }
    }
}

impl<T: ConvertEndian> BEndian<T> {
    /// Creates a new big-endian wrapper from a native-endian value.
    pub const fn new(v: T) -> Self {
        BEndian(v)
    }

    /// Reads the stored value, converting it back to native endianness.
    pub const fn get(self) -> T {
        unsafe { transmute_copy(&CInt::from_be(self.raw)) }
    }

    /// Constructs a `BEndian` from its raw bytes (in native byte order).
    pub const fn from_bytes(bytes: <T::Store as PrimaryInt>::BytesArray) -> Self {
        Self { raw: CInt::from_ne_bytes(bytes) }
    }

    /// Serializes the stored value to raw bytes (in native byte order).
    pub const fn to_bytes(self) -> <T::Store as PrimaryInt>::BytesArray {
        CInt::to_ne_bytes(self.raw)
    }

    /// Reinterprets the big-endian storage as a different type `U` with the same underlying integer.
    pub const fn into<U: ConvertEndian<Store = T::Store>>(self) -> BEndian<U> {
        BEndian { raw: self.raw }
    }
}

impl<T: ConvertEndian> Unalign<LEndian<T>> {
    /// Reads the unaligned little-endian value, converting to native endianness.
    pub const fn get(self) -> T {
        self.0.get()
    }
}

impl<T: ConvertEndian> Unalign<BEndian<T>> {
    /// Reads the unaligned big-endian value, converting to native endianness.
    pub const fn get(self) -> T {
        self.0.get()
    }
}

unsafe impl<T: ConvertEndian> Pod for LEndian<T> {}

unsafe impl<T: ConvertEndian> Zeroable for LEndian<T> {}

unsafe impl<T: ConvertEndian> Pod for BEndian<T> {}

unsafe impl<T: ConvertEndian> Zeroable for BEndian<T> {}

impl<T: ConvertEndian> Hash for LEndian<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: ConvertEndian> Hash for BEndian<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: ConvertEndian + Ord> Ord for LEndian<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: ConvertEndian + PartialOrd> PartialOrd for LEndian<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: ConvertEndian + Ord> Ord for BEndian<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: ConvertEndian + PartialOrd> PartialOrd for BEndian<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: ConvertEndian> From<T> for LEndian<T> {
    fn from(value: T) -> Self {
        LEndian(value)
    }
}

impl<T: ConvertEndian> From<T> for BEndian<T> {
    fn from(value: T) -> Self {
        BEndian(value)
    }
}

impl<T: ConvertEndian> From<T> for Unalign<LEndian<T>> {
    fn from(value: T) -> Self {
        Self(LEndian(value))
    }
}

impl<T: ConvertEndian> From<T> for Unalign<BEndian<T>> {
    fn from(value: T) -> Self {
        Self(BEndian(value))
    }
}

impl<T> From<T> for Unalign<T> {
    fn from(value: T) -> Self {
        Unalign(value)
    }
}

macro_rules! implfmt {
    ($trait:path) => {
        impl<T: ConvertEndian + $trait> $trait for LEndian<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <T as $trait>::fmt(&self.get(), f)
            }
        }

        impl<T: ConvertEndian + $trait> $trait for BEndian<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                <T as $trait>::fmt(&self.get(), f)
            }
        }
    };
}
implfmt!(core::fmt::Debug);
implfmt!(core::fmt::Display);
implfmt!(core::fmt::Binary);
implfmt!(core::fmt::LowerHex);
implfmt!(core::fmt::UpperHex);
implfmt!(core::fmt::Octal);

macro_rules! impl_int_endian {
    ($type:ty) => {
        unsafe impl ConvertEndian for $type {
            type Store = <$type as PrimaryInt>::UnsignedPrimary;

            unsafe fn from_store(store: Self::Store) -> Self {
                store as Self
            }

            fn into_store(src: Self) -> Self::Store {
                src as Self::Store
            }
        }
    };
}
impl_int_endian!(u8);
impl_int_endian!(u16);
impl_int_endian!(u32);
impl_int_endian!(u64);
impl_int_endian!(u128);
impl_int_endian!(usize);
impl_int_endian!(i8);
impl_int_endian!(i16);
impl_int_endian!(i32);
impl_int_endian!(i64);
impl_int_endian!(i128);
impl_int_endian!(isize);

unsafe impl ConvertEndian for f32 {
    type Store = u32;

    unsafe fn from_store(store: Self::Store) -> Self {
        store as Self
    }

    fn into_store(src: Self) -> Self::Store {
        src as Self::Store
    }
}

unsafe impl ConvertEndian for f64 {
    type Store = u64;

    unsafe fn from_store(store: Self::Store) -> Self {
        store as Self
    }

    fn into_store(src: Self) -> Self::Store {
        src as Self::Store
    }
}

/// Unchecked<BitInt<T, BITS>> and Unchecked<RInt<PrimaryUInt, RANGE>> and Unchecked<Unchecked<RInt<BasicUInt, RANGE>>> is ConvertEndian
unsafe impl<T: Uncheckable<UncheckedRaw: ConvertEndian> + 'static> ConvertEndian for Unchecked<T> {
    type Store = <T::UncheckedRaw as ConvertEndian>::Store;

    unsafe fn from_store(store: Self::Store) -> Self {
        Self::new_with_raw_value(unsafe { ConvertEndian::from_store(store) })
    }

    fn into_store(src: Self) -> Self::Store {
        ConvertEndian::into_store(src.raw_value())
    }
}

mod sealed {
    use super::{BEndian, ConvertEndian, LEndian, Unalign};

    pub trait Sealed<T> {}
    impl<T> Sealed<T> for T {}
    impl<T: ConvertEndian> Sealed<T> for LEndian<T> {}
    impl<T: ConvertEndian> Sealed<T> for BEndian<T> {}
    impl<T> Sealed<T> for Unalign<T> {}
    impl<T: ConvertEndian> Sealed<T> for Unalign<LEndian<T>> {}
    impl<T: ConvertEndian> Sealed<T> for Unalign<BEndian<T>> {}
}
