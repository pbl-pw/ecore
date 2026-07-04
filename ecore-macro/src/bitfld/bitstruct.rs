use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;

use crate::{bitfld::const_trait, error, valid_uint};

pub(super) fn bitfld(attr: proc_macro::TokenStream, mut struct_def: syn::ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    use syn::Fields;
    let fields = match &mut struct_def.fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(_) => return error(struct_def.fields.span(), "bit struct must not defined as tuple struct, although it's actually a tuple struct"),
        Fields::Unit => return error(struct_def.fields.span(), "bit struct must not unit struct"),
    };
    let ReprField { vis, repr, overlay: global_ovlay, relative } = syn::parse(attr)?;
    let total_bits = valid_uint(&repr)?;
    let ovlay_checker: &mut dyn CheckOverlay = if total_bits <= usize::BITS { &mut OverlayChecker::<usize>(0) } else { &mut OverlayChecker::<u128>(0) };
    let mut fldsdef = quote!(
        #[inline(always)] #vis const fn mut_fld<FLD: BitsCast, const START: u32, const END: u32>(&mut self, _:fn(&Self)->&BitField<Self,FLD,START,END>) -> &mut BitField<Self,FLD,START,END> { BitField::bind_mut(self) }
    );
    let mut next_bit = 0;
    for syn::Field { attrs, vis, ident, ty, .. } in fields.named.iter_mut() {
        let Some(fldname) = ident else { return error(ident.span(), "bit field must have a name") };
        let bits_attr = if let syn::Type::Macro(mty) = ty
            && mty.mac.path.is_ident("bitfld")
        {
            let BitsType { bitsty, attr } = syn::parse2(std::mem::replace(&mut mty.mac.tokens, quote!()))?;
            *ty = bitsty;
            attr
        } else {
            let id = attrs.iter().position(|attr| attr.path().is_ident("bitfld"));
            let Some(attr) = id.map(|id| attrs.remove(id)) else {
                return error(ident.span(), "bit struct only support bit field, maybe using bitfld!(..) type or add #[bitfld(..)]");
            };
            attr.parse_args::<BitsAttr>()?
        };
        bits_attr.valid_relative(relative)?;
        let BitsAttr { range, range2, overlay, span: attr_span } = bits_attr;
        let true = !(global_ovlay && overlay) else { return error(attr_span, "Overlay has setted by bit struct") };
        let overlay = global_ovlay || overlay;
        let (start, last) = range.get_range(total_bits, &mut next_bit, overlay, ovlay_checker, attr_span)?;
        if let Some(range2) = range2 {
            let (start2, last2) = range2.get_range(total_bits, &mut next_bit, overlay, ovlay_checker, attr_span)?;
            let true = (last2 < start || last < start2) else { return error(attr_span, "must not overlay inside bit field's multi range") };
            let true = last + 1 != start2 else { return error(attr_span, "multi range is continue, using single range instead") };
            quote!(#(#attrs)* #[inline(always)] #vis const fn #fldname(&self) -> &BitField2<Self, #ty, #start, #last, #start2, #last2> { BitField2::bind_ref(self) })
                .to_tokens(&mut fldsdef);
        } else {
            let last = (start != last).then_some(quote!(, #last));
            quote!(#(#attrs)* #[inline(always)] #vis const fn #fldname(&self) -> &BitField<Self, #ty, #start #last> { BitField::bind_ref(self) })
                .to_tokens(&mut fldsdef);
        }
    }

    let mut out = proc_macro2::TokenStream::new();
    let is_primary = total_bits.is_multiple_of(8) && total_bits.is_power_of_two();
    let syn::ItemStruct { attrs, vis: struct_vis, ident: struct_name, generics, .. } = &struct_def;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let struct_repr = if attrs.iter().any(|attr| attr.path().is_ident("repr")) { quote!() } else { quote!(#[repr(transparent)]) };
    let const_trait = const_trait();
    quote!(
        #(#attrs)* #struct_repr #struct_vis struct #struct_name #impl_generics(#vis #repr) #where_clause;

        impl #impl_generics #struct_name #ty_generics #where_clause { #fldsdef }

        impl #impl_generics Copy for #struct_name #ty_generics #where_clause {}
        impl #impl_generics Clone for #struct_name #ty_generics #where_clause { fn clone(&self) -> Self { Self(self.0) } }

        unsafe impl #impl_generics Zeroable for #struct_name #ty_generics #where_clause {}

        impl #impl_generics #const_trait BitsCast for #struct_name #ty_generics #where_clause {
            const BITS: u32 = #total_bits;
            #[inline(always)]
            fn into_underlying<Bits: PrimaryInt>(sf: Self) -> Bits { sf.0.cast_as() }
            #[inline(always)]
            fn from_underlying<Bits: PrimaryInt>(v: Bits) -> Self { Self(v.cast_as()) }
        }
        unsafe impl #impl_generics #const_trait PlainBitsCast for #struct_name #ty_generics #where_clause {
            type Bits = #repr;
        }
    )
    .to_tokens(&mut out);
    if is_primary {
        quote!(
            unsafe impl #impl_generics Pod for #struct_name #ty_generics #where_clause {}

            unsafe impl #impl_generics #const_trait ConvertEndian for #struct_name #ty_generics #where_clause {
                type Store = #repr;
                #[inline(always)]
                unsafe fn from_store(store: Self::Store) -> Self { Self(store) }
                #[inline(always)]
                fn into_store(src: Self) -> Self::Store { src.0 }
            }
        )
        .to_tokens(&mut out);
    } else {
        quote!(
            impl #impl_generics #const_trait Uncheckable for #struct_name #ty_generics #where_clause {
                type UncheckedRaw = <#repr as Uncheckable>::UncheckedRaw;
                #[inline(always)]
                fn raw_value(sf: Self) -> Self::UncheckedRaw { sf.0.value() }
                #[inline(always)]
                fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> { <#repr>::try_new(raw).map(#struct_name) }
            }
            unsafe impl #impl_generics #const_trait PlainUncheckable for #struct_name #ty_generics #where_clause {}
        )
        .to_tokens(&mut out);
    }
    Ok(out)
}

enum BitsRange {
    /// like (3) for single bit
    Single(u32),
    /// like (0..2), (..2), (0..), (..), for bits range, omit start means next bit of last field, omited end means underlying bits len
    HalfOpen(Option<u32>, Option<u32>),
    /// like (0..=2), (..=2), for bits range, omit start means next bit of last field
    Closed(Option<u32>, u32),
    /// like (0:2), (:2), (bits_start:bits_len), for bits range,, start bit is always next bit of last field, like C's bit field means
    Bits(Option<u32>, u32),
}
impl BitsRange {
    fn is_absolute(&self) -> bool {
        matches!(self, BitsRange::Single(_) | BitsRange::HalfOpen(Some(_), Some(_)) | BitsRange::Closed(Some(_), _) | BitsRange::Bits(Some(_), _))
    }

    fn get_range(
        &self,
        total_bits: u32,
        next_bit: &mut u32,
        overlay: bool,
        ovlay_checker: &mut dyn CheckOverlay,
        span: proc_macro2::Span,
    ) -> syn::Result<(u32, u32)> {
        let (start, end) = match *self {
            Self::Single(id) => (Some(id), id + 1),
            Self::HalfOpen(start, end) => (start, end.unwrap_or(total_bits)),
            Self::Closed(start, last) => (start, last + 1),
            Self::Bits(start, bits) => (start, start.unwrap_or(*next_bit) + bits),
        };
        let start = start.unwrap_or(core::mem::replace(next_bit, end));
        let true = (start < end && end <= total_bits) else { return error(span, "Bits range overflow") };
        if ovlay_checker.has_overlay(start, end) && !overlay {
            return error(span, "Bits range is overlayed");
        }
        Ok((start, end - 1))
    }
}
impl syn::parse::Parse for BitsRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first = input.parse::<syn::Index>().ok().map(|id| id.index);
        let out = if let Ok(lmi) = input.parse::<syn::RangeLimits>() {
            match lmi {
                syn::RangeLimits::HalfOpen(_) => Ok(Self::HalfOpen(first, input.parse::<syn::Index>().ok().map(|id| id.index))),
                syn::RangeLimits::Closed(_) => Ok(Self::Closed(first, input.parse::<syn::Index>()?.index)),
            }
        } else if input.parse::<syn::Token![:]>().is_ok() {
            let bits: syn::Index = input.parse()?;
            Ok(Self::Bits(first, bits.index))
        } else if let Some(first) = first {
            Ok(Self::Single(first))
        } else {
            Err(())
        };
        if let Ok(out) = out {
            Ok(out)
        } else {
            error(
                input.span(),
                "Invalid bits range. (3) for single bit, (0..2) or (0..=2) like range expr for bits range, (0:2) for (bits_start:bits_len), (:2) for C like bits number",
            )
        }
    }
}

