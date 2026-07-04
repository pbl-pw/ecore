use core::{fmt::Debug, hint::unreachable_unchecked, marker::PhantomData};

use bytemuck::{Pod, Zeroable};
use const_default::ConstDefault;

use crate::{
    bitfld::{BitsCast, PlainBitsCast},
    int::{BasicInt, BasicUInt, CInt, PrimaryInt, PrimaryUInt},
    repr::{ConvertEndian, Uncheckable},
};

/// Bits enum which can re-layout tag and payload
pub trait ReLayoutBitsEnum: Copy {
    const TAG_START: u32;
    const PAYLOAD_START: u32;
    type Repr: PrimaryUInt;
    type Tag: BasicInt;
    type Payload: BasicUInt;
    #[allow(clippy::result_unit_err)]
    fn try_from_raw_parts(tag: Self::Repr, payload: Self::Repr) -> Result<Self, ()>;
    fn into_raw_parts(self) -> (Self::Repr, Self::Repr);
}

#[repr(transparent)]
pub struct BitsEnumReLayout<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> {
    bits: Bits,
    mark: PhantomData<Enum>,
}

impl<Enum: ReLayoutBitsEnum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    const TAG_END: u32 = TAG_START + Enum::Tag::BITS;
    const PAYLOAD_END: u32 = PAYLOAD_START + Enum::Tag::BITS;
    const ASSERT: () = assert!(
        Self::TAG_END <= Bits::BITS && Self::PAYLOAD_END <= Bits::BITS && (TAG_START >= Self::PAYLOAD_END || PAYLOAD_START >= Self::TAG_END),
        "tag and payload bits range must inside target bits range, and must not overlay"
    );

    const TAG_MARK: Bits::Primary = CInt::not(CInt::shl(Bits::Primary::ALL_ONE, Enum::Tag::BITS));
    const PAYLOAD_MARK: Bits::Primary = CInt::not(CInt::shl(Bits::Primary::ALL_ONE, Enum::Payload::BITS));

    /// Reads the enum value from the packed bit representation.
    pub fn get(self) -> Enum {
        let Ok(out) = Enum::try_from_raw_parts((self.bits >> TAG_START).cast_as(), (self.bits >> PAYLOAD_START).cast_as()) else {
            unsafe { unreachable_unchecked() }
        };
        out
    }

    /// Creates a packed bit representation from an enum value.
    pub fn new(src: Enum) -> Self {
        let () = Self::ASSERT;
        let (tag, payload) = src.into_raw_parts();
        let tag: Bits::Primary = tag.cast_as();
        let payload: Bits::Primary = payload.cast_as();
        Self { bits: Bits::cast_from_primary((tag & Self::TAG_MARK) << TAG_START | (payload & Self::PAYLOAD_MARK) << PAYLOAD_START), mark: PhantomData }
    }

    /// Constructs from raw tag and payload values, applying the appropriate masks and shifts.
    pub const fn from_raw_parts<T: PrimaryUInt>(tag: T, payload: T) -> Self {
        let () = Self::ASSERT;
        Self { bits: CInt::cast_from_primary(Self::combine(CInt::cast_as(tag), CInt::cast_as(payload))), mark: PhantomData }
    }

    /// Decomposes into raw tag and payload values.
    pub const fn into_raw_parts<T: PrimaryUInt>(self) -> (T, T) {
        const { assert!(T::BITS >= Enum::Tag::BITS && T::BITS >= Enum::Payload::BITS) }
        let (tag, payload) = Self::split(CInt::cast_as_primary(self.bits));
        (CInt::cast_as(tag), CInt::cast_as(payload))
    }

    /// Constructs directly from the packed bits type.
    pub const fn from_bits(bits: Bits) -> Self {
        let () = Self::ASSERT;
        Self { bits, mark: PhantomData }
    }

    /// Returns the underlying packed bits.
    pub const fn into_bits(self) -> Bits {
        self.bits
    }

    #[inline(always)]
    const fn combine(tag: Bits::Primary, payload: Bits::Primary) -> Bits::Primary {
        use core::mem::transmute_copy;
        macro_rules! imp {
            ($type:ty) => {{
                let tag: $type = transmute_copy(&tag);
                let payload: $type = transmute_copy(&payload);
                let tag_mark: $type = transmute_copy(&Self::TAG_MARK);
                let payload_mark: $type = transmute_copy(&Self::PAYLOAD_MARK);
                let bits = (tag & tag_mark) << TAG_START | (payload & payload_mark) << PAYLOAD_START;
                transmute_copy(&bits)
            }};
        }
        unsafe {
            match Bits::Primary::BITS {
                8 => imp!(u8),
                16 => imp!(u16),
                32 => imp!(u32),
                64 => imp!(u64),
                128 => imp!(u128),
                _ => unreachable_unchecked(),
            }
        }
    }

    #[inline(always)]
    const fn split(bits: Bits::Primary) -> (Bits::Primary, Bits::Primary) {
        use core::mem::transmute_copy;
        macro_rules! imp {
            ($type:ty) => {{
                let bits: $type = transmute_copy(&bits);
                (transmute_copy(&(bits >> TAG_START)), transmute_copy(&(bits >> PAYLOAD_START)))
            }};
        }
        unsafe {
            match Bits::Primary::BITS {
                8 => imp!(u8),
                16 => imp!(u16),
                32 => imp!(u32),
                64 => imp!(u64),
                128 => imp!(u128),
                _ => unreachable_unchecked(),
            }
        }
    }
}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> BitsCast for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    const BITS: u32 = Bits::BITS;

    fn from_underlying<SBits: PrimaryInt>(v: SBits) -> Self {
        Self { bits: v.cast_as(), mark: PhantomData }
    }

    fn into_underlying<SBits: PrimaryInt>(sf: Self) -> SBits {
        sf.bits.cast_as()
    }
}

