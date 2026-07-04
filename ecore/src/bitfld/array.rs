use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    Array, EnumMap, MapEnum, PlainMapEnum,
    bitfld::{BitField, BitsCast, DynBitField, IntoBitField, PlainBitsCast},
    int::{BasicInt, BasicUInt, CInt},
};

#[repr(transparent)]
pub struct BitFieldElement<STRUCT, FLD, const START: u32, const INDEX: usize> {
    raw: STRUCT,
    mark: PhantomData<FLD>,
}

impl<const START: u32, const INDEX: usize, STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast> BitFieldElement<STRUCT, FLD, START, INDEX> {
    fn raw(&self) -> <STRUCT::Bits as BasicInt>::Primary {
        STRUCT::into_underlying(self.raw)
    }

    /// Reads the field value from the bit-struct at the given array index.
    pub fn read(&self) -> FLD {
        FLD::from_underlying(self.raw() >> AryOp::<START, INDEX, STRUCT::Bits, FLD>::BEGIN)
    }

    /// Generates a new bit-struct value with the given field value at this array index.
    #[must_use]
    pub fn with(&self, fld: impl IntoBitField<FLD>) -> STRUCT {
        STRUCT::from_underlying(AryOp::<START, INDEX, STRUCT::Bits, FLD>::with_bits(self.raw(), IntoBitField::into_bit_field(fld)))
    }

    /// Returns a dynamic bit-field reference to this array element.
    pub const fn as_ref(&self) -> &DynBitField<FLD, STRUCT> {
        DynBitField::new(&self.raw, AryOp::<START, INDEX, STRUCT::Bits, FLD>::BEGIN)
    }

    /// Returns a mutable dynamic bit-field reference to this array element.
    pub const fn as_mut(&mut self) -> &mut DynBitField<FLD, STRUCT> {
        DynBitField::new_mut(&mut self.raw, AryOp::<START, INDEX, STRUCT::Bits, FLD>::BEGIN)
    }

