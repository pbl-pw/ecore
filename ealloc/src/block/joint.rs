use core::{
    alloc::Layout,
    hint::assert_unchecked,
    marker::PhantomData,
    mem::offset_of,
    ops::{Add, Sub},
    ptr::NonNull,
};

use ecore::int::{CInt, PrimaryUInt};

use crate::ptr::Ptr;

use super::{Block, assert_element_type};

/// The count of elements in a memory block, stored in a compact integer type.
///
/// * When `size_of::<StateElement>() == size_of::<usize>()`, the underlying value represents **bytes**.
/// * Otherwise, the underlying value represents **element count**.
#[repr(transparent)]
pub struct ElementCount<StateElement, Element>(StateElement, PhantomData<Element>);

impl<StateElement: Copy, Element> Copy for ElementCount<StateElement, Element> {}
impl<StateElement: Clone, Element> Clone for ElementCount<StateElement, Element> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<StateElement: Eq, Element> Eq for ElementCount<StateElement, Element> {}
impl<StateElement: PartialEq, Element> PartialEq for ElementCount<StateElement, Element> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<StateElement: Ord, Element> Ord for ElementCount<StateElement, Element> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<StateElement: PartialOrd, Element> PartialOrd for ElementCount<StateElement, Element> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<StateElement: PrimaryUInt, Element> ElementCount<StateElement, Element> {
    pub const IS_USIZE: bool = {
        assert!(size_of::<StateElement>() <= size_of::<usize>());
        size_of::<StateElement>() == size_of::<usize>()
    };

    /// Zero element count.
    pub const ZERO: Self = Self(StateElement::ZERO, PhantomData);

    const MAX_UNDERLYING: usize = if Self::IS_USIZE { usize::MAX >> 1 } else { CInt::cast_as(CInt::shr(StateElement::MAX, 1u32)) };

    /// Maximum possible element count.
    pub const MAX: Self = {
        if Self::IS_USIZE {
            Self(CInt::cast_as((usize::MAX >> 1) / size_of::<Element>() * size_of::<Element>()), PhantomData)
        } else {
            Self(CInt::shr(StateElement::MAX, 1u32), PhantomData)
        }
    };

    /// Maximum element count when `BITS` bits out of `StateElement::BITS` are reserved.
    pub const fn max_for_bits<const BITS: usize>() -> Self {
        assert_element_type::<Element>();
        const {
            assert!((BITS as u32) < StateElement::BITS);
            let remain = StateElement::BITS - BITS as u32;
            if Self::IS_USIZE {
                Self(CInt::cast_as((usize::MAX >> 1) / size_of::<Element>() * size_of::<Element>()), PhantomData)
            } else {
                Self(CInt::shr(StateElement::MAX, remain), PhantomData)
            }
        }
    }

    /// Maximum element count for the given number of elements, clamped to [`MAX`](Self::MAX).
    pub const fn max_for_count(count: usize) -> Self {
        assert_element_type::<Element>();
        if count >= Self::MAX.to_count() {
            Self::MAX
        } else if Self::IS_USIZE {
            Self(CInt::cast_as(count * size_of::<Element>()), PhantomData)
        } else {
            Self(CInt::cast_as(count), PhantomData)
        }
    }

    /// Maximum element count for the given number of bytes.
    pub const fn max_for_bytes(bytes: usize) -> Self {
        Self::max_for_count(bytes / size_of::<Element>())
    }

    /// Largest element count whose actual size in bytes does not exceed `bytes`.
    pub const fn from_bytes_floor(bytes: usize) -> Option<Self> {
        assert_element_type::<Element>();
        let max = const { Self::MAX.to_count() };
        let count = bytes / size_of::<Element>();
        if Self::IS_USIZE {
            if count <= max { Some(Self(CInt::cast_as(count * size_of::<Element>()), PhantomData)) } else { None }
        } else {
            if count <= max { Some(Self(CInt::cast_as(count), PhantomData)) } else { None }
        }
    }

    /// min count which actual size not less than bytes
    const fn from_bytes_ceil(bytes: usize) -> Option<Self> {
        assert_element_type::<Element>();
        let max = const { Self::MAX.to_bytes() };
        if Self::IS_USIZE {
            if bytes <= max { Some(Self(CInt::cast_as(bytes.next_multiple_of(size_of::<Element>())), PhantomData)) } else { None }
        } else {
            if bytes <= max { Some(Self(CInt::cast_as(bytes.div_ceil(size_of::<Element>())), PhantomData)) } else { None }
        }
    }

    /// Minimum element count needed to hold a struct of type `Node`.
    #[inline(always)]
    pub const fn min_for<Node>() -> Self {
        const {
            assert!(align_of::<Node>() <= align_of::<Element>());
            Self::raw_calc(Layout::new::<Node>()).unwrap()
        }
    }

    /// Element count required for a given layout, clamped to at least `min_for::<Node>()`.
    #[inline(always)]
    pub const fn for_layout<Node>(layout: Layout) -> Option<Self> {
        Self::min_for::<Node>().sat_layout(layout)
    }

    /// Returns `true` if the element count is zero.
    pub const fn is_zero(self) -> bool {
        CInt::is_zero(self.0)
    }

    /// Returns the underlying raw value: bytes if `IS_USIZE`, element count otherwise.
    pub const fn underlying(self) -> usize {
        CInt::cast_as(self.0)
    }

