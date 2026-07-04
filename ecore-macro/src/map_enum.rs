use proc_macro2::Span;
use quote::{ToTokens as _, format_ident, quote};
use syn::spanned::Spanned as _;

use crate::{error, valid_uint};

pub fn map_enum_derive(item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let item = syn::parse::<syn::ItemEnum>(item)?;
    let (impl_generic, type_generic, where_generic) = item.generics.split_for_impl();
    let discriminant: Vec<_> = item.attrs.iter().filter(|attr| attr.path().is_ident("discriminant")).collect();
    let count = item.variants.len();
    let mut names = Vec::new();
    names.reserve_exact(count);
    let mut nature = true;
    let mut not_unit_only = false;
    for (id, variant) in item.variants.iter().enumerate() {
        not_unit_only = not_unit_only || !matches!(variant.fields, syn::Fields::Unit);
        names.push(&variant.ident);
        let true = nature else { continue };
        let Some((_, discriminant)) = &variant.discriminant else { continue };
        nature = if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit), .. }) = discriminant
            && lit.base10_parse::<usize>().is_ok_and(|value| value == id)
        {
            true
        } else {
            false
        };
    }
    let mut out = proc_macro2::TokenStream::new();
    let name = &item.ident;
    let vis = item.vis;
    if not_unit_only {
        if discriminant.is_empty() {
            return error(item.ident.span(), "Not unit-only enum derive MapEnum require 'discriminant' attribute");
        }
        for attr in discriminant.into_iter() {
            match &attr.meta {
                syn::Meta::List(syn::MetaList { tokens, .. }) => quote! { #[ #tokens ] }.to_tokens(&mut out),
                syn::Meta::Path(..) => {}
                syn::Meta::NameValue(value) => quote! { #[ #value ] }.to_tokens(&mut out),
            }
        }
        let disname = format_ident!("{}Discriminant", name);
        let patterns = item.variants.iter().map(|variant| {
            let name = &variant.ident;
            let fields = match variant.fields {
                syn::Fields::Named(..) => quote!({ .. }),
                syn::Fields::Unnamed(..) => quote! {(..)},
                syn::Fields::Unit => quote! {},
            };
            quote!(Self::#name #fields => #disname::#name)
        });
        let repr = syn::Ident::new(&format!("u{}", (usize::BITS - names.len().leading_zeros()).next_multiple_of(8).next_power_of_two()), name.span());
        quote! {
            #[repr(#repr)]
            #vis enum #disname { #(#names),* }
             impl #disname {
                #vis const fn get_index(&self) -> usize { unsafe { core::ptr::read(self) as usize } }
            }

            impl MapEnum for #disname {
                type Discriminant = Self;
                type Array = [(); #count];
                fn iter() -> impl ExactSizeIterator<Item = Self::Discriminant> { (0..#count).map(|id| unsafe{core::mem::transmute_copy(&id)}) }
                fn get_index(&self) -> usize { Self::get_index(self) }
            }

            unsafe impl PlainMapEnum for #disname { type NatureInt = #repr; }

            impl #impl_generic MapEnum for #name #type_generic #where_generic {
                type Discriminant = #disname;
                type Array = [(); #count];
                fn iter() -> impl ExactSizeIterator<Item = Self::Discriminant> { #disname::iter() }
                fn get_index(&self) -> usize { match self { #(#patterns),* }.get_index() }
            }
        }
        .to_tokens(&mut out);
    } else {
        if let [first, ..] = discriminant.as_slice() {
            return error(first.span(), "Not tagged enum can't define additional 'Discriminant' enum");
        }
        if nature {
            quote! {
                impl #impl_generic #name #type_generic #where_generic {
                    #vis const fn get_index(&self) -> usize { unsafe { core::ptr::read(self) as usize } }
                }

                impl #impl_generic MapEnum for #name #type_generic #where_generic {
                    type Discriminant = Self;
                    type Array = [(); #count];
                    fn iter() -> impl ExactSizeIterator<Item = Self::Discriminant> { (0..#count).map(|id| unsafe{core::mem::transmute_copy(&id)}) }
                    fn get_index(&self) -> usize { Self::get_index(self) }
                }
            }
            .to_tokens(&mut out);
            if let Some(repr) = get_uint_repr(&item.attrs) {
                quote!(unsafe impl #impl_generic PlainMapEnum for #name #type_generic #where_generic { type NatureInt = #repr; }).to_tokens(&mut out);
            }
        } else {
            let ntoi = names.iter().enumerate().map(|(id, varname)| quote!(#name::#varname => #id));
            let iton = names.iter().enumerate().map(|(id, varname)| quote!(#id => #name::#varname));
            quote! {
                impl #impl_generic #name #type_generic #where_generic {
                    #vis const fn get_index(&self) -> usize { match self { #( #ntoi ),* } }
                }

                impl #impl_generic MapEnum for #name #type_generic #where_generic {
                    type Discriminant = Self;
                    type Array = [(); #count];
                    fn iter() -> impl ExactSizeIterator<Item = Self> { (0..#count).map(|id| match id { #( #iton , )* _ => unsafe { core::hint::unreachable_unchecked() } }) }
                    fn get_index(&self) -> usize { Self::get_index(self) }
                }
            }
            .to_tokens(&mut out);
        }
    }
    Ok(out)
}

pub fn map_enum(item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let expr = syn::parse::<syn::ExprMatch>(item)?;
    let out = syn::Ident::new("out", Span::mixed_site());
    let buf = syn::Ident::new("buf", Span::mixed_site());
    let writefn = &syn::Ident::new("write", Span::mixed_site());
    match expr.expr.as_ref() {
        syn::Expr::Path(enum_path) => {
            let mut check: Vec<_> = Vec::new();
            let mut write: Vec<_> = Vec::new();
            check.reserve(expr.arms.len());
            write.reserve(expr.arms.len());
            for arm in expr.arms.iter() {
                if let Some(guard) = &arm.guard {
                    return error(guard.0.span(), "not support guard");
                }
                let syn::Pat::Ident(syn::PatIdent { attrs: _, by_ref: None, mutability: None, ident, subpat: None }) = &arm.pat else {
                    return error(arm.pat.span(), "only support enum variant name");
                };
                check.push(quote!(Discriminant::#ident => (),));
                let value = arm.body.as_ref();
                let attrs = &arm.attrs;
                write.push(quote!(#(#attrs)* #writefn(&mut #buf[Discriminant::#ident.get_index()], #value);));
            }
            let path = &enum_path.path;
            Ok(quote!({
                type Discriminant = <#path as MapEnum>::Discriminant;
                let #writefn = {
                    fn check(id:&Discriminant) { match id { #(#check)* } }
                    const fn write<T>(ptr:&mut T, v:T) { unsafe{::core::ptr::write(ptr, v)} }
                    write
                };
                let mut #out = ::core::mem::MaybeUninit::<EnumMap::<#path, _>>::uninit();
                #[allow(unused_unsafe)]
                let #buf = unsafe { #out.assume_init_mut() }.as_array_mut();
                #(#write)*
                #[allow(unused_unsafe)]
                unsafe { #out.assume_init() }
            }))
        }
        syn::Expr::Tuple(enum_paths) => {
            let num = enum_paths.elems.len();
            let mut paths: Vec<_> = Vec::new();
            let mut discs: Vec<_> = Vec::new();
            paths.reserve(num);
            discs.reserve(num);
            for (id, pat) in enum_paths.elems.iter().enumerate() {
                let syn::Expr::Path(path) = pat else { return error(pat.span(), "only support `ident`") };
                paths.push(&path.path);
                discs.push(format_ident!("Discriminant{}", id));
            }
            let mut check: Vec<_> = Vec::new();
            let mut write: Vec<_> = Vec::new();
            check.reserve(expr.arms.len());
            write.reserve(expr.arms.len());
            for arm in expr.arms.iter() {
                if let Some(guard) = &arm.guard {
                    return error(guard.0.span(), "not support guard");
                }
                let syn::Pat::Tuple(pats) = &arm.pat else {
                    return error(arm.pat.span(), "only support enum variant name tuple");
                };
                let true = pats.elems.len() == num else { return error(pats.span(), "names number not match enums number") };
                let mut idents: Vec<_> = Vec::with_capacity(num);
                for pat in pats.elems.iter() {
                    let syn::Pat::Ident(syn::PatIdent { attrs: _, by_ref: None, mutability: None, ident, subpat: None }) = pat else {
                        return error(arm.pat.span(), "only support enum variant name");
                    };
                    idents.push(ident);
                }
                let check_idents = discs.iter().zip(idents.iter()).map(|(disc, ident)| quote!(#disc::#ident));
                let write_exprs = check_idents.clone().map(|ident| quote!(.as_array_mut()[#ident.get_index()]));
                check.push(quote!((#(#check_idents,)*) => (),));
                let value = arm.body.as_ref();
                let attrs = &arm.attrs;
                write.push(quote!(#(#attrs)* #writefn(&mut #buf #(#write_exprs)*, #value);));
            }
            let alias = discs.iter().zip(paths.iter()).map(|(disc, path)| quote!(type #disc = <#path as MapEnum>::Discriminant;));
            let mut full_type = quote!(_);
            for path in paths.iter().rev() {
                full_type = quote!(EnumMap::<#path, #full_type>);
            }
            Ok(quote!({
                #(#alias)*
                let #writefn = {
                    fn check(id:&(#(#discs,)*)) { match id { #(#check)* } }
                    const fn write<T>(ptr:&mut T, v:T) { unsafe{::core::ptr::write(ptr, v)} }
                    write
                };
                let mut #out = ::core::mem::MaybeUninit::<#full_type>::uninit();
                #[allow(unused_unsafe)]
                let #buf = unsafe { #out.assume_init_mut() };
                #(#write)*
                #[allow(unused_unsafe)]
                unsafe { #out.assume_init() }
            }))
        }
        _ => error(expr.expr.span(), "only support `ident` or `(ident, ...)`"),
    }
}

fn get_uint_repr(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        let true = attr.path().is_ident("repr") else { continue };
        let Ok(metas) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated) else { return None };
        for meta in metas {
            let Some(ident) = meta.path().get_ident() else { continue };
            if valid_uint(ident).is_ok() {
                return Some(ident.clone());
            }
        }
    }
    None
}
