use core::hint::unreachable_unchecked;

use imp::EncodedVarInt;

use crate::int::{BasicInt, CInt, CalcFitted, PrimaryInt, PrimarySInt, PrimaryUInt};

const DATA_BITS: u32 = u8::BITS - 1;

/// Variable-length integer encoding/decoding utilities.
///
/// Uses a 7-bit-per-byte encoding scheme where the high bit of each byte
/// indicates whether more bytes follow (similar to Protocol Buffers varint).
pub struct VarInt;

impl VarInt {
    /// require header bytes for slice len
    pub const fn header_bytes_for(len: usize) -> usize {
        let Some(bits) = len.checked_ilog2() else { return 1 };
        bits.div_ceil(DATA_BITS) as usize
    }

    /// max storable data bytes for total buf len
    pub const fn max_bytes_for(total_len: usize) -> usize {
        let header_bytes = Self::header_bytes_for(total_len);
        let max_bytes = total_len - header_bytes;
        let true = header_bytes > 1 else { return max_bytes };
        let nmax = total_len - header_bytes + 1;
        if Self::header_bytes_for(nmax) < header_bytes { nmax } else { max_bytes }
    }

    /// max bytes for encoded int type
    pub const fn max_bytes_of<T: PrimaryInt>() -> usize {
        const { (T::BITS as usize + 6) / 7 } // bits = n * 7, bits / 7 = n <= (bits + 6) / 7
    }

    /// decode unsigned PrimaryInt from bytes
    pub const fn decode_unsigned<Tgt: PrimaryUInt>(data: &[u8]) -> Result<(Tgt, usize), ()> {
        if let Ok((v, len)) = unsafe { decode_unsigned_imp::<Tgt::CalcFitted>(data.as_ptr(), const { Tgt::CalcFitted::BITS - Tgt::BITS }, data.len()) } {
            Ok((CInt::cast_as(v), len))
        } else {
            Err(())
        }
    }

    /// decode unsigned PrimaryInt from bytes ptr
    pub const unsafe fn decode_unsigned_raw<Tgt: PrimaryUInt>(data: *const u8) -> Result<(Tgt, usize), ()> {
        if let Ok((v, len)) =
            unsafe { decode_unsigned_imp::<Tgt::CalcFitted>(data, const { Tgt::CalcFitted::BITS - Tgt::BITS }, const { Self::max_bytes_of::<Tgt>() }) }
        {
            Ok((CInt::cast_as(v), len))
        } else {
            Err(())
        }
    }

    /// decode bytes slice which len encoded as varint
    pub const fn decode_slice<Tgt: PrimaryUInt>(data: &[u8]) -> Result<(&[u8], usize), ()> {
        if let Ok((v, hdr_len)) = unsafe { decode_unsigned_imp::<Tgt::CalcFitted>(data.as_ptr(), const { Tgt::CalcFitted::BITS - Tgt::BITS }, data.len()) } {
            let len = CInt::cast_as(v);
            let total = len + hdr_len;
            let true = total <= data.len() else { return Err(()) };
            Ok((unsafe { core::slice::from_raw_parts(data.as_ptr().add(hdr_len), len) }, total))
        } else {
            Err(())
        }
    }

    /// decode bytes slice which len encoded as varint
    pub const unsafe fn decode_slice_raw<Tgt: PrimaryUInt>(data: *const u8) -> Result<(&'static [u8], usize), ()> {
        if let Ok((v, hdr_len)) =
            unsafe { decode_unsigned_imp::<Tgt::CalcFitted>(data, const { Tgt::CalcFitted::BITS - Tgt::BITS }, const { Self::max_bytes_of::<Tgt>() }) }
        {
            let len = CInt::cast_as(v);
            Ok((unsafe { core::slice::from_raw_parts(data.add(hdr_len), len) }, len + hdr_len))
        } else {
            Err(())
        }
    }

    /// encode unsigned PrimaryInt to bytes
    pub const fn encode_unsigned_to<Src: PrimaryUInt>(v: Src, buf: &mut [u8]) -> Option<&mut [u8]> {
        encode_unsigned_imp(CInt::cast_as_calc(v), buf)
    }

