use core::{
    marker::PhantomData,
    num::NonZero,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::int::{BasicInt as _, PrimaryUInt};

/// Optimaized bits range for `T` and must not zero bits, [`Option<BitsRange>`] also has optimized repr
#[repr(transparent)]
pub struct BitsRange<T = u128>(NonZero<u16>, PhantomData<T>);

impl<T> BitsRange<T> {
    /// Returns the starting bit position (0-based, inclusive).
    pub const fn start(self) -> u32 {
        self.0.get() as u32 & 0xFF
    }

    /// Returns the number of bits in the range.
    pub const fn bits(self) -> NonZero<u32> {
        unsafe { NonZero::new_unchecked(self.0.get() as u32 >> 8 & 0xFF) }
    }

    /// Returns the ending bit position (exclusive). Equivalent to `start() + bits()`.
    pub const fn end(self) -> u32 {
        self.start() + self.bits().get()
    }

    /// Strip `start` for `cell_bits`, return new bits range and striped groups, `new_start = old_start % cell_bits` and `cells = old_start / cell_bits`
    /// for example, `BitsRange::new(18, 5).unwrap().strip_cells(8) == (BitsRange::new(2, 5).unwrap(), 2)`
    #[inline(always)]
    pub const fn strip_cells(self, cell_bits: u32) -> (Self, u32) {
        let start = self.start();
        (Self(unsafe { NonZero::new_unchecked((start % cell_bits) as u16 | self.0.get() >> 8 << 8) }, PhantomData), start / cell_bits)
    }

    /// Cross cells from bit 0, actually `(start + bits).div_ceil(cell_bits)`
    #[inline(always)]
    pub const fn cross_cells(self, cell_bits: u32) -> u32 {
        (self.start() + self.bits().get()).div_ceil(cell_bits)
    }
}

impl<T: PrimaryUInt> BitsRange<T> {
    const MAX_BITS: u32 = if T::BITS <= u8::MAX as u32 { T::BITS } else { u8::MAX as u32 };

    /// Creates a `BitsRange` from a starting bit position and length in bits.
    /// Returns `None` if `bits == 0` or if `start + bits` exceeds `MAX_BITS` (up to `min(T::BITS, 255)`).
    pub const fn new(start: u32, bits: u32) -> Option<Self> {
        if bits != 0
            && let Some(max_bits) = Self::MAX_BITS.checked_sub(start)
            && bits <= max_bits
        {
            Some(Self(unsafe { NonZero::new_unchecked(start as u16 & 0xFF | (bits as u16) << 8) }, PhantomData))
        } else {
            None
        }
    }

    /// Creates a `BitsRange` from an inclusive range `start..=last`.
    /// Returns `None` if the range is invalid (e.g. `start > last`).
    pub const fn from(range: RangeInclusive<u32>) -> Option<Self> {
        let (start, last) = (*range.start(), *range.end());
        let Some(bits) = last.checked_sub(start) else { return None };
        Self::new(start, bits + 1)
    }

    /// Copy bits with little-endian byte order.
    /// * `self` specify the target bit range `start..start+bits` to copy.
    /// * `src` is the source bit data, valid bit range is `0..bits` (`bits..src.len()*8` is discarded), and it is copied into the target bit range `start..start+bits`.
    /// * `tgt` is the target data, with bit range `0..tgt.len()*8`. Ensure`tgt.len()*8 >= self.end()` otherwise return [Err]
    /// * `T` is the integer type used for bit manipulation. All bit ranges above must fit within the bit width of `T`. Ensure`T::BITS >= self.end()` otherwise return [Err]
    #[allow(clippy::result_unit_err)]
    pub fn copy_le_bits(self, src: &[u8], tgt: &mut [u8]) -> Result<(), ()> {
        let cells = self.cross_cells(u8::BITS) as usize;
        let true = (cells <= size_of::<T>() && cells <= tgt.len()) else { return Err(()) };
        let bytes = self.copy_bits(Self::from_le_bytes(src), Self::from_le_bytes(tgt)).to_le_bytes();
        unsafe { tgt.as_mut_ptr().copy_from_nonoverlapping(bytes.as_ref().as_ptr(), cells) }
        Ok(())
    }

    /// Copy bits with big-endian byte order.
    /// * `self` specify the target bit range `start+bits-1..=start` to copy.
    /// * `src` is the source bit data, valid bit range is `bits-1..=0` (`src.len()*8-1..=bits` is discarded), and it is copied into the target bit range `start+bits-1..=start`.
    /// * `tgt` is the target data, with bit range `tgt.len()*8-1..=0`. Ensure`tgt.len()*8 >= self.end()` otherwise return [Err]
    /// * `T` is the integer type used for bit manipulation. All bit ranges above must fit within the bit width of `T`. Ensure`T::BITS >= self.end()` otherwise return [Err]
    #[allow(clippy::result_unit_err)]
    pub fn copy_be_bits(self, src: &[u8], tgt: &mut [u8]) -> Result<(), ()> {
        let cells = self.cross_cells(u8::BITS) as usize;
        let Some(empty_cells) = tgt.len().checked_sub(cells) else { return Err(()) };
        let Some(empty_bytes) = size_of::<T>().checked_sub(cells) else { return Err(()) };
        let v = self.copy_bits(Self::from_be_bytes(src), Self::from_be_bytes(tgt));
        let bytes = (v << (empty_bytes * u8::BITS as usize)).to_be_bytes();
        unsafe { tgt.as_mut_ptr().add(empty_cells).copy_from_nonoverlapping(bytes.as_ref().as_ptr(), cells) }
        Ok(())
    }

    fn copy_bits(self, src: T, tgt: T) -> T {
        let mask = (T::ALL_ONE >> (T::BITS - self.bits().get())) << self.start();
        tgt & !mask | (src << self.start()) & mask
    }

    fn from_le_bytes(src: &[u8]) -> T {
        let mut v = T::ZERO.to_ne_bytes();
        unsafe { v.as_mut().as_mut_ptr().copy_from_nonoverlapping(src.as_ptr(), src.len().min(size_of::<T>())) };
        T::from_le_bytes(v)
    }

    fn from_be_bytes(src: &[u8]) -> T {
        let mut v = T::ZERO;
        for &src in src {
            v = v << u8::BITS | src.cast_as();
        }
        v
    }
}

impl<T> Copy for BitsRange<T> {}
impl<T> Clone for BitsRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for BitsRange<T> {}
impl<T> PartialEq for BitsRange<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> core::fmt::Debug for BitsRange<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}..{}", self.start(), self.end())
    }
}

