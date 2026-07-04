#![allow(non_camel_case_types)]

use seq_macro::seq;

use super::BitInt;

seq!(N in 1..=7 {
    /// See [`BitInt`]
    pub type u~N = BitInt<u8, N>;
    /// See [`BitInt`]
    pub type i~N = BitInt<i8, N>;
});
seq!(N in 9..=15 {
    /// See [`BitInt`]
    pub type u~N = BitInt<u16, N>;
    /// See [`BitInt`]
    pub type i~N = BitInt<i16, N>;
});
seq!(N in 17..=31 {
    /// See [`BitInt`]
    pub type u~N = BitInt<u32, N>;
    /// See [`BitInt`]
    pub type i~N = BitInt<i32, N>;
});
seq!(N in 33..=63 {
    /// See [`BitInt`]
    pub type u~N = BitInt<u64, N>;
    /// See [`BitInt`]
    pub type i~N = BitInt<i64, N>;
});
seq!(N in 65..=127 {
    /// See [`BitInt`]
    pub type u~N = BitInt<u128, N>;
    /// See [`BitInt`]
    pub type i~N = BitInt<i128, N>;
});
