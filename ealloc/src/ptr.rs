use core::{hint::unreachable_unchecked, marker::PhantomData, num::NonZero, ptr::NonNull};

use const_default::ConstDefault;

#[repr(transparent)]
pub(super) struct Ptr<T>(NonNull<T>);

impl<T> Ptr<T> {
    #[inline(always)]
    pub(super) const fn new(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    #[inline(always)]
    pub(super) const fn raw(self) -> NonNull<T> {
        self.0
    }

    #[inline(always)]
    pub(super) const fn cast<U>(self) -> Ptr<U> {
        Ptr(self.0.cast())
    }

    #[inline(always)]
    pub(super) const fn read(self) -> T {
        unsafe { self.0.read() }
    }

    #[inline(always)]
    pub(super) const fn write(self, v: T) {
        unsafe { self.0.write(v) }
    }

    #[inline(always)]
    pub(super) const fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }

    #[inline(always)]
    pub(super) const fn byte_add(self, count: usize) -> Self {
        Self(unsafe { self.0.byte_add(count) })
    }

    #[inline(always)]
    pub(super) const fn byte_sub(self, count: usize) -> Self {
        Self(unsafe { self.0.byte_sub(count) })
    }

    #[inline(always)]
    pub(super) const fn byte_offset_from(self, origin: Self) -> usize {
        unsafe { self.0.byte_offset_from(origin.0) as usize }
    }

    #[inline(always)]
    pub(super) fn map_addr(self, f: impl FnOnce(NonZero<usize>) -> NonZero<usize>) -> Self {
        Self(self.0.map_addr(f))
    }
}

impl<T> Eq for Ptr<T> {}
impl<T> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> From<&mut T> for Ptr<T> {
    fn from(value: &mut T) -> Self {
        Self(NonNull::from(value))
    }
}

impl<T> Copy for Ptr<T> {}
impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Allocator pointer maybe narrowed:
/// * [`NonNull<Element>`] for normal pointer
/// * [`NonZero<uN>`] for narrow relative pointer, only supported when `size_of::<uN>() < size_of::<usize>()`
/// * [`FixedBasePtr<NonZero<uN>>`] for narrow fixed-base pointer, only supported when `size_of::<uN>() < size_of::<usize>()`
pub trait OptimizedPtr<Element>: Copy + Eq {
    /// type of base, must be `()` for fixed base ptr, must be [*mut Element] for relative ptr
    type VirtualBase: Copy + ConstDefault;

    /// create base from first avaiable ptr, noop for fixed base ptr
    fn new_base(ptr: NonNull<Element>) -> Self::VirtualBase;

    /// avaiable address range, maybe different if base is null for relative ptr, but fixed for fixed base ptr
    fn address_range(base: Self::VirtualBase) -> (usize, usize);

    fn new(base: Self::VirtualBase, ptr: NonNull<Element>) -> Self;

    fn get(self, base: Self::VirtualBase) -> NonNull<Element>;
}

impl<Element> OptimizedPtr<Element> for NonNull<Element> {
    type VirtualBase = ();

    #[inline(always)]
    fn new_base(_: NonNull<Element>) -> Self::VirtualBase {}

    #[inline(always)]
    fn address_range(_: Self::VirtualBase) -> (usize, usize) {
        (size_of::<Element>(), const { isize::MAX as usize / size_of::<Element>() * size_of::<Element>() })
    }

    #[inline(always)]
    fn new(_: Self::VirtualBase, ptr: NonNull<Element>) -> Self {
        ptr
    }

    #[inline(always)]
    fn get(self, _: Self::VirtualBase) -> NonNull<Element> {
        self
    }
}

macro_rules! impl_relative {
    ($type:ty) => {
        impl<Element> OptimizedPtr<Element> for NonZero<$type> {
            type VirtualBase = *mut Element;

            #[inline(always)]
            fn new_base(base: NonNull<Element>) -> Self::VirtualBase {
                let base = base.as_ptr().wrapping_sub(1);
                debug_assert!(!base.is_null());
                base
            }

            #[inline(always)]
            fn address_range(base: Self::VirtualBase) -> (usize, usize) {
                if base.is_null() {
                    NonNull::<Element>::address_range(())
                } else {
                    (base.wrapping_add(1).addr(), base.wrapping_add(Self::MAX.get() as usize).addr())
                }
            }

            fn new(base: Self::VirtualBase, ptr: NonNull<Element>) -> Self {
                let bytes = ptr.addr().get().wrapping_sub(base.addr());
                debug_assert!(bytes % size_of::<Element>() == 0);
                let len = bytes / size_of::<Element>();
                let len = len.try_into().unwrap_or_else(|_| unsafe { unreachable_unchecked() });
                NonZero::new(len).unwrap_or_else(|| unsafe { unreachable_unchecked() })
            }

            fn get(self, base: Self::VirtualBase) -> NonNull<Element> {
                let ptr = base.wrapping_add(self.get() as usize);
                NonNull::new(ptr).unwrap_or_else(|| unsafe { unreachable_unchecked() })
            }
        }
    };
}
impl_relative!(u8);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_relative!(u16);

#[cfg(target_pointer_width = "64")]
impl_relative!(u32);

/// Indicates a fixed base address for pointer narrowing.
///
/// Implementors provide a known base pointer via [`null_base`](Self::null_base),
/// allowing relative offsets to be stored in a narrower integer type.
pub trait FixedBase<Element> {
    /// Returns the base pointer. When null, it typically points immediately before the first available element.
    fn null_base() -> *mut Element;
}

/// A narrow pointer stored as an offset from a fixed base address.
///
/// * `Underlying` is typically `NonZero<uN>` storing the element offset.
/// * `Base` implements [`FixedBase`] and provides the absolute base address.
#[repr(transparent)]
pub struct FixedBasePtr<Underlying, Base>(Underlying, PhantomData<Base>);

impl<Underlying: Eq, Base> Eq for FixedBasePtr<Underlying, Base> {}
impl<Underlying: PartialEq, Base> PartialEq for FixedBasePtr<Underlying, Base> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Underlying: Copy, Base> Copy for FixedBasePtr<Underlying, Base> {}
impl<Underlying: Clone, Base> Clone for FixedBasePtr<Underlying, Base> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

macro_rules! impl_fixedbase {
    ($type:ty) => {
        impl<Element, Base: FixedBase<Element>> OptimizedPtr<Element> for FixedBasePtr<NonZero<$type>, Base> {
            type VirtualBase = ();

            fn new_base(_: NonNull<Element>) -> Self::VirtualBase {}

            fn address_range(_: Self::VirtualBase) -> (usize, usize) {
                let base = Base::null_base();
                (base.wrapping_add(1).addr(), base.wrapping_add(<$type>::MAX as usize).addr())
            }

            fn new(_: Self::VirtualBase, ptr: NonNull<Element>) -> Self {
                let count = unsafe { ptr.as_ptr().offset_from(Base::null_base()) };
                let count = count.try_into().unwrap_or_else(|_| unsafe { unreachable_unchecked() });
                let count = NonZero::new(count).unwrap_or_else(|| unsafe { unreachable_unchecked() });
                Self(count, PhantomData)
            }

            fn get(self, _: Self::VirtualBase) -> NonNull<Element> {
                unsafe { NonNull::new_unchecked(Base::null_base().wrapping_add(self.0.get() as usize)) }
            }
        }
    };
}
impl_fixedbase!(u8);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_fixedbase!(u16);

#[cfg(target_pointer_width = "64")]
impl_fixedbase!(u32);
