use core::{
    mem::{MaybeUninit, transmute_copy},
    ops::{Index, IndexMut},
};

/// `E` or `[E; N]`, is `[E; N]` if `Sized`
pub trait ArrayOrSlice:
    AsRef<[Self::E]> + AsMut<[Self::E]> + Index<usize, Output = Self::E> + IndexMut<usize, Output = Self::E> + sealed::ArrayOrSlice
{
    type E;
    type Map<NE>: ArrayOrSlice<E = NE> + ?Sized;
    const LEN: Option<usize>;
}

/// Any `[E; N]` type, which is a sized counterpart of [`ArrayOrSlice`].
///
/// Always implemented for `[E; N]` where `N` is a const generic.
pub trait Array:
    Sized + AsRef<[Self::E]> + AsMut<[Self::E]> + Index<usize, Output = Self::E> + IndexMut<usize, Output = Self::E> + sealed::ArrayOrSlice
{
    type E;
    type Map<T>: Array<E = T>;
    const LEN: usize;
}

impl<E> ArrayOrSlice for [E] {
    type E = E;
    type Map<NE> = [NE];
    const LEN: Option<usize> = None;
}

impl<E, const N: usize> ArrayOrSlice for [E; N] {
    type E = E;
    type Map<NE> = [NE; N];
    const LEN: Option<usize> = Some(N);
}

impl<E, const N: usize> Array for [E; N] {
    type E = E;
    type Map<T> = [T; N];
    const LEN: usize = N;
}

/// `MaybeUninit<E>` or `MaybeUninit<[E; N]>`
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct MaybeUninitArrayOrSlice<T: ArrayOrSlice + ?Sized>(T::Map<MaybeUninit<T::E>>);

impl<T: ArrayOrSlice> MaybeUninitArrayOrSlice<T>
where
    T::Map<MaybeUninit<T::E>>: Sized,
{
    const ASSERT: () = assert!(size_of::<T>() == size_of::<T::Map<MaybeUninit<T::E>>>() && align_of::<T>() == align_of::<T::Map<MaybeUninit<T::E>>>());

    /// Creates a new `MaybeUninitArrayOrSlice` from an initialized `T`, consuming the original value
    /// and leaving the returned wrapper uninitialized from the borrow checker's perspective.
    pub const fn new(v: T) -> Self {
        let () = Self::ASSERT;
        let out = Self(unsafe { transmute_copy(&v) });
        core::mem::forget(v);
        out
    }

    /// Creates a new `MaybeUninitArrayOrSlice` in an uninitialized state.
    pub const fn uninit() -> Self {
        let () = Self::ASSERT;
        Self(unsafe { transmute_copy(&MaybeUninit::<T>::uninit()) })
    }

    /// Creates a new `MaybeUninitArrayOrSlice` with all bytes zeroed.
    pub const fn zeroed() -> Self {
        let () = Self::ASSERT;
        Self(unsafe { transmute_copy(&MaybeUninit::<T>::zeroed()) })
    }

    /// # Safety
    ///
    /// The caller must ensure the memory has been fully initialized.
    pub const unsafe fn assume_init(self) -> T {
        let out = unsafe { transmute_copy(&self) };
        core::mem::forget(self);
        out
    }

    /// # Safety
    ///
    /// The caller must ensure the memory has been fully initialized.
    pub const unsafe fn assume_init_read(&self) -> T {
        unsafe { self.as_ptr().read() }
    }

    /// # Safety
    ///
    /// The caller must ensure the memory has been fully initialized with a valid `T`.
    pub unsafe fn assume_init_drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_ptr()) }
    }

    /// Writes an initialized `T` into this wrapper, overwriting any previous value.
    /// Returns a mutable reference to the now-initialized value.
    pub const fn write(&mut self, v: T) -> &mut T {
        unsafe { self.as_mut_ptr().write(v) };
        unsafe { self.assume_init_mut() }
    }
}

impl<T: ArrayOrSlice + ?Sized> MaybeUninitArrayOrSlice<T> {
    /// Returns a raw const pointer to the underlying data.
    pub const fn as_ptr(&self) -> *const T {
        unsafe { transmute_copy(&self) }
    }

    /// Returns a raw mutable pointer to the underlying data.
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { transmute_copy(&self) }
    }

    /// Returns a shared reference to the initialised data.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory has been fully initialized.
    pub const unsafe fn assume_init_ref(&self) -> &T {
        unsafe { transmute_copy(&self) }
    }

    /// Returns a mutable reference to the initialised data.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory has been fully initialized.
    pub const unsafe fn assume_init_mut(&mut self) -> &mut T {
        unsafe { transmute_copy(&self) }
    }
}

mod sealed {
    pub trait ArrayOrSlice {}
    impl<E> ArrayOrSlice for [E] {}
    impl<E, const N: usize> ArrayOrSlice for [E; N] {}
}