struct BitsAttr {
    range: BitsRange,
    range2: Option<BitsRange>,
    overlay: bool,
    span: proc_macro2::Span,
}
impl BitsAttr {
    fn valid_relative(&self, relative: bool) -> syn::Result<()> {
        let false = relative else { return Ok(()) };
        if self.range.is_absolute() {
            Ok(())
        } else {
            error(self.span, "relative range is not enabled, using #[bitfld(repr, .., relative, ..)] like attribute for bit struct can fix it")
        }
    }
}
impl syn::parse::Parse for BitsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let (range, range2) = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let range: BitsRange = content.parse()?;
            content.parse::<syn::Token![,]>()?;
            let range2: BitsRange = content.parse()?;
            let true = (range.is_absolute() && range2.is_absolute()) else { return error(content.span(), "multi range must be absolute") };
            (range, Some(range2))
        } else {
            (input.parse()?, None)
        };
        let overlay = if input.parse::<syn::Token![,]>().is_ok() {
            input.parse::<bkw::overlay>()?;
            true
        } else {
            false
        };
        Ok(Self { range, range2, overlay, span })
    }
}

struct BitsType {
    bitsty: syn::Type,
    attr: BitsAttr,
}
impl syn::parse::Parse for BitsType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let attr = input.parse()?;
        Ok(Self { bitsty: ty, attr })
    }
}