    /// Writes the field value at this array index.
    pub fn write(&mut self, fld: FLD) {
        self.raw = self.with(fld);
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> BitField<STRUCT, [FLD; N], START, LAST> {
    pub const fn cget<const INDEX: usize>(&self) -> &BitFieldElement<STRUCT, FLD, START, INDEX> {
        const { assert!(INDEX < N) }
        unsafe { core::mem::transmute(self) }
    }

    pub const fn get(&self, index: usize) -> Option<&DynBitField<FLD, STRUCT>> {
        if index < N { Some(unsafe { self.get_unchecked(index) }) } else { None }
    }

    pub const fn get_mut(&mut self, index: usize) -> Option<&mut DynBitField<FLD, STRUCT>> {
        if index < N { Some(unsafe { self.get_unchecked_mut(index) }) } else { None }
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked(&self, index: usize) -> &DynBitField<FLD, STRUCT> {
        DynBitField::new(&self.raw, START + index as u32 * FLD::BITS)
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut DynBitField<FLD, STRUCT> {
        DynBitField::new_mut(&mut self.raw, START + index as u32 * FLD::BITS)
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> Index<usize>
    for BitField<STRUCT, [FLD; N], START, LAST>
{
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> IndexMut<usize>
    for BitField<STRUCT, [FLD; N], START, LAST>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> DynBitField<[FLD; N], STRUCT> {
    pub const fn get(&self, index: usize) -> Option<&DynBitField<FLD, STRUCT>> {
        if index < N { Some(unsafe { self.get_unchecked(index) }) } else { None }
    }

    pub const fn get_mut(&mut self, index: usize) -> Option<&mut DynBitField<FLD, STRUCT>> {
        if index < N { Some(unsafe { self.get_unchecked_mut(index) }) } else { None }
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked(&self, index: usize) -> &DynBitField<FLD, STRUCT> {
        DynBitField::new(&self.underlying, self.start_bit() + index as u32 * FLD::BITS)
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut DynBitField<FLD, STRUCT> {
        let start_bit = self.start_bit() + index as u32 * FLD::BITS;
        DynBitField::new_mut(&mut self.underlying, start_bit)
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> Index<usize> for DynBitField<[FLD; N], STRUCT> {
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, FLD: BitsCast, const N: usize> IndexMut<usize> for DynBitField<[FLD; N], STRUCT> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<FLD: BitsCast, const N: usize> DynBitField<[FLD; N]> {
    pub const fn get(&self, index: usize) -> Option<&DynBitField<FLD>> {
        if index < N { Some(unsafe { self.get_unchecked(index) }) } else { None }
    }

    pub const fn get_mut(&mut self, index: usize) -> Option<&mut DynBitField<FLD>> {
        if index < N { Some(unsafe { self.get_unchecked_mut(index) }) } else { None }
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked(&self, index: usize) -> &DynBitField<FLD> {
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + index as u32 * FLD::BITS) as u8);
        unsafe { &*DynBitField::new_ptr((&raw const self.underlying).cast(), meta) }
    }

    /// # Safety
    ///
    /// The caller must ensure `index < N`.
    pub const unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut DynBitField<FLD> {
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + index as u32 * FLD::BITS) as u8);
        unsafe { &mut *DynBitField::new_mut_ptr((&raw mut self.underlying).cast(), meta) }
    }
}

impl<FLD: BitsCast, const N: usize> Index<usize> for DynBitField<[FLD; N]> {
    type Output = DynBitField<FLD>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<FLD: BitsCast, const N: usize> IndexMut<usize> for DynBitField<[FLD; N]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + PlainMapEnum, FLD: BitsCast>
    BitField<STRUCT, EnumMap<Ids, FLD>, START, LAST>
{
    pub const fn get(&self, ids: &Ids) -> &DynBitField<FLD, STRUCT> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        DynBitField::new(&self.raw, START + CInt::cast_as::<_, u32>(ids) * FLD::BITS)
    }

    pub const fn get_mut(&mut self, ids: &Ids) -> &mut DynBitField<FLD, STRUCT> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        DynBitField::new_mut(&mut self.raw, START + CInt::cast_as::<_, u32>(ids) * FLD::BITS)
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> Index<Ids>
    for BitField<STRUCT, EnumMap<Ids, FLD>, START, LAST>
{
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: Ids) -> &Self::Output {
        &self[&index]
    }
}

impl<const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> IndexMut<Ids>
    for BitField<STRUCT, EnumMap<Ids, FLD>, START, LAST>
{
    fn index_mut(&mut self, index: Ids) -> &mut Self::Output {
        &mut self[&index]
    }
}

impl<'id, const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast>
    Index<&'id Ids> for BitField<STRUCT, EnumMap<Ids, FLD>, START, LAST>
{
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: &'id Ids) -> &Self::Output {
        DynBitField::new(&self.raw, START + index.get_index() as u32 * FLD::BITS)
    }
}

impl<'id, const START: u32, const LAST: u32, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast>
    IndexMut<&'id Ids> for BitField<STRUCT, EnumMap<Ids, FLD>, START, LAST>
{
    fn index_mut(&mut self, index: &'id Ids) -> &mut Self::Output {
        DynBitField::new_mut(&mut self.raw, START + index.get_index() as u32 * FLD::BITS)
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + PlainMapEnum, FLD: BitsCast> DynBitField<EnumMap<Ids, FLD>, STRUCT> {
    pub const fn get(&self, ids: &Ids) -> &DynBitField<FLD, STRUCT> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        DynBitField::new(&self.underlying, self.start_bit() + CInt::cast_as::<_, u32>(ids) * FLD::BITS)
    }

    pub const fn get_mut(&mut self, ids: &Ids) -> &mut DynBitField<FLD, STRUCT> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        let start_bit = self.start_bit() + CInt::cast_as::<_, u32>(ids) * FLD::BITS;
        DynBitField::new_mut(&mut self.underlying, start_bit)
    }
}

impl<'id, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast> Index<&'id Ids>
    for DynBitField<EnumMap<Ids, FLD>, STRUCT>
{
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: &'id Ids) -> &Self::Output {
        DynBitField::new(&self.underlying, self.start_bit() + index.get_index() as u32 * FLD::BITS)
    }
}

impl<'id, STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast> IndexMut<&'id Ids>
    for DynBitField<EnumMap<Ids, FLD>, STRUCT>
{
    fn index_mut(&mut self, index: &'id Ids) -> &mut Self::Output {
        let start_bit = self.start_bit() + index.get_index() as u32 * FLD::BITS;
        DynBitField::new_mut(&mut self.underlying, start_bit)
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> Index<Ids> for DynBitField<EnumMap<Ids, FLD>, STRUCT> {
    type Output = DynBitField<FLD, STRUCT>;

    fn index(&self, index: Ids) -> &Self::Output {
        &self[&index]
    }
}

impl<STRUCT: PlainBitsCast<Bits: BasicUInt>, Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> IndexMut<Ids>
    for DynBitField<EnumMap<Ids, FLD>, STRUCT>
{
    fn index_mut(&mut self, index: Ids) -> &mut Self::Output {
        &mut self[&index]
    }
}

impl<Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + PlainMapEnum, FLD: BitsCast> DynBitField<EnumMap<Ids, FLD>> {
    pub const fn get(&self, ids: &Ids) -> &DynBitField<FLD> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + CInt::cast_as::<_, u32>(ids) * FLD::BITS) as u8);
        unsafe { &*DynBitField::new_ptr((&raw const self.underlying).cast(), meta) }
    }

    pub const fn get_mut(&mut self, ids: &Ids) -> &mut DynBitField<FLD> {
        let ids: <Ids as PlainMapEnum>::NatureInt = unsafe { core::mem::transmute_copy(ids) };
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + CInt::cast_as::<_, u32>(ids) * FLD::BITS) as u8);
        unsafe { &mut *DynBitField::new_mut_ptr((&raw mut self.underlying).cast(), meta) }
    }
}

impl<'id, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast> Index<&'id Ids> for DynBitField<EnumMap<Ids, FLD>> {
    type Output = DynBitField<FLD>;

    fn index(&self, index: &'id Ids) -> &Self::Output {
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + index.get_index() as u32 * FLD::BITS) as u8);
        unsafe { &*DynBitField::new_ptr((&raw const self.underlying).cast(), meta) }
    }
}

impl<'id, Ids: MapEnum<Array: Array<Map<FLD>: Copy>> + 'id, FLD: BitsCast> IndexMut<&'id Ids> for DynBitField<EnumMap<Ids, FLD>> {
    fn index_mut(&mut self, index: &'id Ids) -> &mut Self::Output {
        let meta = self.meta();
        let meta = meta.start_bit().const_with((meta.start_bit().const_read() as u32 + index.get_index() as u32 * FLD::BITS) as u8);
        unsafe { &mut *DynBitField::new_mut_ptr((&raw mut self.underlying).cast(), meta) }
    }
}

impl<Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> Index<Ids> for DynBitField<EnumMap<Ids, FLD>> {
    type Output = DynBitField<FLD>;

    fn index(&self, index: Ids) -> &Self::Output {
        &self[&index]
    }
}

impl<Ids: MapEnum<Array: Array<Map<FLD>: Copy>>, FLD: BitsCast> IndexMut<Ids> for DynBitField<EnumMap<Ids, FLD>> {
    fn index_mut(&mut self, index: Ids) -> &mut Self::Output {
        &mut self[&index]
    }
}

struct AryOp<const BITS_BEGIN: u32, const ID: usize, Under, FLD>(PhantomData<(Under, FLD)>);

impl<const BITS_BEGIN: u32, const ID: usize, Under: BasicUInt, FLD: BitsCast> AryOp<BITS_BEGIN, ID, Under, FLD> {
    const BEGIN: u32 = BITS_BEGIN + ID as u32 * FLD::BITS;
    const REMAIN_BITS: u32 = Under::Primary::BITS - FLD::BITS;
    fn with_bits(v: Under::Primary, bv: Under::Primary) -> Under::Primary {
        let mask = Under::Primary::ALL_ONE >> Self::REMAIN_BITS << Self::BEGIN;
        v & !mask | (bv << Self::BEGIN) & mask
    }
}
