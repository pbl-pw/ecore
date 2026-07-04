#![doc = include_str!("../README.md")]

use std::num::NonZero;

enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}
impl<Left: syn::parse::Parse, Right: syn::parse::Parse> syn::parse::Parse for Either<Left, Right> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(left) = input.parse::<Left>() { Ok(Self::Left(left)) } else { Ok(Self::Right(input.parse::<Right>()?)) }
    }
}

fn error<T>(span: proc_macro2::Span, message: impl std::fmt::Display) -> syn::Result<T> {
    Err(syn::Error::new(span, message))
}

/// ## Derive MapEnum for enum
///
/// **note: keep variant value same as define order (0..variants_count) will get optimized runtime performance**,
///
/// For tagged enum must add 'discriminant' or 'discriminant(attr_for_discriminant_enum)' attribute to define additional discriminant enum
/// ```ignore
/// #[derive(MapEnum)]
/// #[discriminant(derive(Clone, Copy))]
/// enum Enum1 {
///     Arm1(u8),
///     Arm2
/// }
/// ```
/// will generate discriminant enum additionally:
/// ```ignore
/// #[derive(Clone, Copy)]
/// enum Enum1Discriminant {
///     Arm1,
///     Arm2
/// }
/// ```
#[proc_macro_derive(MapEnum, attributes(discriminant))]
pub fn map_enum_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(map_enum::map_enum_derive(item))
}

/// Map enum to `EnumMap`, macro version for `EnumMap::map_new`, can move value, can invoke in const, requires import `EnumMap` and `MapEnum`
///
/// ```ignore
/// #[derive(MapEnum)]
/// enum Enum1 {
///     Var0,
///     Var1,
/// }
///
/// const ENUM1: EnumMap<Enum1, u8> = map_enum!(match Enum1 {
///     Var0 => 2,
///     Var1 => 5,
/// });
///
/// #[derive(MapEnum)]
/// enum Enum2 {
///     Kar0,
///     Kar1,
/// }
///
/// /// support map multi level enum using tuple,
/// const ENUM2: EnumMap<Enum1, EnumMap<Enum2, u8>> = map_enum!(match (Enum1, Enum2) {
///     (Var0, Kar0) => 2,
///     (Var0, Kar1) => 5,
///     (Var1, Kar0) => 8,
///     (Var1, Kar1) => 9,
/// });
/// ```
#[proc_macro]
pub fn map_enum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(map_enum::map_enum(item))
}

/// Derive `IterEnumDiscriminants` for enum, providing compile-time iteration over
/// all discriminant names and their associated integer values.
///
/// This generates a constant iterator that can be used with `ecore::EnumDiscriminantsIterator`
/// to iterate over all variants and their associated metadata at runtime.
#[proc_macro_derive(IterEnumDiscriminants)]
pub fn iter_enum_derive(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(iter_enum::iter_enum_derive(item))
}

/// ## Derive BitsCast (for enum only right now)
///
/// Inside usage but not recommanded:
/// ```ignore
/// #[derive(BitsCast, Clone, Copy)] // must derive `Clone` and `Copy` too
/// #[bitfld(u8)] // must has helper attribute
/// enum {...}
/// ```
/// Recommand usage:
/// ```ignore
/// #[bitfld(u8)] // attribute macro `bitfld` will auto expand to correct usage
/// enum {...}
/// ```
#[proc_macro_derive(BitsCast, attributes(bitfld))]
pub fn derive_bits_cast(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(bitfld::bitenum::derive_bits_cast(item))
}