    /// Converts the element count to bytes.
    pub const fn to_bytes(self) -> usize {
        if Self::IS_USIZE { self.underlying() } else { self.underlying() * size_of::<Element>() }
    }

    /// Converts the element count to the number of elements.
    pub const fn to_count(self) -> usize {
        if Self::IS_USIZE { self.underlying() / size_of::<Element>() } else { self.underlying() }
    }

    /// Returns the number of bytes available to the user (total bytes minus joint overhead).
    pub const fn usable_bytes_len(self) -> usize {
        self.to_bytes() - Joint::<StateElement, Element>::SIZE
    }

    /// Reads the element count from the header of a memory block.
    pub fn get_from_block(block: NonNull<u8>) -> Self {
        assert_element_type::<Element>();
        Ptr::<Block<StateElement, Element>>::new(block.cast()).head_state().element_count()
    }

    /// Calculate the minimum required [`ElementCount`] for a layout.
    const fn raw_calc(layout: Layout) -> Option<Self> {
        let size = layout.size();
        let align = layout.align();
        let align = if align <= Joint::<StateElement, Element>::ALIGN { Joint::<StateElement, Element>::ALIGN } else { align };
        unsafe { assert_unchecked(align.is_power_of_two()) };
        ElementCount::from_bytes_ceil((size + Joint::<StateElement, Element>::SIZE).next_multiple_of(align))
    }

    /// ensure count of layout not less than self
    const fn sat_layout(self, layout: Layout) -> Option<Self> {
        let Some(count) = Self::raw_calc(layout) else { return None };
        Some(if count.underlying() <= self.underlying() { self } else { count })
    }

    /// Adds two element counts together (const-friendly).
    pub const fn add(self, rhs: Self) -> Self {
        debug_assert!(self.underlying() + rhs.underlying() <= Self::MAX_UNDERLYING);
        Self(CInt::add(self.0, rhs.0), PhantomData)
    }
}

impl<StateElement: PrimaryUInt, Element> Add for ElementCount<StateElement, Element> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.underlying() + rhs.underlying() <= Self::MAX_UNDERLYING);
        Self(self.0 + rhs.0, PhantomData)
    }
}

impl<StateElement: PrimaryUInt, Element> Sub for ElementCount<StateElement, Element> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, PhantomData)
    }
}

#[repr(transparent)]
pub(super) struct State<StateElement, Element>(StateElement, PhantomData<Element>); // trick: bit0 using as is free flag, 0=used, 1=free

impl<StateElement: Copy, Element> Copy for State<StateElement, Element> {}
impl<StateElement: Clone, Element> Clone for State<StateElement, Element> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<StateElement: Eq, Element> Eq for State<StateElement, Element> {}
impl<StateElement: PartialEq, Element> PartialEq for State<StateElement, Element> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<StateElement: PrimaryUInt, Element> State<StateElement, Element> {
    pub(super) fn new(element_count: ElementCount<StateElement, Element>, free: bool) -> Self {
        if ElementCount::<StateElement, Element>::IS_USIZE {
            Self(element_count.0 | if free { StateElement::ONE } else { StateElement::ZERO }, PhantomData)
        } else {
            Self(element_count.0 << 1u32 | if free { StateElement::ONE } else { StateElement::ZERO }, PhantomData)
        }
    }

    pub(super) fn element_count(self) -> ElementCount<StateElement, Element> {
        if ElementCount::<StateElement, Element>::IS_USIZE {
            ElementCount(self.0 & const { CInt::not(StateElement::ONE) }, PhantomData)
        } else {
            ElementCount(self.0 >> 1u32, PhantomData)
        }
    }

    pub(super) fn is_free(self) -> bool {
        self.0 & StateElement::ONE != StateElement::ZERO
    }
}
impl<StateElement: PrimaryUInt, Element> Ptr<State<StateElement, Element>> {
    #[inline(always)]
    pub(super) fn element_count(self) -> ElementCount<StateElement, Element> {
        self.read().element_count()
    }

    #[inline(always)]
    pub(super) fn is_free(self) -> bool {
        self.read().is_free()
    }

    #[inline(always)]
    pub(super) fn bytes_len(self) -> usize {
        self.element_count().to_bytes()
    }

    /// user usable bytes len, is_free must false
    #[inline(always)]
    pub(super) fn usable_bytes_len(self) -> usize {
        if ElementCount::<StateElement, Element>::IS_USIZE {
            // optimize trick: when block is used, bit0 is 0 and count raw is bytes, no mask and no shift required
            CInt::cast_as::<_, usize>(self.read().0) - Joint::<StateElement, Element>::SIZE
        } else {
            // optimize trick: when block is used, bit0 is 0, no mask required, but count is doubled
            CInt::cast_as::<_, usize>(self.read().0) * const { size_of::<Element>() / 2 } - Joint::<StateElement, Element>::SIZE
        }
    }
}

#[repr(C)]
pub(super) struct Joint<StateElement, Element> {
    pub(super) low_state: State<StateElement, Element>,
    pub(super) high_state: State<StateElement, Element>,
}
impl<StateElement, Element> Joint<StateElement, Element> {
    pub(super) const SIZE: usize = size_of::<Self>();
    pub(super) const ALIGN: usize = align_of::<Self>();
}
impl<StateElement, Element> Ptr<Joint<StateElement, Element>> {
    #[inline(always)]
    pub(super) fn low_state(self) -> Ptr<State<StateElement, Element>> {
        const { assert!(offset_of!(Joint<StateElement, Element>, low_state) == 0) }
        self.cast()
    }
}