    /// encode unsigned PrimaryInt to bytes
    pub const fn encode_unsigned<Src: PrimaryUInt>(v: Src) -> EncodedVarInt<<Src::CalcFitted as CalcFitted>::VarIntBuf> {
        let mut buf = EncodedVarInt::new();
        let Some(out) = encode_unsigned_imp(CInt::cast_as_calc(v), buf.as_mut()) else { unsafe { unreachable_unchecked() } };
        let len = out.len();
        buf.set_len(len);
        buf
    }

    /// zigzag encode signed PrimaryInt to unsigned PrimaryInt
    const fn encode_zigzag<Src: PrimarySInt>(src: Src) -> Src::Unsigned {
        CInt::cast_as(encode_zigzag_imp(CInt::cast_as_calc(src)))
    }

    /// zigzag decode unsigned PrimaryInt to signed PrimaryInt
    const fn decode_zigzag<Src: PrimaryUInt>(src: Src) -> Src::Signed {
        CInt::cast_as(decode_zigzag_imp(CInt::cast_as_calc(src)))
    }

    /// encode PrimaryInt, for signed zigzag then varint, for unsigned only varint
    pub const fn encode_to<Src: PrimaryInt>(v: Src, buf: &mut [u8]) -> Option<&mut [u8]> {
        if Src::SIGNED {
            Self::encode_unsigned_to(Self::encode_zigzag(CInt::cast_as_signed(v)), buf)
        } else {
            Self::encode_unsigned_to(CInt::cast_as_unsigned(v), buf)
        }
    }

    /// encode PrimaryInt, for signed zigzag then varint, for unsigned only varint
    pub const fn encode<Src: PrimaryInt>(v: Src) -> EncodedVarInt<<<Src::Unsigned as BasicInt>::CalcFitted as CalcFitted>::VarIntBuf> {
        if Src::SIGNED { Self::encode_unsigned(Self::encode_zigzag(CInt::cast_as_signed(v))) } else { Self::encode_unsigned(CInt::cast_as_unsigned(v)) }
    }

    /// decode PrimaryInt, for signed varint then zigzag, for unsigned only varint
    pub const fn decode<Tgt: PrimaryInt>(data: &[u8]) -> Result<(Tgt, usize), ()> {
        let Ok((v, len)) = Self::decode_unsigned::<Tgt::Unsigned>(data) else { return Err(()) };
        if Tgt::SIGNED { Ok((CInt::cast_from_signed(Self::decode_zigzag(v)), len)) } else { Ok((CInt::cast_from_unsigned(v), len)) }
    }

    /// decode PrimaryInt, for signed varint then zigzag, for unsigned only varint
    pub const unsafe fn decode_raw<Tgt: PrimaryInt>(data: *const u8) -> Result<(Tgt, usize), ()> {
        let Ok((v, len)) = (unsafe { Self::decode_unsigned_raw::<Tgt::Unsigned>(data) }) else { return Err(()) };
        if Tgt::SIGNED { Ok((CInt::cast_from_signed(Self::decode_zigzag(v)), len)) } else { Ok((CInt::cast_from_unsigned(v), len)) }
    }
}

const unsafe fn decode_unsigned_imp<Tgt: PrimaryInt + CalcFitted>(data: *const u8, zero_bits: u32, max_bytes: usize) -> Result<(Tgt, usize), ()> {
    let mut v = Tgt::ZERO;
    let mut i = 0;
    loop {
        let true = i < max_bytes else { return Err(()) };
        let byte = unsafe { data.add(i).read() };
        let Some(nv) = CInt::checked_shl(CInt::cast_as(byte & 0b0111_1111), DATA_BITS * i as u32) else { return Err(()) };
        v = CInt::bitor(v, nv);
        i += 1;
        if byte & 0b1000_0000 == 0 {
            break;
        }
    }
    if CInt::leading_zeros(v) >= zero_bits { Ok((v, i)) } else { Err(()) }
}