/// ## Define bit field struct or bit field enum or bit field
///
/// ### Define bit enum
/// mark a enum can be used inside bit struct, if enum defined every value valid for repr type then impl `BitsCast`, otherwise impl `Uncheckable`
/// , derive [Clone] and [Copy] implicitly, default `#[repr(primary int of bits type)]` if not add `#[repr(..)]` attribute
/// ```ignore
/// #[bitfld(u3)] // u3 is bits type, actually expand to `#[derive(BitsCast, Clone, Copy)] #[bitfld(u3)] #[repr(u8)]`
/// #[derive(MapEnum)] // if requires `EnumMap<BitEnum1, ..>`
/// enum BitEnum1 {
///     A = 0,
///     B = 1
/// }
/// ```
/// For fieldness enum, must set `tag(ty, range)` and maybe set `payload(ty, range)` to define bits layout, `range` can be singlebit (like 2) or
///  bit range (like 1..3 or 1..=2) or start:bits (like 1:2). `range` is optional, if omited in `tag` then default is start from zero,
///  if omited in `payload` then default is start from tag end until total end. `tag` type `ty` must be int type but `payload` type `ty` must be unsigned int type
/// ```ignore
/// #[bitfld(u4, tag(u1, 0), payload(u3, 1..4))]
/// #[derive(MapEnum)] // if requires `EnumMap<BitEnum1, ..>`
/// enum BitEnum2 {
///     A(u2) = 0,
///     B(u3) = 1
/// }
/// ```
/// ### Define bit field struct, note:
/// * ****Every field in struct will be convert to a method to get bit field ref****
/// * because fields will be convert to method, `#[bitfld(...)]` maybe needs to add before other attributes if it requires real field
/// * underlying type must be unsigned basic int (u1, u2, ..., u128)
/// * bit field type can be any `T: BitsCast` type and `[T; N]` array and `EnumMap<Enum, T> where Enum: MapEnum`,
///   includes u1, u2, ..., u128, i1, i2, ..., i128, bool, any bit struct and any bit enum
/// * derive [Clone] and [Copy] and `bytemuck::Zeroable` and [`BitsCast`], derive `bytemuck::Pod` and `ConvertEndian` or `Uncheckable` only if avaiable
/// * default `#[repr(transparent)]` if not add `#[repr(..)]` attribute
/// ```ignore
/// #[bitfld(pub u32, relative)] // using #[bitfld(pub u32, .., overlay, ..)] will allow overlay for all fields, u32 is underlying type
/// struct BitStruct1 {
///     pub a: bitfld!(bool, 1), // single bit, same as 1..2 or 1..=1 or 1:1
///
///     pub b: bitfld!(BitEnum1, 2..=4), // same as 2..5 or 2:3
///
///     pub c: bitfld!(u5, 2..7, overlay), // same as 2..=6 or 2:5, reuqires `overlay` here because overlay field `b`
///
///     pub d: bitfld!(i3, :3), // occupy 3 bits, start bit use next bit of pre defined bit field, same as 7..10 or 7..=9 or 7:3 here
///
///     pub e: bitfld!(bool, ..11), // start bit use next bit of pre defined bit field, same as 10..11 or 10..=10 or 10:1 here
///
///     pub f: bitfld!([u2; 2], 11:4)  // same as 11..15 or 11..=14 here, array of any `BitsCast` type is allowed
///
///     pub g: bitfld!(EnumMap<BitEnum1, u4>, 15:4) // same as 15..19 or 15..=18 here, enum map of any `BitsCast` type is allowed
///
///     #[bitfld(19..=22)] // same as ..=22 or 19..23 or ..23 or 19:4 or :4, syntax same as bitfld!(..) without the first bit type
///     pub h: EnumMap<BitEnum1, u4>, // using attribute to define bit field is also ok
///
///     pub i: bitfld!(u9, ..), // from next bit of pre defined bit field, until end of underlying, same as 23..32 or 23..=31 or 23:9 here
/// }
/// ```
/// ### Define bit field
/// using attribute `#[bitfld(range, attributes..)]` or macro type `bitfld!(bittype, range, attributes..)` to define bit field
/// * absolute range `a..b` or `a..=b` or single bit `a`(= `a..a+1` or `a..=a`) or `a:bits`(= `a..a+bits`)
/// * relative range, start from next bit of pre defined bit field, `..b` or `..=b` or `:bits`, requires `relative` in bit struct's #[bitfld(..)]
/// * relative tail range, `a..` or `..`, occupy until underlying end, requires `relative` in bit struct's #[bitfld(..)]
#[proc_macro_attribute]
pub fn bitfld(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(bitfld::bitfld(attr, item))
}

/// Generate `RInt` type from value range, auto select `RRxx` value type
/// * full form `rint!(MIN..=MAX, default = DEFAULT, step = STEP, bits = BITS)`, `MIN` and `MAX` is required, others is optional
/// * range `MIN..=MAX` is same as `MIN..MAX+1``
/// * if `default` not setted, then is `MIN`
/// * if `step` not setted, then is 1, `step` must >= 1, and must ensure `MIN % STEP == 0 && MAX % STEP == 0 && DEFAULT % STEP == 0`
/// * if `bits` not setted, then is `((MAX - MIN) / STEP).bit_width()`
/// ```ignore
/// rint!(-10..=100) // RInt<u7, RRI8<-10, 100>>
/// rint!(-10..=100, default = 3) // RInt<u7, RRI8<-10, 100, 3>>
/// rint!(-10..=100, default = 20, step = 10) // RInt<u4, RRI8<-10, 100, 20, 10>>
/// rint!(3..50, default = 20) // RInt<u6, RRU8<3, 49, 20>>
/// rint!(3..50, default = 20, bits = 6) // RInt<u7, RRU8<3, 49, 20>>
/// ```
#[proc_macro]
pub fn rint(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    unwrap_stream(rint::rint(item))
}

fn unwrap_stream(stream: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    match stream {
        Ok(stream) => stream,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

/// must 1~128
fn valid_int_bits(bits: &[u8]) -> Option<NonZero<u8>> {
    let bits = match bits {
        [ss @ b'1'..=b'9'] => ss - b'0',
        [fs @ b'1'..=b'9', ss @ b'0'..=b'9'] => (fs - b'0') * 10 + (ss - b'0'),
        [b'1', fs @ b'0'..=b'1', ss @ b'0'..=b'9'] => 100 + (fs - b'0') * 10 + (ss - b'0'),
        [b'1', b'2', ss @ b'0'..=b'8'] => 120 + (ss - b'0'),
        _ => 0,
    };
    NonZero::new(bits)
}

/// u1~u128
fn valid_uint(repr: &syn::Ident) -> syn::Result<u32> {
    if let [b'u', bits @ ..] = repr.to_string().as_bytes()
        && let Some(bits) = valid_int_bits(bits)
    {
        Ok(bits.get() as u32)
    } else {
        error(repr.span(), "must be unsigned integer type")
    }
}

/// u1~u128 or i1~i128
fn valid_int(repr: &syn::Ident) -> syn::Result<(bool, u32)> {
    if let [sign @ b'u' | sign @ b'i', bits @ ..] = repr.to_string().as_bytes()
        && let Some(bits) = valid_int_bits(bits)
    {
        Ok((*sign == b'i', bits.get() as u32))
    } else {
        error(repr.span(), "must be integer type")
    }
}

mod bitfld;
mod iter_enum;
mod map_enum;
mod rint;
