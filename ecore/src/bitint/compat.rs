use super::*;

impl<T: PrimaryInt, const BITS: u32> BitInt<T, BITS> {
    /// Extracts `BITS` bits from a `u8` starting at `start_bit`.
    pub const fn extract_u8(value: u8, start_bit: usize) -> Self {
        Self::cast_new(CInt::cast_as(value >> start_bit))
    }

    /// Extracts `BITS` bits from a `u16` starting at `start_bit`.
    pub const fn extract_u16(value: u16, start_bit: usize) -> Self {
        Self::cast_new(CInt::cast_as(value >> start_bit))
    }

    /// Extracts `BITS` bits from a `u32` starting at `start_bit`.
    pub const fn extract_u32(value: u32, start_bit: usize) -> Self {
        Self::cast_new(CInt::cast_as(value >> start_bit))
    }

    /// Extracts `BITS` bits from a `u64` starting at `start_bit`.
    pub const fn extract_u64(value: u64, start_bit: usize) -> Self {
        Self::cast_new(CInt::cast_as(value >> start_bit))
    }

    /// Extracts `BITS` bits from a `u128` starting at `start_bit`.
    pub const fn extract_u128(value: u128, start_bit: usize) -> Self {
        Self::cast_new(CInt::cast_as(value >> start_bit))
    }

    /// Returns the bit-reversed value, where the LSB becomes the MSB within `BITS`.
    pub const fn reverse_bits(self) -> Self {
        Self(if BITS == 0 { T::ZERO } else { CInt::shr(CInt::reverse_bits(self.0), Self::UNUSED_BITS) })
    }
}
