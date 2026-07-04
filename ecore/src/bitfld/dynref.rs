use core::{hint::unreachable_unchecked, marker::PhantomData};

use crate::{
    bitfld::{prelude::*, sealed::IntoBitField},
    bitint::u3,
    int::{BasicUInt, BitsOp},
};

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const START: u32, const LAST: u32> BitField<STRUCT, FLD, START, LAST> {
    const META: BitFldMeta = BitFldMeta(0)
        .start_bit()
        .const_with(Self::START as u8)
        .bits()
        .const_with(Self::BITS as u8)
        .bytes_log2()
        .const_with(u3::new(size_of::<STRUCT>().ilog2() as u8).unwrap())
        .aligned()
        .const_with(align_of::<STRUCT>() >= size_of::<STRUCT>());

    pub const fn as_ref(&self) -> &DynBitField<FLD, STRUCT> {
        DynBitField::new(&self.raw, START)
    }

    pub const fn as_mut(&mut self) -> &mut DynBitField<FLD, STRUCT> {
        DynBitField::new_mut(&mut self.raw, START)
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const START: u32, const LAST: u32> AsRef<DynBitField<FLD, STRUCT>>
    for BitField<STRUCT, FLD, START, LAST>
{
    #[inline(always)]
    fn as_ref(&self) -> &DynBitField<FLD, STRUCT> {
        self.as_ref()
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const START: u32, const LAST: u32> AsMut<DynBitField<FLD, STRUCT>>
    for BitField<STRUCT, FLD, START, LAST>
{
    #[inline(always)]
    fn as_mut(&mut self) -> &mut DynBitField<FLD, STRUCT> {
        self.as_mut()
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const START: u32, const LAST: u32> AsRef<DynBitField<FLD>> for BitField<STRUCT, FLD, START, LAST> {
    #[inline(always)]
    fn as_ref(&self) -> &DynBitField<FLD> {
        unsafe { &*DynBitField::new_ptr(core::ptr::from_ref(self).cast::<()>(), Self::META) }
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const START: u32, const LAST: u32> AsMut<DynBitField<FLD>> for BitField<STRUCT, FLD, START, LAST> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut DynBitField<FLD> {
        unsafe { &mut *DynBitField::new_mut_ptr(core::ptr::from_mut(self).cast::<()>(), Self::META) }
    }
}

/// [BitField] in dyn bit struct
/// * [`DynBitField<FLD>`] support any bit struct, can not pass miri test
/// * [`DynBitField<FLD, STRUCT>`] support only `STRUCT` bit struct, can pass miri test
pub struct DynBitField<FLD, STRUCT = DynBitStruct> {
    underlying: STRUCT,
    mark: PhantomData<FLD>,
    tail: [()],
}

impl<FLD, STRUCT: PlainBitsCast<Bits: BasicUInt>> DynBitField<FLD, STRUCT> {
    const fn new(obj: &STRUCT, start_bit: u32) -> &Self {
        unsafe { &*(core::ptr::slice_from_raw_parts(core::ptr::from_ref(obj).cast::<()>(), start_bit as usize) as *const Self) }
    }

    const fn new_mut(obj: &mut STRUCT, start_bit: u32) -> &mut Self {
        unsafe { &mut *(core::ptr::slice_from_raw_parts_mut(core::ptr::from_mut(obj).cast::<()>(), start_bit as usize) as *mut Self) }
    }

    pub(super) const fn start_bit(&self) -> u32 {
        self.tail.len() as u32
    }

    pub(super) const fn underlying(&self) -> &STRUCT {
        &self.underlying
    }
}

impl<FLD: BitsCast, STRUCT: PlainBitsCast<Bits: BasicUInt>> DynBitField<FLD, STRUCT> {
    const META: BitFldMeta = BitFldMeta(0)
        .bits()
        .const_with(FLD::BITS as u8)
        .bytes_log2()
        .const_with(u3::new(size_of::<STRUCT>().ilog2() as u8).unwrap())
        .aligned()
        .const_with(align_of::<STRUCT>() >= size_of::<STRUCT>());

    pub fn read(&self) -> FLD {
        FLD::from_underlying(self.raw().unbounded_shr(self.start_bit()))
    }

    pub fn with(&self, fld: impl IntoBitField<FLD>) -> STRUCT {
        let start_bit = self.start_bit();
        STRUCT::from_underlying(BitsOp::with_bits_i(&self.raw(), start_bit..start_bit + FLD::BITS, IntoBitField::into_bit_field(fld)))
    }

    pub fn write(&mut self, fld: impl IntoBitField<FLD>) {
        self.underlying = self.with(fld);
    }

    pub const fn as_ref(&self) -> &DynBitField<FLD> {
        unsafe { &*DynBitField::new_ptr((&raw const self.underlying).cast(), Self::META.start_bit().const_with(self.start_bit() as u8)) }
    }

    pub const fn as_mut(&mut self) -> &mut DynBitField<FLD> {
        unsafe { &mut *DynBitField::new_mut_ptr((&raw mut self.underlying).cast(), Self::META.start_bit().const_with(self.start_bit() as u8)) }
    }

    fn raw(&self) -> <STRUCT::Bits as BasicInt>::Primary {
        STRUCT::into_underlying(self.underlying)
    }
}

pub struct DynBitStruct {}

impl<FLD: BitsCast> DynBitField<FLD, DynBitStruct> {
    /// Reads the field value from a type-erased bit-struct.
    pub fn read(&self) -> FLD {
        FLD::from_underlying(self.meta().read_underlying(core::ptr::from_ref(self).cast()))
    }

    /// Writes the field value into a type-erased bit-struct.
    pub fn write(&mut self, v: FLD) {
        self.meta().write_underlying(core::ptr::from_mut(self).cast(), FLD::into_underlying(v));
    }
}

impl<FLD: BitsCast + PlainBitsCast> DynBitField<FLD, DynBitStruct> {
    pub const fn const_read(&self) -> FLD {
        let () = FLD::ASSERT;
        let underlying = self.meta().read_underlying(core::ptr::from_ref(self).cast());
        if <FLD::Bits as BasicInt>::Primary::BITS <= usize::BITS {
            let underlying = underlying as usize;
            let unused_bits = const { usize::BITS.wrapping_sub(FLD::BITS) };
            let left_unused_bits = unused_bits - self.meta().start_bit().const_read() as u32;
            let underlying = if FLD::Bits::SIGNED {
                ((underlying as isize) << left_unused_bits >> unused_bits) as usize
            } else {
                underlying << left_unused_bits >> unused_bits
            };
            unsafe { core::mem::transmute_copy(&underlying) }
        } else {
            let unused_bits = const { u128::BITS - FLD::BITS };
            let left_unused_bits = unused_bits - self.meta().start_bit().const_read() as u32;
            let underlying = if FLD::Bits::SIGNED {
                ((underlying as i128) << left_unused_bits >> unused_bits) as u128
            } else {
                underlying << left_unused_bits >> unused_bits
            };
            unsafe { core::mem::transmute_copy(&underlying) }
        }
    }

    pub const fn const_write(&mut self, fld: FLD) {
        let () = FLD::ASSERT;
        let mut buf = 0u128;
        unsafe { (&raw mut buf).cast::<FLD>().write(fld) };
        self.meta().write_underlying(core::ptr::from_mut(self).cast(), buf);
    }
}

impl<FLD> DynBitField<FLD, DynBitStruct> {
    const unsafe fn new_ptr(ptr: *const (), meta: BitFldMeta) -> *const Self {
        core::ptr::slice_from_raw_parts(ptr, meta.0 as usize) as *const Self
    }

    const unsafe fn new_mut_ptr(ptr: *mut (), meta: BitFldMeta) -> *mut Self {
        core::ptr::slice_from_raw_parts_mut(ptr, meta.0 as usize) as *mut Self
    }

    const fn meta(&self) -> BitFldMeta {
        #[cfg(target_pointer_width = "32")]
        type Under = u32;
        #[cfg(target_pointer_width = "64")]
        type Under = u64;
        BitFldMeta(self.tail.len() as Under)
    }
}

#[cfg_attr(target_pointer_width = "32", bitfld(u32, relative))]
#[cfg_attr(target_pointer_width = "64", bitfld(u64, relative))]
struct BitFldMeta {
    start_bit: bitfld!(u8, :8),

    bits: bitfld!(u8, :8),

    /// Bit struct bytes
    bytes_log2: bitfld!(u3, :3),

    /// Bit struct is aligned
    aligned: bitfld!(bool, :1),
}

impl BitFldMeta {
    const fn read_underlying(self, ptr: *const ()) -> u128 {
        let mut out = 0u128;
        let buf = &raw mut out;
        let start_bit = self.start_bit().const_read() as u32;
        let aligned = self.aligned().const_read();
        macro_rules! rdv {
            ($type:ty) => {{
                let ptr = ptr.cast::<$type>();
                let v = unsafe { if aligned { ptr.read() } else { ptr.read_unaligned() } };
                unsafe { buf.cast::<$type>().write(v >> start_bit) };
            }};
        }
        match self.bytes_log2().const_read().value() {
            0 => rdv!(u8),
            1 => rdv!(u16),
            2 => rdv!(u32),
            3 => rdv!(u64),
            4 => rdv!(u128),
            _ => unsafe { unreachable_unchecked() },
        }
        out
    }

    const fn write_underlying(self, obj: *mut (), mut buf: u128) {
        let buf = &raw mut buf;
        let start_bit = self.start_bit().const_read() as u32;
        let bits = self.bits().const_read() as u32;
        let aligned = self.aligned().const_read();
        macro_rules! wrv {
            ($type:ty) => {{
                let mask = <$type>::MAX >> <$type>::BITS - bits << start_bit;
                let obj = obj.cast::<$type>();
                unsafe {
                    let v = if aligned { obj.read() } else { obj.read_unaligned() };
                    let v = v & !mask | buf.cast::<$type>().read() << start_bit & mask;
                    if aligned { obj.write(v) } else { obj.write_unaligned(v) }
                };
            }};
        }
        match self.bytes_log2().const_read().value() {
            0 => wrv!(u8),
            1 => wrv!(u16),
            2 => wrv!(u32),
            3 => wrv!(u64),
            4 => wrv!(u128),
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[path = "array.rs"]
mod array;
