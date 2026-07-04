use core::num::NonZero;

use crate::bitint::BitInt;

macro_rules! impl_int {
    ($type:ty) => {
        impl<const BITS: u32> BitInt<NonZero<$type>, BITS> {
            /// Returns the inner non-zero `BitInt` value.
            pub const fn get(self) -> BitInt<$type, BITS> {
                BitInt(self.0.get())
            }

            /// Constructs a non-zero `BitInt` from a regular `BitInt`, returning `None` if the value is zero.
            pub const fn new_non_zero(v: BitInt<$type, BITS>) -> Option<Self> {
                let Some(v) = NonZero::new(v.0) else { return None };
                Some(BitInt(v))
            }
        }
    };
}
impl_int!(u8);
impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(u128);
impl_int!(usize);
impl_int!(i8);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);
impl_int!(i128);
impl_int!(isize);
