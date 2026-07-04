use bytemuck::{Pod, Zeroable};
use const_default::ConstDefault;

/// Type which can safe cast to `Self::Raw` but not always can cast from `Self::Raw`
pub trait Uncheckable: Copy {
    type UncheckedRaw: Copy;
    fn raw_value(sf: Self) -> Self::UncheckedRaw;
    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw>;
}

/// [Uncheckable] type which can safe transmute into `Self::UncheckedRaw`, **will deprecated if `const_trait_impl` is stable**,
/// usually should use `#[bitfld(..)]` to gen impl
///
/// # Safety
///
/// Implementors must ensure that transmuting from `Self` to `Self::UncheckedRaw` and back is sound.
pub unsafe trait PlainUncheckable: Uncheckable {}

/// A type that does not check value validity during storage (storage bit width is still guaranteed),
/// but performs validation when the value is retrieved.
#[derive(Debug)]
#[repr(transparent)]
pub struct Unchecked<T: Uncheckable> {
    raw: T::UncheckedRaw,
}

impl<T: Uncheckable> Unchecked<T> {
    /// Returns the raw stored value without validation.
    pub const fn raw_value(self) -> T::UncheckedRaw {
        self.raw
    }

    /// Constructs an `Unchecked<T>` directly from a raw value, without validation.
    pub const fn new_with_raw_value(raw: T::UncheckedRaw) -> Self {
        Self { raw }
    }

    /// Returns a reference to the raw stored value.
    pub const fn raw_ref(&self) -> &T::UncheckedRaw {
        &self.raw
    }

    pub fn get(self) -> Result<T, T::UncheckedRaw> {
        T::try_from_raw_value(self.raw)
    }

    /// # Safety
    ///
    /// The caller must ensure the contained raw value represents a valid `T`.
    pub unsafe fn get_unchecked(self) -> T {
        unsafe { self.get().unwrap_unchecked() }
    }

    pub fn get_or_default(self) -> T
    where
        T: Default,
    {
        self.get().unwrap_or_default()
    }
}

impl<T: Uncheckable<UncheckedRaw: Uncheckable>> Unchecked<Unchecked<T>> {
    /// Validates both layers of `Unchecked` wrapping, returning the inner value or the outermost invalid raw value.
    pub fn flatten_get(self) -> Result<T, <T::UncheckedRaw as Uncheckable>::UncheckedRaw> {
        T::try_from_raw_value(T::UncheckedRaw::try_from_raw_value(self.raw)?).map_err(|_| self.raw)
    }

    /// Unwraps both validation layers without checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure the contained raw value represents a valid `T`.
    pub unsafe fn flatten_get_unchecked(self) -> T {
        unsafe { self.flatten_get().unwrap_unchecked() }
    }

    /// Validates both layers and returns the default value of `T` on failure.
    pub fn flatten_get_or_default(self) -> T
    where
        T: Default,
    {
        self.flatten_get().unwrap_or_default()
    }
}

impl<T: Uncheckable<UncheckedRaw: ConstDefault>> ConstDefault for Unchecked<T> {
    const DEFAULT: Self = Self { raw: ConstDefault::DEFAULT };
}

impl<T: Uncheckable<UncheckedRaw: Default>> Default for Unchecked<T> {
    fn default() -> Self {
        Self { raw: Default::default() }
    }
}

impl<T: Uncheckable> Copy for Unchecked<T> {}
impl<T: Uncheckable> Clone for Unchecked<T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T: Uncheckable<UncheckedRaw: Zeroable>> Zeroable for Unchecked<T> {}
unsafe impl<T: Uncheckable<UncheckedRaw: Pod> + 'static> Pod for Unchecked<T> {}

impl<T: Uncheckable<UncheckedRaw: Uncheckable>> Uncheckable for Unchecked<T> {
    type UncheckedRaw = <T::UncheckedRaw as Uncheckable>::UncheckedRaw;

    fn raw_value(sf: Self) -> Self::UncheckedRaw {
        T::UncheckedRaw::raw_value(sf.raw)
    }

    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
        Ok(Self { raw: T::UncheckedRaw::try_from_raw_value(raw)? })
    }
}
unsafe impl<T: Uncheckable<UncheckedRaw: Uncheckable + PlainUncheckable> + PlainUncheckable> PlainUncheckable for Unchecked<T> {}

impl<T: Uncheckable> From<T> for Unchecked<T> {
    fn from(value: T) -> Self {
        Self { raw: T::raw_value(value) }
    }
}

impl<T: Uncheckable<UncheckedRaw: PartialEq>> PartialEq<Unchecked<T>> for Unchecked<T> {
    fn eq(&self, other: &Unchecked<T>) -> bool {
        self.raw == other.raw
    }
}
impl<T: Uncheckable<UncheckedRaw: Eq>> Eq for Unchecked<T> {}

impl<T: Uncheckable<UncheckedRaw: PartialEq>> PartialEq<T> for Unchecked<T> {
    fn eq(&self, other: &T) -> bool {
        self.raw == T::raw_value(*other)
    }
}

impl Uncheckable for bool {
    type UncheckedRaw = u8;

    fn raw_value(sf: Self) -> Self::UncheckedRaw {
        sf as u8
    }

    fn try_from_raw_value(raw: Self::UncheckedRaw) -> Result<Self, Self::UncheckedRaw> {
        if raw & 1 == raw { Ok(raw != 0) } else { Err(raw) }
    }
}

impl Unchecked<bool> {
    pub const fn masked_value(&self) -> bool {
        self.raw & 1 != 0
    }
}
