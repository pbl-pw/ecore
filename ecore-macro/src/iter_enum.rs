use std::{fmt::Display, mem::replace, str::FromStr};

use proc_macro2::Literal;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

use crate::{error, valid_int};

pub fn iter_enum_derive(item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item = syn::parse::<syn::ItemEnum>(item)?;
    let (signed, bits, repr) =
        if let Some(repr) = get_int_repr(&item.attrs) { repr } else { (true, 32, syn::Ident::new("i32", proc_macro2::Span::call_site())) }; // default is i32
    let count = item.variants.len();
    let mut names: Vec<_> = Vec::new();
    let mut values: Vec<_> = Vec::new();
    names.reserve_exact(count);
    values.reserve_exact(count);
    let mut nature = true;
    let value_getter: &mut dyn GetValue = if signed {
        if bits <= usize::BITS { &mut ValueGetter::<isize>::default() } else { &mut ValueGetter::<i128>::default() }
    } else {
        if bits <= usize::BITS { &mut ValueGetter::<usize>::default() } else { &mut ValueGetter::<u128>::default() }
    };
    for (id, variant) in item.variants.iter().enumerate() {
        names.push(variant.ident.to_string());
        let (value, eqindex) = value_getter.add_variant(variant, id)?;
        nature = nature && eqindex;
        values.push(value);
    }
    let (impl_generic, type_generic, where_generic) = item.generics.split_for_impl();
    let name = &item.ident;
    let out = if nature {
        quote!(
            impl #impl_generic IterEnumDiscriminants for #name #type_generic #where_generic {
                type Value = #repr;
                type Discriminants = (EnumMap<Self::Discriminant, &'static str>, ::core::marker::PhantomData<EnumMap<Self::Discriminant, Self::Value>>);
                const DISCRIMINANTS: &Self::Discriminants = &(EnumMap::from_array([#( #names ),*]), ::core::marker::PhantomData);
            }
        )
    } else {
        quote!(
            impl #impl_generic IterEnumDiscriminants for #name #type_generic #where_generic {
                type Value = #repr;
                type Discriminants = (EnumMap<Self::Discriminant, &'static str>, EnumMap<Self::Discriminant, Self::Value>);
                const DISCRIMINANTS: &Self::Discriminants = &(EnumMap::from_array([#( #names ),*]), EnumMap::from_array([#( #values ),*]));
            }
        )
    };
    Ok(out)
}

fn get_int_repr(attrs: &[syn::Attribute]) -> Option<(bool, u32, syn::Ident)> {
    for attr in attrs {
        let true = attr.path().is_ident("repr") else { continue };
        let Ok(metas) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) else { return None };
        for meta in metas {
            let Some(ident) = meta.path().get_ident() else { continue };
            if let Ok(repr) = valid_int(ident) {
                return Some((repr.0, repr.1, ident.clone()));
            }
        }
    }
    None
}

trait GetValue {
    fn add_variant(&mut self, variant: &syn::Variant, index: usize) -> syn::Result<(proc_macro2::TokenStream, bool)>;
}

#[derive(Default)]
struct ValueGetter<T> {
    next_value: T,
}

impl<T: GetValueInt> GetValue for ValueGetter<T> {
    fn add_variant(&mut self, variant: &syn::Variant, index: usize) -> syn::Result<(proc_macro2::TokenStream, bool)> {
        if let Some((_, discriminant)) = &variant.discriminant {
            let (neg, lit) = if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) = discriminant {
                (None, lit)
            } else if let syn::Expr::Unary(syn::ExprUnary { op: syn::UnOp::Neg(op), expr, .. }) = discriminant
                && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) = &**expr
            {
                (Some(op), lit)
            } else {
                return error(discriminant.span(), "IterEnumDiscriminants only support int literal");
            };
            let value: T = lit.base10_parse()?;
            let value = value.neg(neg)?;
            Ok((lit.into_token_stream(), value.eq_index(index)))
        } else {
            let next_value = self.next_value.step_one();
            let value = replace(&mut self.next_value, next_value);
            Ok((value.unsuffixed().into_token_stream(), value.eq_index(index)))
        }
    }
}

trait GetValueInt: Copy + FromStr<Err: Display> + Default {
    fn step_one(self) -> Self;
    fn unsuffixed(self) -> Literal;
    fn eq_index(self, index: usize) -> bool;
    fn neg(self, neg: Option<&syn::token::Minus>) -> syn::Result<Self>;
}

impl GetValueInt for usize {
    fn step_one(self) -> Self {
        self + 1
    }

    fn unsuffixed(self) -> Literal {
        Literal::usize_unsuffixed(self)
    }

    fn eq_index(self, index: usize) -> bool {
        self == index
    }

    fn neg(self, neg: Option<&syn::token::Minus>) -> syn::Result<Self> {
        if let Some(neg) = neg {
            return error(neg.span, "value overflow");
        }
        Ok(self)
    }
}

impl GetValueInt for isize {
    fn step_one(self) -> Self {
        self + 1
    }

    fn unsuffixed(self) -> Literal {
        Literal::isize_unsuffixed(self)
    }

    fn eq_index(self, index: usize) -> bool {
        !self.is_negative() && self as usize == index
    }

    fn neg(self, neg: Option<&syn::token::Minus>) -> syn::Result<Self> {
        Ok(if neg.is_some() { -self } else { self })
    }
}

impl GetValueInt for u128 {
    fn step_one(self) -> Self {
        self + 1
    }

    fn unsuffixed(self) -> Literal {
        Literal::u128_unsuffixed(self)
    }

    fn eq_index(self, index: usize) -> bool {
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        {
            self == index as u128
        }
    }

    fn neg(self, neg: Option<&syn::token::Minus>) -> syn::Result<Self> {
        if let Some(neg) = neg {
            return error(neg.span, "value overflow");
        }
        Ok(self)
    }
}

impl GetValueInt for i128 {
    fn step_one(self) -> Self {
        self + 1
    }

    fn unsuffixed(self) -> Literal {
        Literal::i128_unsuffixed(self)
    }

    fn eq_index(self, index: usize) -> bool {
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        {
            !self.is_negative() && self as u128 == index as u128
        }
    }

    fn neg(self, neg: Option<&syn::token::Minus>) -> syn::Result<Self> {
        Ok(if neg.is_some() { -self } else { self })
    }
}
