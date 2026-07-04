use crate::{
    bitfld::{BitField, BitField2, BitsCast, DynBitField, PlainBitsCast},
    int::{BasicInt, BasicUInt, PrimaryUInt},
    repr::{PlainUncheckable, Uncheckable, Unchecked},
};

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast + PlainBitsCast> BitField<STRUCT, FLD, START, LAST> {
    /// Read bit field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[inline(always)]
    pub const fn const_read(&self) -> FLD {
        unsafe { from_bits(FieldOp::<START, LAST>::read_bits(into_bits(self.raw))) }
    }

    /// Gen new bit struct value from origin bit struct value with new field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[must_use]
    #[inline(always)]
    pub const fn const_with(&self, fld: impl IntoBitField<FLD>) -> STRUCT {
        unsafe { from_bits(FieldOp::<START, LAST>::with_fld(into_bits(self.raw), into_bit_field(fld))) }
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast + PlainBitsCast> DynBitField<FLD, STRUCT> {
    /// Read bit field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[inline(always)]
    pub const fn const_read(&self) -> FLD {
        unsafe {
            if <STRUCT::Bits as BasicInt>::Primary::BITS <= usize::BITS {
                let unused_bits = const { usize::BITS.wrapping_sub(FLD::BITS) };
                let left_unused_bits = unused_bits - self.start_bit();
                force_truncate(usize_read_bits(int_cast_up(into_bits(*self.underlying())), left_unused_bits, unused_bits, FLD::Bits::SIGNED))
            } else {
                let unused_bits = const { u128::BITS.wrapping_sub(FLD::BITS) };
                let left_unused_bits = unused_bits - self.start_bit();
                force_truncate(u128_read_bits(int_cast_up(into_bits(*self.underlying())), left_unused_bits, unused_bits, FLD::Bits::SIGNED))
            }
        }
    }

    /// Gen new bit struct value from origin bit struct value with new field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[must_use]
    #[inline(always)]
    pub const fn const_with(&self, fld: impl IntoBitField<FLD>) -> STRUCT {
        let src = unsafe { into_bits(*self.underlying()) };
        let fld = unsafe { into_bit_field(fld) };
        unsafe {
            if <STRUCT::Bits as BasicInt>::Primary::BITS <= usize::BITS {
                force_truncate(usize_with_fld(int_cast_up(src), int_cast_up(fld), const { usize::BITS.wrapping_sub(FLD::BITS) }, self.start_bit()))
            } else {
                force_truncate(u128_with_fld(int_cast_up(src), int_cast_up(fld), const { u128::BITS.wrapping_sub(FLD::BITS) }, self.start_bit()))
            }
        }
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: PlainBitsCast, const LOW_START: u32, const LOW_LAST: u32, const HIGH_START: u32, const HIGH_LAST: u32>
    BitField2<STRUCT, FLD, LOW_START, LOW_LAST, HIGH_START, HIGH_LAST>
{
    /// Read bit field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[inline(always)]
    pub const fn const_read(&self) -> FLD {
        let () = Self::ASSERT;
        unsafe { from_bits(Field2Op::<LOW_START, LOW_LAST, HIGH_START, HIGH_LAST>::read_bits(into_bits(self.raw))) }
    }

    /// Gen new bit struct value from origin bit struct value with new field value
    /// Should be deprecated if `const_trait_impl` is stabled
    #[must_use]
    #[inline(always)]
    pub const fn const_with(&self, fld: impl IntoBitField<FLD>) -> STRUCT {
        let () = Self::ASSERT;
        unsafe { from_bits(Field2Op::<LOW_START, LOW_LAST, HIGH_START, HIGH_LAST>::with_fld(into_bits(self.raw), into_bit_field(fld))) }
    }
}

/// Value can be convert bit field value `Fld`, only has two implement:
/// * `impl<Fld: BitsCast> IntoBitField<Fld> for Fld`
/// * `impl<Fld: Uncheckable> IntoBitField<Unchecked<Fld>> for Fld where Unchecked<Fld>: BitsCast`
///
/// For examble:
/// ```ignore
/// #[bitfld(u2)]
/// enum Enum1 {
///     A = 0,
///     B = 1,
/// }
/// #[bitfld(u8)]
/// struct Struct1 {
///     #[bitfld(0)] a: bool,
///     #[bitfld(1)] b: Unchecked<Enum1>
/// }
/// Struct1(0).a().const_with(true); // field `a` must with a `bool` value
/// Struct1(0).b().const_with(Unchecked::from(Enum1::A)).b().const_with(Enum1::B); // field `b` can with a `Unchecked<Enum1>` value, or with a `Enum1` value
/// ```
///
pub trait IntoBitField<Fld: BitsCast + PlainBitsCast>: Copy {
    // fn into_bit_field(src: Self) -> <Fld::Bits as BasicInt>::Unsigned;
}

impl<Fld: BitsCast + PlainBitsCast> IntoBitField<Fld> for Fld {}

impl<Fld: Uncheckable<UncheckedRaw: BasicInt> + PlainUncheckable> IntoBitField<Unchecked<Fld>> for Fld {}

struct FieldOp<const START: u32, const LAST: u32>;

impl<const START: u32, const LAST: u32> FieldOp<START, LAST> {
    const END: u32 = LAST + 1;
    const BITS: u32 = Self::END - START;

    #[inline(always)]
    const unsafe fn read_bits<T: BasicUInt, Fld: BasicInt>(src: T) -> Fld {
        unsafe {
            if T::Primary::BITS <= usize::BITS {
                let (left_unused_bits, unused_bits) = const { (usize::BITS.wrapping_sub(Self::END), usize::BITS.wrapping_sub(Self::BITS)) };
                force_truncate(usize_read_bits(int_cast_up(src), left_unused_bits, unused_bits, Fld::SIGNED))
            } else {
                let (left_unused_bits, unused_bits) = const { (u128::BITS.wrapping_sub(Self::END), u128::BITS.wrapping_sub(Self::BITS)) };
                force_truncate(u128_read_bits(int_cast_up(src), left_unused_bits, unused_bits, Fld::SIGNED))
            }
        }
    }

    #[inline(always)]
    const unsafe fn with_fld<T: BasicUInt, Fld: BasicInt>(src: T, fld: Fld) -> T {
        unsafe {
            if T::Primary::BITS <= usize::BITS {
                force_truncate(usize_with_fld(int_cast_up(src), int_cast_up(fld), const { usize::BITS.wrapping_sub(Self::BITS) }, START))
            } else {
                force_truncate(u128_with_fld(int_cast_up(src), int_cast_up(fld), const { u128::BITS.wrapping_sub(Self::BITS) }, START))
            }
        }
    }
}

struct Field2Op<const LOW_START: u32, const LOW_LAST: u32, const HIGH_START: u32, const HIGH_LAST: u32>;

impl<const LOW_START: u32, const LOW_LAST: u32, const HIGH_START: u32, const HIGH_LAST: u32> Field2Op<LOW_START, LOW_LAST, HIGH_START, HIGH_LAST> {
    const LOW_END: u32 = LOW_LAST + 1;
    const LOW_BITS: u32 = Self::LOW_END - LOW_START;

    const HIGH_END: u32 = HIGH_LAST + 1;
    const HIGH_BITS: u32 = Self::HIGH_END - HIGH_START;

    #[inline(always)]
    const unsafe fn read_bits<T: BasicUInt, Fld: BasicInt>(src: T) -> Fld {
        unsafe {
            if T::Primary::BITS <= usize::BITS {
                let src = int_cast_up(src);
                let (left_unused_bits, unused_bits) = const { (usize::BITS.wrapping_sub(Self::LOW_END), usize::BITS.wrapping_sub(Self::LOW_BITS)) };
                let low = usize_read_bits(src, left_unused_bits, unused_bits, false);
                let (left_unused_bits, unused_bits) = const { (usize::BITS.wrapping_sub(Self::HIGH_END), usize::BITS.wrapping_sub(Self::HIGH_BITS)) };
                let high = usize_read_bits(src, left_unused_bits, unused_bits, Fld::SIGNED);
                force_truncate(high << Self::LOW_BITS | low)
            } else {
                let src = int_cast_up(src);
                let (left_unused_bits, unused_bits) = const { (u128::BITS.wrapping_sub(Self::LOW_END), u128::BITS.wrapping_sub(Self::LOW_BITS)) };
                let low = u128_read_bits(src, left_unused_bits, unused_bits, false);
                let (left_unused_bits, unused_bits) = const { (u128::BITS.wrapping_sub(Self::HIGH_END), u128::BITS.wrapping_sub(Self::HIGH_BITS)) };
                let high = u128_read_bits(src, left_unused_bits, unused_bits, Fld::SIGNED);
                force_truncate(high << Self::LOW_BITS | low)
            }
        }
    }

    #[inline(always)]
    const unsafe fn with_fld<T: BasicUInt, Fld: BasicInt>(src: T, fld: Fld) -> T {
        unsafe {
            if T::Primary::BITS <= usize::BITS {
                let fld = int_cast_up(fld);
                let low = usize_with_fld(int_cast_up(src), fld, const { usize::BITS.wrapping_sub(Self::LOW_BITS) }, LOW_START);
                force_truncate(usize_with_fld(low, fld >> Self::LOW_BITS, const { usize::BITS.wrapping_sub(Self::HIGH_BITS) }, HIGH_START))
            } else {
                let fld = int_cast_up(fld);
                let low = u128_with_fld(int_cast_up(src), fld, const { u128::BITS.wrapping_sub(Self::LOW_BITS) }, LOW_START);
                force_truncate(u128_with_fld(low, fld >> Self::LOW_BITS, const { u128::BITS.wrapping_sub(Self::HIGH_BITS) }, HIGH_START))
            }
        }
    }
}

#[inline(always)]
const fn usize_read_bits(src: usize, left_unused_bits: u32, unused_bits: u32, signed: bool) -> usize {
    let src = src << left_unused_bits;
    if signed { ((src as isize) >> unused_bits) as usize } else { src >> unused_bits }
}

#[inline(always)]
const fn u128_read_bits(src: u128, left_unused_bits: u32, unused_bits: u32, signed: bool) -> u128 {
    let src = src << left_unused_bits;
    if signed { ((src as i128) >> unused_bits) as u128 } else { src >> unused_bits }
}

#[inline(always)]
const unsafe fn int_cast_up<Src: BasicInt, Tgt: PrimaryUInt>(src: Src) -> Tgt {
    const { assert!(size_of::<Src>() <= size_of::<Tgt>()) }
    let mut v = Tgt::ZERO;
    unsafe { (&raw mut v as *mut Src).write(src) };
    v
}

#[inline(always)]
const fn usize_with_fld(src: usize, fld: usize, remain_bits: u32, start: u32) -> usize {
    let mask = usize::ALL_ONE >> remain_bits << start;
    src & !mask | fld << start & mask
}

#[inline(always)]
const fn u128_with_fld(src: u128, fld: u128, remain_bits: u32, start: u32) -> u128 {
    let mask = u128::ALL_ONE >> remain_bits << start;
    src & !mask | fld << start & mask
}

#[inline(always)]
const unsafe fn force_truncate<Src: Copy, Tgt>(src: Src) -> Tgt {
    const { assert!(size_of::<Src>() >= size_of::<Tgt>()) }
    let ptr = &raw const src as *const Tgt;
    unsafe { if align_of::<Src>() >= align_of::<Tgt>() { ptr.read() } else { ptr.read_unaligned() } }
}

#[inline(always)]
const unsafe fn from_bits<T: PlainBitsCast>(src: T::Bits) -> T {
    let () = T::ASSERT;
    unsafe { force_truncate(src) }
}

#[inline(always)]
const unsafe fn into_bits<T: PlainBitsCast>(src: T) -> T::Bits {
    let () = T::ASSERT;
    unsafe { force_truncate(src) }
}

#[inline(always)]
const unsafe fn into_bit_field<Fld: BitsCast + PlainBitsCast, T: IntoBitField<Fld>>(src: T) -> Fld::Bits {
    let () = Fld::ASSERT;
    unsafe { force_truncate(src) }
}