/// A range type used for indexing.
#[non_exhaustive]
pub struct IndexRange<Idx> {
    start: Idx,
    len: Idx,
}
impl<Idx: Copy> IndexRange<Idx> {
    pub const fn start(&self) -> Idx {
        self.start
    }
    pub const fn len(&self) -> Idx {
        self.len
    }
}

pub trait GetIndexRange<Idx> {
    fn get_index_range(src: &Self, len: Idx) -> IndexRange<Idx>;
}

macro_rules! impl_get_index_range {
    ($unsigned:ty, $signed:ty, $normal_id:ident) => {
        impl GetIndexRange<$unsigned> for RangeFull {
            fn get_index_range(_: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                IndexRange { start: 0, len }
            }
        }

        impl GetIndexRange<$unsigned> for $unsigned {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if *src < len { IndexRange { start: *src, len: 1 } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        impl GetIndexRange<$unsigned> for ($unsigned, $unsigned) {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.0 < len && src.1 <= len - src.0 { IndexRange { start: src.0, len: src.1 } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        impl GetIndexRange<$unsigned> for Range<$unsigned> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.start < src.end && src.end <= len { IndexRange { start: src.start, len: src.end - src.start } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        impl GetIndexRange<$unsigned> for RangeFrom<$unsigned> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.start < len { IndexRange { start: src.start, len: len - src.start } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        impl GetIndexRange<$unsigned> for RangeInclusive<$unsigned> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.start() <= src.end() && *src.end() < len {
                    IndexRange { start: *src.start(), len: src.end() - src.start() + 1 }
                } else {
                    IndexRange { start: 0, len: 0 }
                }
            }
        }

        impl GetIndexRange<$unsigned> for RangeTo<$unsigned> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.end <= len { IndexRange { start: 0, len: src.end } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        impl GetIndexRange<$unsigned> for RangeToInclusive<$unsigned> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                if src.end < len { IndexRange { start: 0, len: src.end + 1 } } else { IndexRange { start: 0, len: 0 } }
            }
        }

        fn $normal_id(src: $signed, len: $unsigned) -> $unsigned {
            if src < 0 { len.wrapping_add_signed(src) } else { src as $unsigned }
        }

        impl GetIndexRange<$unsigned> for $signed {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&($normal_id(*src, len)), len)
            }
        }

        impl GetIndexRange<$unsigned> for ($signed, $unsigned) {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&($normal_id(src.0, len), src.1), len)
            }
        }

        impl GetIndexRange<$unsigned> for Range<$signed> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&($normal_id(src.start, len)..$normal_id(src.end, len)), len)
            }
        }

        impl GetIndexRange<$unsigned> for RangeFrom<$signed> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&($normal_id(src.start, len)..), len)
            }
        }

        impl GetIndexRange<$unsigned> for RangeInclusive<$signed> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&($normal_id(*src.start(), len)..=$normal_id(*src.end(), len)), len)
            }
        }

        impl GetIndexRange<$unsigned> for RangeTo<$signed> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&(..$normal_id(src.end, len)), len)
            }
        }

        impl GetIndexRange<$unsigned> for RangeToInclusive<$signed> {
            fn get_index_range(src: &Self, len: $unsigned) -> IndexRange<$unsigned> {
                GetIndexRange::get_index_range(&(..=$normal_id(src.end, len)), len)
            }
        }
    };
}

impl_get_index_range!(usize, isize, normal_id_usize);
impl_get_index_range!(u32, i32, normal_id_u32);
