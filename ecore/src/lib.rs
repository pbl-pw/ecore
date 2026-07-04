#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

pub use ecore_macro::{IterEnumDiscriminants, MapEnum, map_enum};

pub use self::{
    ary_or_slc::{Array, ArrayOrSlice, MaybeUninitArrayOrSlice},
    map_enum::{EnumDiscriminantsIterator, EnumDiscriminantsIteratorExt, EnumMap, IterEnumDiscriminants, MapEnum, PlainMapEnum},
};

pub mod int;
pub mod nbool;
pub mod range;
pub mod repr;

#[cfg(feature = "bitfld")]
pub mod bitfld;

#[cfg(feature = "bitint")]
pub mod bitint;

mod ary_or_slc;
mod map_enum;