unsafe impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> PlainBitsCast for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    type Bits = Bits;
}

impl<Enum: ReLayoutBitsEnum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> ReLayoutBitsEnum
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    const TAG_START: u32 = TAG_START;
    const PAYLOAD_START: u32 = PAYLOAD_START;
    type Repr = Enum::Repr;
    type Tag = Enum::Tag;
    type Payload = Enum::Payload;

    fn try_from_raw_parts(tag: Self::Repr, payload: Self::Repr) -> Result<Self, ()> {
        Enum::try_from_raw_parts(tag, payload).map(Self::new)
    }

    fn into_raw_parts(self) -> (Self::Repr, Self::Repr) {
        self.into_raw_parts()
    }
}

impl<Enum: 'static, Bits: BasicUInt + Uncheckable, const TAG_START: u32, const PAYLOAD_START: u32> Uncheckable
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    type UncheckedRaw = <Bits as Uncheckable>::UncheckedRaw;

    fn raw_value(sf: Self) -> Self::UncheckedRaw {
        Bits::raw_value(sf.bits)
    }

    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
        Ok(Self { bits: Bits::try_from_raw_value(raw)?, mark: PhantomData })
    }
}

unsafe impl<Enum: 'static, Bits: BasicUInt + ConvertEndian, const TAG_START: u32, const PAYLOAD_START: u32> ConvertEndian
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    type Store = <Bits as ConvertEndian>::Store;

    unsafe fn from_store(store: Self::Store) -> Self {
        Self { bits: unsafe { Bits::from_store(store) }, mark: PhantomData }
    }

    fn into_store(src: Self) -> Self::Store {
        Bits::into_store(src.bits)
    }
}

unsafe impl<Enum: 'static, Bits: BasicUInt + Pod, const TAG_START: u32, const PAYLOAD_START: u32> Pod
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
}

unsafe impl<Enum, Bits: BasicUInt + Zeroable, const TAG_START: u32, const PAYLOAD_START: u32> Zeroable
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> Copy for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> Clone for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> Eq for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> PartialEq for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> Default for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    fn default() -> Self {
        Self { bits: Default::default(), mark: PhantomData }
    }
}

impl<Enum, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> ConstDefault for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START> {
    const DEFAULT: Self = Self { bits: Bits::DEFAULT, mark: PhantomData };
}

impl<Enum: ReLayoutBitsEnum + Debug, Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> Debug
    for BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get().fmt(f)
    }
}
