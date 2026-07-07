use core::marker::PhantomData;

use crate::{
    Array, EnumMap, MapEnum,
    int::{BasicInt, PrimaryInt},
};

/// Extension of [`MapEnum`] providing compile-time iteration of enum discriminants
/// with associated name and value pairs.
pub trait IterEnumDiscriminants: MapEnum {
    type Value: PrimaryInt;
    type Discriminants: EnumDiscriminantsIterator<Value = Self::Value> + 'static;
    const DISCRIMINANTS: &Self::Discriminants;
}

/// Iterator over the names and numeric values of an enum's discriminants.
/// Implemented by compile-time generated constant arrays.
#[allow(clippy::len_without_is_empty)]
pub trait EnumDiscriminantsIterator {
    type Value;
    /// Returns the total number of discriminant entries.
    fn len(&self) -> usize;
    /// Returns the name and value at the given index.
    ///
    /// # Safety
    ///
    /// The caller must ensure `index < self.len()`.
    unsafe fn get_unchecked(&self, index: usize) -> (&str, Self::Value);
}

/// Extension trait providing a safe `get` method on [`EnumDiscriminantsIterator`].
pub trait EnumDiscriminantsIteratorExt: EnumDiscriminantsIterator {
    fn get(&self, index: usize) -> Option<(&str, Self::Value)> {
        if index < self.len() { Some(unsafe { self.get_unchecked(index) }) } else { None }
    }
}
impl<E: ?Sized + EnumDiscriminantsIterator> EnumDiscriminantsIteratorExt for E {}

impl<Ids: MapEnum, Name: AsRef<str>, Value: PrimaryInt> EnumDiscriminantsIterator for (EnumMap<Ids, Name>, EnumMap<Ids, Value>) {
    type Value = Value;

    fn len(&self) -> usize {
        Ids::Array::LEN
    }

    unsafe fn get_unchecked(&self, index: usize) -> (&str, Self::Value) {
        unsafe { (self.0.as_array().as_ref().get_unchecked(index).as_ref(), *self.1.as_array().as_ref().get_unchecked(index)) }
    }
}

impl<Ids: MapEnum, Name: AsRef<str>, Value: PrimaryInt> EnumDiscriminantsIterator for (EnumMap<Ids, Name>, PhantomData<EnumMap<Ids, Value>>) {
    type Value = Value;

    fn len(&self) -> usize {
        Ids::Array::LEN
    }

    unsafe fn get_unchecked(&self, index: usize) -> (&str, Self::Value) {
        unsafe { (self.0.as_array().as_ref().get_unchecked(index).as_ref(), index.cast_as()) }
    }
}

#[cfg(feature = "bitfld")]
impl<Enum: MapEnum + crate::bitfld::ReLayoutBitsEnum, Bits: crate::int::BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32> MapEnum
    for crate::bitfld::BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    type Discriminant = Enum::Discriminant;
    type Array = Enum::Array;

    fn iter() -> impl ExactSizeIterator<Item = Self::Discriminant> {
        Enum::iter()
    }

    fn get_index(&self) -> usize {
        self.get().get_index()
    }
}

#[cfg(feature = "bitfld")]
impl<Enum: IterEnumDiscriminants + crate::bitfld::ReLayoutBitsEnum, Bits: crate::int::BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32>
    IterEnumDiscriminants for crate::bitfld::BitsEnumReLayout<Enum, Bits, TAG_START, PAYLOAD_START>
{
    type Value = Enum::Value;
    type Discriminants = Enum::Discriminants;
    const DISCRIMINANTS: &Self::Discriminants = Enum::DISCRIMINANTS;
}
