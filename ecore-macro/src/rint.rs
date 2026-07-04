use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Rem, Sub},
    str::FromStr,
};

use quote::{format_ident, quote};

use crate::error;

pub fn rint(item: proc_macro::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let lit: LitRInt = syn::parse(item)?;
    let neg = lit.min.base10_digits().starts_with('-') || lit.max.base10_digits().starts_with('-');
    if neg {
        if let Ok(min) = lit.min.base10_parse::<isize>()
            && let Ok(max) = lit.max.base10_parse::<isize>()
        {
            RInt { min, max }.gen_def(lit, neg)
        } else {
            let min = lit.min.base10_parse::<i128>()?;
            let max = lit.max.base10_parse::<i128>()?;
            RInt { min, max }.gen_def(lit, neg)
        }
    } else {
        if let Ok(min) = lit.min.base10_parse::<usize>()
            && let Ok(max) = lit.max.base10_parse::<usize>()
        {
            RInt { min, max }.gen_def(lit, neg)
        } else {
            let min = lit.min.base10_parse::<u128>()?;
            let max = lit.max.base10_parse::<u128>()?;
            RInt { min, max }.gen_def(lit, neg)
        }
    }
}

struct LitRInt {
    min: syn::LitInt,
    range_limits: syn::RangeLimits,
    max: syn::LitInt,
    default: Option<syn::LitInt>,
    step: Option<syn::LitInt>,
    bits: Option<syn::LitInt>,
}

impl syn::parse::Parse for LitRInt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let min = input.parse()?;
        let range_limits = input.parse()?;
        let max = input.parse()?;
        let mut default = None;
        let mut step = None;
        let mut bits = None;
        while input.parse::<syn::Token![,]>().is_ok() {
            let false = input.is_empty() else { break };
            let name: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            if name == "default" {
                let None = default.replace(input.parse()?) else { return error(name.span(), "already setted") };
            } else if name == "step" {
                let None = step.replace(input.parse()?) else { return error(name.span(), "already setted") };
            } else if name == "bits" {
                let None = bits.replace(input.parse()?) else { return error(name.span(), "already setted") };
            } else {
                return error(name.span(), "Unknown property");
            }
        }
        Ok(Self { min, range_limits, max, default, step, bits })
    }
}

struct RInt<T> {
    min: T,
    max: T,
}

impl<T: ParseInt> RInt<T> {
    fn gen_def(self, lit: LitRInt, is_neg: bool) -> syn::Result<proc_macro2::TokenStream> {
        let Self { min, max } = self;
        let max = match lit.range_limits {
            syn::RangeLimits::HalfOpen(_) if min < max => max - T::ONE,
            syn::RangeLimits::Closed(_) if min <= max => max,
            _ => return error(lit.max.span(), "max must not less than min"),
        };
        let (step, min_bits) = if let Some(value) = lit.step {
            let step: T = value.base10_parse()?;
            let true = step > T::ZERO else { return error(value.span(), "step must >= 1") };
            let true = min % step == T::ZERO else { return error(lit.min.span(), "not muliple of step") };
            let true = max % step == T::ZERO else { return error(lit.max.span(), format!("{} not muliple of step", max)) };
            (if step > T::ONE { Some(step) } else { None }, (max.abs_diff(min) / step.cast_unsigned()).bit_width())
        } else {
            (None, max.abs_diff(min).bit_width())
        };
        let default = if let Some(value) = lit.default {
            let default: T = value.base10_parse()?;
            let true = (min <= default && default <= max) else { return error(value.span(), format!("default must in setted range: {}..={}", min, max)) };
            let true = (step.is_none() || step.is_some_and(|step| default % step == T::ZERO)) else { return error(value.span(), "not muliple of step") };
            Some(default)
        } else {
            None
        };
        let bits = if let Some(value) = lit.bits {
            let bits = value.base10_parse()?;
            let true = min_bits <= bits else { return error(value.span(), format!("bits underflow, valid min is {}", min_bits)) };
            let true = bits <= 128 else { return error(value.span(), "bits overflow, valid max is 128") };
            bits
        } else {
            min_bits
        };

        let repr = format_ident!("u{}", bits);
        let rr = format_ident!("RR{}{}", if is_neg { 'I' } else { 'U' }, min.bit_width().max(max.bit_width()).next_multiple_of(8).next_power_of_two());
        let tail = if let Some(step) = step {
            let default = default.unwrap_or(min).unsuffixed();
            let step = step.unsuffixed();
            quote!(, #default, #step)
        } else if let Some(default) = default {
            let default = default.unsuffixed();
            quote!(, #default)
        } else {
            quote!()
        };
        let min = min.unsuffixed();
        let max = max.unsuffixed();
        Ok(quote!(RInt<#repr, #rr<#min, #max #tail>>))
    }
}

trait ParseInt: FromStr<Err: Display> + Ord + Sub<Output = Self> + Add<Output = Self> + Copy + Rem<Output = Self> + Div<Output = Self> + Debug + Display {
    type Unsigned: ParseInt;
    const ONE: Self;
    const ZERO: Self;
    fn unsuffixed(self) -> proc_macro2::Literal;
    fn bit_width(self) -> u32;
    fn abs_diff(self, other: Self) -> Self::Unsigned;
    fn cast_unsigned(self) -> Self::Unsigned;
}

macro_rules! impl_parseint {
    ($signed:ty,$unsigned:ty, $sop:ident, $uop:ident) => {
        impl ParseInt for $signed {
            type Unsigned = $unsigned;
            const ONE: Self = 1;
            const ZERO: Self = 0;

            fn unsuffixed(self) -> proc_macro2::Literal {
                proc_macro2::Literal::$sop(self)
            }

            fn bit_width(self) -> u32 {
                let true = self != 0 else { return 1 };
                if self < 0 { (const { Self::BITS + 1 }) - self.leading_ones() } else { Self::BITS - self.leading_zeros() }
            }

            fn abs_diff(self, other: Self) -> Self::Unsigned {
                self.abs_diff(other)
            }

            fn cast_unsigned(self) -> Self::Unsigned {
                self.cast_unsigned()
            }
        }

        impl ParseInt for $unsigned {
            type Unsigned = Self;
            const ONE: Self = 1;
            const ZERO: Self = 0;

            fn unsuffixed(self) -> proc_macro2::Literal {
                proc_macro2::Literal::$uop(self)
            }

            fn bit_width(self) -> u32 {
                if self == 0 { 1 } else { Self::BITS - self.leading_zeros() }
            }

            fn abs_diff(self, other: Self) -> Self::Unsigned {
                self.abs_diff(other)
            }

            fn cast_unsigned(self) -> Self::Unsigned {
                self
            }
        }
    };
}
impl_parseint!(isize, usize, isize_unsuffixed, usize_unsuffixed);
impl_parseint!(i128, u128, i128_unsuffixed, u128_unsuffixed);
