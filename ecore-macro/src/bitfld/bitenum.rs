use std::{
    fmt::Display,
    ops::{Neg, Shr},
    str::FromStr,
};

use proc_macro2::Literal;
use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;

use crate::{bitfld::const_trait, error, valid_int};

pub fn bitfld(
    attr: proc_macro::TokenStream,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    body: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let EnumRepr { repr, split_repr } = syn::parse::Parser::parse(
        |input: syn::parse::ParseStream| {
            // if has `:`, means maybe has been processed by `bitfld`, not do it again for prevent recurse loop
            let Err(_) = input.parse::<syn::Token![:]>() else { return error(input.span(), "Invalid repr, maybe need import `BitsCast` derive macro") };
            input.parse()
        },
        attr.clone(),
    )?;
    let primary_repr = if !attrs.iter().any(|attr| attr.path().is_ident("repr")) {
        let (_, bits) = valid_int(&repr)?;
        let signed = if let Some((FldRepr { repr, .. }, _)) = &split_repr { valid_int(repr)?.0 } else { false };
        let primary_bits = bits.next_multiple_of(8).next_power_of_two();
        let primary_repr = syn::Ident::new(&format!("{}{}", if signed { 'i' } else { 'u' }, primary_bits), repr.span());
        Some(quote!(#[repr(#primary_repr)]))
    } else {
        None
    };
    let attr: proc_macro2::TokenStream = attr.into();
    //  #[bitfld(ty)] to `#[bitfld(:ty)]` prevent recurse loop
    Ok(quote!(#[derive(BitsCast, Clone, Copy)] #[bitfld(:#attr)] #primary_repr #(#attrs)* #vis enum #body))
}

pub fn derive_bits_cast(item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let enum_def: syn::ItemEnum = syn::parse(item)?;
    let true = enum_def.generics.params.is_empty() else { return error(enum_def.generics.span(), "bitfld enum not support generic right now") };
    let Some(repr) = enum_def.attrs.iter().find(|attr| attr.path().is_ident("bitfld")) else { return error(enum_def.span(), "#[bitfld(..)] not founded") };
    let EnumRepr { repr, split_repr } = repr.parse_args()?;
    if let Some((tag_repr, payload_repr)) = split_repr {
        return fieldness_derive(&enum_def, repr, tag_repr, payload_repr);
    }
    let (signed, bits) = valid_int(&repr)?;
    let is_exhaustive = bits < usize::BITS && 1 << bits == enum_def.variants.len();
    let primary_bits = bits.next_multiple_of(8).next_power_of_two();
    let is_primary = bits == primary_bits;
    let remap = {
        let mut remap = Vec::new();
        remap.reserve_exact(enum_def.variants.len());
        let checker: &mut dyn CheckValue = if bits <= usize::BITS {
            if signed { &mut ValueChecker::<isize>::new(bits) } else { &mut ValueChecker::<usize>::new(bits) }
        } else {
            if signed { &mut ValueChecker::<i128>::new(bits) } else { &mut ValueChecker::<u128>::new(bits) }
        };
        for variant in enum_def.variants.iter() {
            let syn::Fields::Unit = variant.fields else {
                return error(repr.span(), "non unit-only enum must use #[bitfld(inttype, tag(...), payload(...)] to set underlying");
            };
            let expr = checker.add_variant(variant)?;
            let name = &variant.ident;
            remap.push(if is_exhaustive { quote!(#expr => Self::#name) } else { quote!(#expr => Ok(Self::#name)) });
        }
        remap
    };
    let name = &enum_def.ident;
    let primary_repr = if is_primary { repr.clone() } else { syn::Ident::new(&format!("{}{}", if signed { 'i' } else { 'u' }, primary_bits), repr.span()) };
    let getter = if is_primary { quote!(self as #repr) } else { quote!(unsafe { #repr::new_unchecked(self as #primary_repr) }) };
    let raw = if is_primary { quote!(raw) } else { quote!(raw.value()) };
    let fallback =
        if is_exhaustive { if is_primary { quote!() } else { quote!(_=>unsafe{core::hint::unreachable_unchecked()}) } } else { quote!(_=>Err(#raw)) };
    let result = if is_exhaustive { quote!(Self) } else { quote!(Result<Self,#primary_repr>) };
    let mut out = quote!(
        impl #name {
            pub const fn raw_value(self) -> #repr {
                #getter
            }
            pub const fn new_with_raw_value(raw: #repr) -> #result {
                match #raw {
                    #(#remap ,)*
                    #fallback
                }
            }
        }
    );
    let const_trait = const_trait();
    if is_exhaustive {
        quote!(
            impl #const_trait BitsCast for #name {
                const BITS: u32 = #bits;
                fn from_underlying<Bits: PrimaryInt>(v: Bits) -> Self {
                    Self::new_with_raw_value(v.cast_as())
                }
                fn into_underlying<Bits: PrimaryInt>(sf: Self) -> Bits {
                    sf.raw_value().cast_as()
                }
            }
        )
        .to_tokens(&mut out);
        if let Some(attr) = enum_def.attrs.iter().find(|attr| attr.path().is_ident("repr"))
            && let Ok(attr) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
            && attr.iter().find(|repr| repr.path().get_ident().is_some_and(|ident| ident == &primary_repr)).is_some()
        {
            quote!(unsafe impl #const_trait PlainBitsCast for #name { type Bits = #repr; }).to_tokens(&mut out);
        }
        if !is_primary {
            quote!(
                impl #const_trait Uncheckable for #name {
                    type UncheckedRaw = #primary_repr;
                    fn raw_value(sf:Self) -> Self::UncheckedRaw {
                        sf.raw_value().value()
                    }
                    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
                        let Some(v)= #repr::new(raw) else{return Err(raw)};
                        Ok(Self::new_with_raw_value(v))
                    }
                }
            )
            .to_tokens(&mut out);
        }
    } else {
        quote!(
            impl #const_trait Uncheckable for #name {
                type UncheckedRaw = #repr;
                fn raw_value(sf:Self) -> Self::UncheckedRaw {
                    sf.raw_value()
                }
                fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
                    let Ok(v)=Self::new_with_raw_value(raw) else{return Err(raw)};
                    Ok(v)
                }
            }
        )
        .to_tokens(&mut out);
    };
    if !is_exhaustive || !is_primary {
        quote!(unsafe impl #const_trait PlainUncheckable for #name {}).to_tokens(&mut out);
    }
    Ok(out)
}

fn fieldness_derive(enum_def: &syn::ItemEnum, total_repr: syn::Ident, tag_repr: FldRepr, payload_repr: FldRepr) -> syn::Result<proc_macro2::TokenStream> {
    let (false, total_bits) = valid_int(&total_repr)? else { return error(total_repr.span(), "non unit-only enum only support unsigned int") };
    let (signed, tag_start, tag_bits) = tag_repr.get_range(total_bits, 0)?;
    let tag_end = tag_start + tag_bits;
    let (payload_repr, payload_start, payload_bits) = {
        let (false, payload_start, payload_bits) = payload_repr.get_range(total_bits, tag_end)? else {
            return error(payload_repr.repr.span(), "only support unsigned int");
        };
        let true = (payload_start >= tag_end || tag_start >= payload_start + payload_bits) else {
            return error(payload_repr.repr.span(), "payload must not overlay tag");
        };
        (payload_repr.repr, payload_start, payload_bits)
    };
    let primary_bits = total_bits.next_multiple_of(8).next_power_of_two();
    let primary_repr = syn::Ident::new(&format!("u{}", primary_bits), total_repr.span());
    let sprimary_repr = syn::Ident::new(&format!("{}{}", if signed { 'i' } else { 'u' }, primary_bits), total_repr.span());
    let tag_is_primary = tag_bits.next_multiple_of(8).next_power_of_two() == tag_bits;

    let tag_repr = tag_repr.repr;
    let enum_ident = &enum_def.ident;

    let mut getter = Vec::new();
    let mut cgetter = Vec::new();
    let mut setter = Vec::new();
    let mut csetter = Vec::new();
    let mut discri = Vec::new();
    let mut asserter = Vec::new();
    let mut unit_only = true;
    getter.reserve_exact(enum_def.variants.len());
    cgetter.reserve_exact(enum_def.variants.len());
    setter.reserve_exact(enum_def.variants.len());
    csetter.reserve_exact(enum_def.variants.len());
    discri.reserve_exact(enum_def.variants.len());
    let checker: &mut dyn CheckValue = if tag_bits <= usize::BITS {
        if signed { &mut ValueChecker::<isize>::new(tag_bits) } else { &mut ValueChecker::<usize>::new(tag_bits) }
    } else {
        if signed { &mut ValueChecker::<i128>::new(tag_bits) } else { &mut ValueChecker::<u128>::new(tag_bits) }
    };
    for variant in enum_def.variants.iter() {
        let expr = checker.add_variant(variant)?;
        let name = &variant.ident;
        match &variant.fields {
            syn::Fields::Unit => {
                getter.push(quote!(Self::#name => (#expr, 0)));
                cgetter.push(quote!(Self::#name => (#expr, 0)));
                setter.push(quote!(#expr => Ok(Self::#name)));
                csetter.push(quote!(#expr => Ok(Self::#name)));
                let expr = if tag_is_primary { expr } else { quote!(const { <#tag_repr>::new(#expr).unwrap() }) };
                discri.push(quote!(Self::#name => #expr));
            }
            syn::Fields::Unnamed(fld) if fld.unnamed.len() == 1 => {
                unit_only = false;
                let Some(syn::Field { ty, .. }) = fld.unnamed.first() else { unreachable!() };
                getter.push(quote!(Self::#name(fld) => (#expr, <#ty as BitsCast>::into_underlying::<#primary_repr>(fld))));
                cgetter.push(quote!(Self::#name(fld) => {
                    <#ty as PlainBitsCast>::ASSERT as ();
                    type Bits = <#ty as PlainBitsCast>::Bits;
                    let fld = unsafe { transmute::<_, <Bits as BasicInt>::Primary>(fld) } as #primary_repr;
                    const SHIFT:u32 = #primary_repr::BITS - Bits::BITS;
                    (#expr, fld << SHIFT >> SHIFT)
                }));
                setter.push(quote!(#expr => Ok(Self::#name(<#ty as BitsCast>::from_underlying(payload_raw)))));
                csetter.push(quote!(#expr => {
                    <#ty as PlainBitsCast>::ASSERT as ();
                    type Bits = <#ty as PlainBitsCast>::Bits;
                    const SHIFT:u32 = #primary_repr::BITS - Bits::BITS;
                    Ok(Self::#name(unsafe { transmute((payload_raw << SHIFT >> SHIFT) as <Bits as BasicInt>::Primary) }))
                }));
                let expr = if tag_is_primary { expr } else { quote!(const { <#tag_repr>::new(#expr).unwrap() }) };
                discri.push(quote!(Self::#name(_) => #expr));
                asserter.push(quote!(assert!(<#ty as BitsCast>::BITS <= #enum_ident::PAYLOAD_BITS)));
            }
            _ => return error(variant.span(), "only support single field tuple variant"),
        };
    }
    let false = unit_only else { return error(tag_repr.span(), "unit only enum not support tag attribute") };

    let (total_new, total_get) = if total_bits == primary_bits {
        (quote!(total_raw), quote!(let total_raw = raw;))
    } else {
        (quote!(unsafe { <#total_repr>::new_unchecked(total_raw) }), quote!(let total_raw = raw.value();))
    };
    let (tag_shl, tag_shr) = if tag_start == 0 { (quote!(), quote!()) } else { (quote!(<<Self::TAG_START), quote!(>>Self::TAG_START)) };
    let (payload_shl, payload_shr) = if payload_start == 0 { (quote!(), quote!()) } else { (quote!(<<Self::PAYLOAD_START), quote!(>>Self::PAYLOAD_START)) };
    let tag_start = Literal::u32_unsuffixed(tag_start);
    let tag_bits = Literal::u32_unsuffixed(tag_bits);
    let payload_start = Literal::u32_unsuffixed(payload_start);
    let payload_bits = Literal::u32_unsuffixed(payload_bits);
    let const_trait = const_trait();
    let out = quote!(
        const _: () = const { #(#asserter;)* };

        impl #enum_ident {
            const TAG_START: u32 = #tag_start;
            const TAG_BITS: u32 = #tag_bits;
            const PAYLOAD_START: u32 = #payload_start;
            const PAYLOAD_BITS: u32 = #payload_bits;
            const TAG_MARK: #primary_repr = !(#primary_repr::MAX << Self::TAG_BITS);
            const PAYLOAD_MARK: #primary_repr = !(#primary_repr::MAX << Self::PAYLOAD_BITS);
            const TAG_UNUSED_BITS: u32 = #sprimary_repr::BITS - Self::TAG_BITS;

            pub const fn discriminant(&self) -> #tag_repr { match self { #(#discri,)* } }

            #[inline(always)]
            pub const fn try_from_bits(raw: #total_repr) -> Result<Self, #total_repr> {
                #total_get
                let Ok(v) = Self::ctry_from_raw_parts(total_raw #tag_shr, total_raw #payload_shr) else { return Err(raw) };
                Ok(v)
            }

            #[inline(always)]
            pub const fn into_bits(self) -> #total_repr {
                let (tag_raw, payload_raw) = self.cinto_raw_parts();
                let total_raw = (tag_raw & Self::TAG_MARK) #tag_shl | (payload_raw & Self::PAYLOAD_MARK) #payload_shl;
                #total_new
            }

            #[inline(always)]
            pub const fn into_unchecked(self) -> Unchecked<Self> { Unchecked::new_with_raw_value(self.into_bits()) }

            #[inline(always)]
            pub const fn try_from_layout<Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32>(layout: BitsEnumReLayout<Self, Bits, TAG_START, PAYLOAD_START>) -> Result<Self, ()> {
                let (tag_raw, payload_raw) = layout.into_raw_parts::<#primary_repr>();
                Self::ctry_from_raw_parts(tag_raw, payload_raw)
            }

            #[inline(always)]
            pub const fn into_layout<Bits: BasicUInt, const TAG_START: u32, const PAYLOAD_START: u32>(self) -> BitsEnumReLayout<Self, Bits, TAG_START, PAYLOAD_START> {
                let (tag_raw, payload_raw) = self.cinto_raw_parts();
                BitsEnumReLayout::from_raw_parts(tag_raw, payload_raw)
            }

            const fn ctry_from_raw_parts(tag_raw: #primary_repr, payload_raw: #primary_repr) -> Result<Self, ()> {
                use ::core::mem::transmute;
                let tag_raw = (tag_raw as #sprimary_repr) << Self::TAG_UNUSED_BITS >> Self::TAG_UNUSED_BITS;
                match tag_raw { #(#csetter,)* _ => Err(()) }
            }

            const fn cinto_raw_parts(self) -> (#primary_repr, #primary_repr) {
                use ::core::mem::transmute;
                let (tag_raw, payload_raw): (#sprimary_repr, #primary_repr) = match self { #(#cgetter,)* };
                (tag_raw as #primary_repr, payload_raw)
            }
        }

        impl ReLayoutBitsEnum for #enum_ident {
            const TAG_START: u32 = #tag_start;
            const PAYLOAD_START: u32 = #payload_start;
            type Repr = #primary_repr;
            type Tag = #tag_repr;
            type Payload = #payload_repr;

            fn try_from_raw_parts(tag_raw: Self::Repr, payload_raw: Self::Repr) -> Result<Self, ()> {
                let tag_raw = (tag_raw as #sprimary_repr) << Self::TAG_UNUSED_BITS >> Self::TAG_UNUSED_BITS;
                match tag_raw { #(#setter,)* _ => Err(()) }
            }

            fn into_raw_parts(self) -> (Self::Repr, Self::Repr) {
                let (tag_raw, payload_raw): (#sprimary_repr, #primary_repr) = match self { #(#getter,)* };
                (tag_raw as #primary_repr, payload_raw)
            }
        }

        impl #const_trait Uncheckable for #enum_ident {
            type UncheckedRaw = #total_repr;

            #[inline(always)]
            fn raw_value(sf:Self) -> Self::UncheckedRaw {
                let (tag_raw, payload_raw) = Self::into_raw_parts(sf);
                let total_raw = (tag_raw & Self::TAG_MARK) #tag_shl | (payload_raw & Self::PAYLOAD_MARK) #payload_shl;
                #total_new
            }

            #[inline(always)]
            fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
                #total_get
                Self::try_from_raw_parts(total_raw #tag_shr, total_raw #payload_shr).map_err(|_|raw)
            }
        }
    );
    Ok(out)
}

struct EnumRepr {
    repr: syn::Ident,
    split_repr: Option<(FldRepr, FldRepr)>,
}
impl syn::parse::Parse for EnumRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<syn::Token![:]>(); // maybe add by `bitfld` attribute for prevent recurse loop
        let repr = input.parse()?;
        let mut tag_repr = None;
        let mut payload_repr = None;
        while input.parse::<syn::Token![,]>().is_ok() {
            let name = input.parse::<syn::Ident>()?;
            if name == "tag" {
                tag_repr = Some(input.parse()?);
            } else if name == "payload" {
                payload_repr = Some(input.parse()?);
            } else {
                return error(name.span(), "unknown attribute");
            }
        }
        match (tag_repr, payload_repr) {
            (Some(tag_repr), Some(payload_repr)) => Ok(Self { repr, split_repr: Some((tag_repr, payload_repr)) }),
            (None, None) => Ok(Self { repr, split_repr: None }),
            (Some(split), None) | (None, Some(split)) => error(split.repr.span(), "tag and payload must be both seted or both unseted"),
        }
    }
}

struct FldRepr {
    repr: syn::Ident,
    range: Option<BitsRange>,
}
impl syn::parse::Parse for FldRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let repr = content.parse()?;
        let range = if content.parse::<syn::Token![,]>().is_ok() { Some(content.parse()?) } else { None };
        Ok(Self { repr, range })
    }
}
impl FldRepr {
    fn get_range(&self, total_bits: u32, default_start: u32) -> syn::Result<(bool, u32, u32)> {
        let (signed, bits) = valid_int(&self.repr)?;
        let start = match self.range {
            None if total_bits.checked_sub(default_start).is_some_and(|rbits| rbits >= bits) => default_start,
            Some(BitsRange::Single(bit)) if bit < total_bits && bits == 1 => bit,
            Some(BitsRange::HalfOpen(start, end)) if end <= total_bits && end.checked_sub(start).is_some_and(|rbits| rbits == bits) => start,
            Some(BitsRange::Closed(start, last)) if last < total_bits && last.checked_sub(start).is_some_and(|v| v + 1 == bits) => start,
            Some(BitsRange::Bits(start, rbits)) if total_bits.checked_sub(start).is_some_and(|v| v >= bits) && rbits == bits => start,
            _ => return error(self.repr.span(), "invalid bits range"),
        };
        Ok((signed, start, bits))
    }
}

enum BitsRange {
    /// like (3) for single bit
    Single(u32),
    /// like (0..2)
    HalfOpen(u32, u32),
    /// like (0..=2)
    Closed(u32, u32),
    /// like (0:2), (bits_start:bits_len)
    Bits(u32, u32),
}
impl syn::parse::Parse for BitsRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first = input.parse::<syn::Index>()?.index;
        if let Ok(lmi) = input.parse::<syn::RangeLimits>() {
            let second = input.parse::<syn::Index>()?.index;
            match lmi {
                syn::RangeLimits::HalfOpen(_) => Ok(Self::HalfOpen(first, second)),
                syn::RangeLimits::Closed(_) => Ok(Self::Closed(first, second)),
            }
        } else if input.parse::<syn::Token![:]>().is_ok() {
            let bits: syn::Index = input.parse()?;
            Ok(Self::Bits(first, bits.index))
        } else {
            Ok(Self::Single(first))
        }
    }
}

trait CheckValue {
    fn add_variant(&mut self, variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream>;
}

trait CheckerInt: Copy + Shr<u32, Output = Self> + FromStr<Err: Display> + Ord {
    const BITS: u32;
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    fn unsuffixed(self) -> proc_macro2::Literal;
    fn checked_inc(self) -> Option<Self>;
    fn get_value(expr: &syn::Expr) -> syn::Result<Self>;
}

struct ValueChecker<T> {
    next_value: Option<T>,
    min: T,
    max: T,
}

impl<T: CheckerInt> ValueChecker<T> {
    fn new(bits: u32) -> Self {
        let unused_bits = T::BITS - bits;
        Self { next_value: Some(T::ZERO), min: T::MIN >> unused_bits, max: T::MAX >> unused_bits }
    }
}

impl<T: CheckerInt> CheckValue for ValueChecker<T> {
    fn add_variant(&mut self, variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream> {
        let (value, expr) = if let Some((_, expr)) = &variant.discriminant {
            (T::get_value(expr)?, quote!(#expr))
        } else if let Some(value) = self.next_value {
            let expr = value.unsuffixed();
            (value, quote!(#expr))
        } else {
            return error(variant.span(), "value overflow");
        };
        if value < self.min || value > self.max {
            return error(variant.span(), "value overflow");
        }
        self.next_value = value.checked_inc();
        Ok(expr)
    }
}

fn get_uint<T: CheckerInt>(expr: &syn::Expr) -> syn::Result<T> {
    let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(expr), .. }) = expr else {
        return error(expr.span(), "bitfld enum only support int literal as explicit discriminant");
    };
    expr.base10_parse()
}

fn get_sint<T: CheckerInt + Neg<Output = T>>(expr: &syn::Expr) -> syn::Result<T> {
    let (neg, expr): (bool, &syn::Expr) =
        if let syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Neg(_), expr, .. }) = expr { (true, expr) } else { (false, expr) };
    let expr: T = get_uint(expr)?;
    Ok(if neg { -expr } else { expr })
}

impl CheckerInt for usize {
    const BITS: u32 = usize::BITS;
    const MIN: Self = usize::MIN;
    const MAX: Self = usize::MAX;
    const ZERO: Self = 0;

    fn unsuffixed(self) -> proc_macro2::Literal {
        proc_macro2::Literal::usize_unsuffixed(self)
    }

    fn checked_inc(self) -> Option<Self> {
        self.checked_add(1)
    }

    fn get_value(expr: &syn::Expr) -> syn::Result<Self> {
        get_uint(expr)
    }
}

impl CheckerInt for u128 {
    const BITS: u32 = u128::BITS;
    const MIN: Self = u128::MIN;
    const MAX: Self = u128::MAX;
    const ZERO: Self = 0;

    fn unsuffixed(self) -> proc_macro2::Literal {
        proc_macro2::Literal::u128_unsuffixed(self)
    }

    fn checked_inc(self) -> Option<Self> {
        self.checked_add(1)
    }

    fn get_value(expr: &syn::Expr) -> syn::Result<Self> {
        get_uint(expr)
    }
}

impl CheckerInt for isize {
    const BITS: u32 = isize::BITS;
    const MIN: Self = isize::MIN;
    const MAX: Self = isize::MAX;
    const ZERO: Self = 0;

    fn unsuffixed(self) -> proc_macro2::Literal {
        proc_macro2::Literal::isize_unsuffixed(self)
    }

    fn checked_inc(self) -> Option<Self> {
        self.checked_add(1)
    }

    fn get_value(expr: &syn::Expr) -> syn::Result<Self> {
        get_sint(expr)
    }
}

impl CheckerInt for i128 {
    const BITS: u32 = i128::BITS;
    const MIN: Self = i128::MIN;
    const MAX: Self = i128::MAX;
    const ZERO: Self = 0;

    fn unsuffixed(self) -> proc_macro2::Literal {
        proc_macro2::Literal::i128_unsuffixed(self)
    }

    fn checked_inc(self) -> Option<Self> {
        self.checked_add(1)
    }

    fn get_value(expr: &syn::Expr) -> syn::Result<Self> {
        get_sint(expr)
    }
}