const fn encode_unsigned_imp<Src: PrimaryInt + CalcFitted>(mut v: Src, buf: &mut [u8]) -> Option<&mut [u8]> {
    let max_len = buf.len();
    let out = buf.as_mut_ptr();
    let mut i = 0;
    while i < max_len {
        let byte: u8 = CInt::cast_as(v);
        v = CInt::shr(v, DATA_BITS);
        let len = i + 1;
        if CInt::eq(v, Src::ZERO) {
            unsafe { out.add(i).write(byte) };
            return Some(unsafe { core::slice::from_raw_parts_mut(out, len) });
        } else {
            unsafe { out.add(i).write(byte | 0b1000_0000) };
        }
        i = len;
    }
    None
}

const fn encode_zigzag_imp<Src: PrimaryInt + CalcFitted>(src: Src) -> Src::Unsigned {
    CInt::cast_as_unsigned(CInt::bitxor(CInt::shl(src, 1), CInt::shr(src, const { Src::BITS - 1 })))
}

const fn decode_zigzag_imp<Src: PrimaryInt + CalcFitted>(src: Src) -> Src::Signed {
    CInt::cast_as_signed(CInt::bitxor(CInt::shr(src, 1), CInt::wrapping_neg(CInt::bitand(src, Src::ONE))))
}

mod imp {
    use core::{mem::MaybeUninit, ops::Deref};

    #[derive(Clone, Copy)]
    pub struct EncodedVarInt<Buf: Copy> {
        len: u8,
        data: MaybeUninit<Buf>,
    }
    impl<Buf: Copy> EncodedVarInt<Buf> {
        pub(super) const fn new() -> Self {
            Self { len: 0, data: MaybeUninit::uninit() }
        }

        pub(super) const fn as_mut(&mut self) -> &mut [u8] {
            unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr().cast::<u8>(), size_of::<Buf>()) }
        }

        pub(super) const fn set_len(&mut self, len: usize) {
            debug_assert!(len <= size_of::<Buf>());
            self.len = len as u8;
        }

        /// get encoded bytes, slice is not empty
        pub const fn as_slice(&self) -> &[u8] {
            unsafe { core::slice::from_raw_parts(self.data.as_ptr().cast::<u8>(), self.len as usize) }
        }
    }
    impl<Buf: Copy> Deref for EncodedVarInt<Buf> {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            self.as_slice()
        }
    }
}

#[cfg(test)]
mod test {
    use super::VarInt;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn varint() {
        for i in u16::MIN..u16::MAX {
            let buf = VarInt::encode_unsigned(i);
            assert_eq!(VarInt::decode_unsigned::<u16>(&buf).unwrap().0, i);
        }
        let data = [3, 106, 97, 100];
        let result = [106, 97, 100];
        assert_eq!(VarInt::decode_slice::<usize>(&data), Ok((result.as_slice(), 4)));
    }

    #[test]
    fn zigzag() {
        assert_eq!(VarInt::encode_zigzag(0i8), 0u8);
        assert_eq!(VarInt::encode_zigzag(-1i8), 1u8);
        assert_eq!(VarInt::encode_zigzag(1i8), 2u8);
        assert_eq!(VarInt::encode_zigzag(-2i8), 3u8);
        assert_eq!(VarInt::encode_zigzag(2i8), 4u8);
        assert_eq!(VarInt::encode_zigzag(-3i8), 5u8);
        assert_eq!(VarInt::encode_zigzag(3i8), 6u8);
        assert_eq!(VarInt::encode_zigzag(-4i8), 7u8);
        assert_eq!(VarInt::encode_zigzag(4i8), 8u8);
        assert_eq!(VarInt::encode_zigzag(-128i8), 255u8);

        for i in i8::MIN..0 {
            assert_eq!(VarInt::encode_zigzag(i), (i.unsigned_abs() - 1) * 2 + 1); // Same as i.unsigned_abs() * 2 - 1, but multiplication overflows for -128
            assert_eq!(VarInt::decode_zigzag(VarInt::encode_zigzag(i)), i);
        }
        for i in 0..=i8::MAX {
            assert_eq!(VarInt::encode_zigzag(i), (i as u8) * 2);
            assert_eq!(VarInt::decode_zigzag(VarInt::encode_zigzag(i)), i);
        }
    }
}