struct ReprField {
    vis: syn::Visibility,
    repr: syn::Ident,
    overlay: bool,
    relative: bool,
}
impl syn::parse::Parse for ReprField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse::<syn::Visibility>().ok().unwrap_or(syn::Visibility::Inherited);
        let repr = input.parse::<syn::Ident>()?;
        let mut overlay = false;
        let mut relative = false;
        while input.parse::<syn::Token![,]>().is_ok() {
            let option = input.parse::<syn::Ident>()?;
            if option == "overlay" {
                overlay = true;
            } else if option == "relative" {
                relative = true;
            } else {
                return error(option.span(), "Unknown attribute");
            }
        }
        Ok(Self { vis, repr, overlay, relative })
    }
}

mod bkw {
    syn::custom_keyword!(overlay);
}

trait CheckOverlay {
    fn has_overlay(&mut self, start: u32, end: u32) -> bool;
}

struct OverlayChecker<T>(T);

impl CheckOverlay for OverlayChecker<usize> {
    fn has_overlay(&mut self, start: u32, end: u32) -> bool {
        let left_unused = usize::BITS - end;
        let unused = left_unused + start;
        let overlay = self.0 << left_unused >> unused != 0;
        self.0 |= usize::MAX >> unused << start;
        overlay
    }
}

impl CheckOverlay for OverlayChecker<u128> {
    fn has_overlay(&mut self, start: u32, end: u32) -> bool {
        let left_unused = u128::BITS - end;
        let unused = left_unused + start;
        let overlay = self.0 << left_unused >> unused != 0;
        self.0 |= u128::MAX >> unused << start;
        overlay
    }
}
