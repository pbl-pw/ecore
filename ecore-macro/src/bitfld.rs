use crate::{Either, error};

enum BitFldItem {
    Struct(syn::ItemStruct),
    Enum { attrs: Vec<syn::Attribute>, vis: syn::Visibility, body: proc_macro2::TokenStream },
}

impl syn::parse::Parse for BitFldItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input).unwrap_or_default();
        let vis: syn::Visibility = input.parse()?;
        let Ok(item) = input.parse::<Either<syn::ItemStruct, syn::Token![enum]>>() else {
            return if input.parse::<syn::ImplItemFn>().is_ok() {
                error(input.span(), "Bit field using bitfld!(..) type not support #[bitfld(..)] attribute")
            } else {
                error(input.span(), "bitfld only support struct or enum")
            };
        };
        match item {
            Either::Left(mut item) => {
                item.attrs.extend(attrs);
                item.vis = vis;
                Ok(Self::Struct(item))
            }
            Either::Right(_) => {
                let body = input.parse()?;
                Ok(Self::Enum { attrs, vis, body })
            }
        }
    }
}

pub fn bitfld(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    match syn::parse::<BitFldItem>(item)? {
        BitFldItem::Struct(struct_def) => bitstruct::bitfld(attr, struct_def),
        BitFldItem::Enum { attrs, vis, body } => bitenum::bitfld(attr, attrs, vis, body),
    }
}

fn const_trait() -> Option<syn::Token![const]> {
    #[cfg(feature = "const-trait")]
    return Some(syn::Token![const](proc_macro2::Span::call_site()));
    #[cfg(not(feature = "const-trait"))]
    return None;
}

pub(crate) mod bitenum;
mod bitstruct;
