use core::{
    fmt::Debug,
    mem::{MaybeUninit, transmute, transmute_copy},
};

use bytemuck::{Pod, TransparentWrapper, Zeroable};
use const_default::ConstDefault;

use crate::{
    Array,
    int::{CInt, PrimaryUInt},
};

pub use self::iter::{EnumDiscriminantsIterator, EnumDiscriminantsIteratorExt, IterEnumDiscriminants};

/// Enum which can map into an array
pub trait MapEnum {
    /// The discriminant type (usually the enum itself).
    type Discriminant: MapEnum<Discriminant = Self::Discriminant, Array = Self::Array>;
    /// The array type used as the storage backend for [`EnumMap`].
    type Array: Array<E = ()>;
    /// Returns an iterator over all discriminant variants.
    fn iter() -> impl ExactSizeIterator<Item = Self::Discriminant>;
    /// Returns the index of this variant as a `usize` offset into the array.
    fn get_index(&self) -> usize;
}

/// [MapEnum] type with can safe transmute into [`Self::NatureInt`], **will deprecated if `const_trait_impl` is stable**,
/// usually should use `#[derive(MapEnum)]` to gen impl
///
/// # Safety
///
/// Implementors must ensure that `Self` can be safely transmuted to `Self::NatureInt`.
pub unsafe trait PlainMapEnum: MapEnum {
    type NatureInt: PrimaryUInt;
}

/// Array which using enum as index
#[repr(transparent)]
pub struct EnumMap<Ids: MapEnum, T>(<Ids::Array as Array>::Map<T>);

impl<Ids: MapEnum, T> EnumMap<Ids, T> {
    /// Create map from array, recomand using `map_new` or `map_enum!`(if require const or move value) instead
    pub const fn from_array(data: <Ids::Array as Array>::Map<T>) -> Self {
        Self(data)
    }

    /// Returns a shared reference to the underlying array.
    pub const fn as_array(&self) -> &<Ids::Array as Array>::Map<T> {
        &self.0
    }

    /// Returns a mutable reference to the underlying array.
    pub const fn as_array_mut(&mut self) -> &mut <Ids::Array as Array>::Map<T> {
        &mut self.0
    }

    /// Consumes the enum map and returns the underlying array.
    pub const fn into_array(self) -> <Ids::Array as Array>::Map<T> {
        let out = unsafe { core::ptr::from_ref(&self.0).read() };
        core::mem::forget(self);
        out
    }

    /// Reinterprets the enum map as being indexed by a different enum type that shares the same underlying array.
    pub const fn remap<TIds: MapEnum<Array = Ids::Array>>(&self) -> &EnumMap<TIds, T> {
        unsafe { transmute(self) }
    }

    /// Mutable version of [`remap`](Self::remap).
    pub const fn remap_mut<TIds: MapEnum<Array = Ids::Array>>(&mut self) -> &mut EnumMap<TIds, T> {
        unsafe { transmute(self) }
    }

    /// Create map by mapping closure, `map` will call excctally once for each enum variant, using `map_enum!` instead if require const or move value
    pub fn map_new(mut map: impl FnMut(Ids::Discriminant) -> T) -> Self {
        let mut out = MaybeUninit::<Self>::uninit();
        let buf = unsafe { out.assume_init_mut() }.0.as_mut();
        for ids in Ids::iter() {
            unsafe { core::ptr::write(buf.get_unchecked_mut(ids.get_index()), map(ids)) };
        }
        unsafe { out.assume_init() }
    }

    /// Creates a new enum map where each element references the corresponding element in `self`.
    pub const fn map_as_ref<'a>(&'a self) -> EnumMap<Ids, &'a T> {
        let mut out = MaybeUninit::<EnumMap<Ids, &'a T>>::uninit();
        let mut src = (&raw const self.0).cast::<T>();
        let mut tgt = (unsafe { &raw mut out.assume_init_mut().0 }).cast::<&'a T>();
        let mut i = 0;
        while i < Ids::Array::LEN {
            unsafe { tgt.write(&*src) };
            i += 1;
            src = unsafe { src.add(1) };
            tgt = unsafe { tgt.add(1) };
        }
        unsafe { out.assume_init() }
    }

    /// Creates a new enum map where each element mutably references the corresponding element in `self`.
    pub const fn map_as_mut<'a>(&'a mut self) -> EnumMap<Ids, &'a mut T> {
        let mut out = MaybeUninit::<EnumMap<Ids, &'a mut T>>::uninit();
        let mut src = (&raw mut self.0).cast::<T>();
        let mut tgt = (unsafe { &raw mut out.assume_init_mut().0 }).cast::<&'a mut T>();
        let mut i = 0;
        while i < Ids::Array::LEN {
            unsafe { tgt.write(&mut *src) };
            i += 1;
            src = unsafe { src.add(1) };
            tgt = unsafe { tgt.add(1) };
        }
        unsafe { out.assume_init() }
    }

    /// Wraps each element of the map in a `TransparentWrapper`, safe because the repr is identical.
    pub const fn transparent_wrap<U: TransparentWrapper<T>>(self) -> EnumMap<Ids, U> {
        let out = unsafe { transmute_copy(&self) };
        core::mem::forget(self);
        out
    }

    /// Unwraps (peels) each element from a `TransparentWrapper`, safe because the repr is identical.
    pub const fn transparent_peel<U>(self) -> EnumMap<Ids, U>
    where
        T: TransparentWrapper<U>,
    {
        let out = unsafe { transmute_copy(&self) };
        core::mem::forget(self);
        out
    }
}

impl<Ids: MapEnum + PlainMapEnum, T> EnumMap<Ids, T> {
    const ASSERT: () = assert!(size_of::<Ids>() == size_of::<Ids::NatureInt>() && size_of::<Ids>() <= size_of::<usize>());
    /// Gets a reference to the element at the index given by the enum variant `index`.
    /// This is a const-compatible version of the `Index` trait.
    pub const fn cget(&self, index: &Ids) -> &T {
        let () = Self::ASSERT;
        let index: Ids::NatureInt = unsafe { transmute_copy(index) };
        unsafe { &*(&raw const self.0 as *const T).add(CInt::cast_as(index)) }
    }

    /// Gets a mutable reference to the element at the index given by the enum variant `index`.
    /// This is a const-compatible version of the `IndexMut` trait.
    pub const fn cget_mut(&mut self, index: &Ids) -> &mut T {
        let () = Self::ASSERT;
        let index: Ids::NatureInt = unsafe { transmute_copy(index) };
        unsafe { &mut *(&raw mut self.0 as *mut T).add(CInt::cast_as(index)) }
    }
}

impl<Ids0: MapEnum<Discriminant: Clone>, Ids1: MapEnum, T> EnumMap<Ids0, EnumMap<Ids1, T>> {
    /// Creates a 2-level enum map by invoking the closure for every combination of `(Ids0, Ids1)` variants.
    pub fn map_new2(mut map: impl FnMut(Ids0::Discriminant, Ids1::Discriminant) -> T) -> Self {
        let mut out = MaybeUninit::<Self>::uninit();
        let buf = unsafe { out.assume_init_mut() }.0.as_mut();
        for ids0 in Ids0::iter() {
            for ids1 in Ids1::iter() {
                let ids0 = ids0.clone();
                unsafe { core::ptr::write(buf.get_unchecked_mut(ids0.get_index()).0.as_mut().get_unchecked_mut(ids1.get_index()), map(ids0, ids1)) };
            }
        }
        unsafe { out.assume_init() }
    }
}

impl<Ids: MapEnum, T> AsRef<<Ids::Array as Array>::Map<T>> for EnumMap<Ids, T> {
    fn as_ref(&self) -> &<Ids::Array as Array>::Map<T> {
        &self.0
    }
}

impl<Ids: MapEnum, T> AsMut<<Ids::Array as Array>::Map<T>> for EnumMap<Ids, T> {
    fn as_mut(&mut self) -> &mut <Ids::Array as Array>::Map<T> {
        &mut self.0
    }
}

impl<Ids: MapEnum, T> core::ops::Index<Ids> for EnumMap<Ids, T> {
    type Output = T;
    fn index(&self, index: Ids) -> &Self::Output {
        &self.0.as_ref()[index.get_index()]
    }
}

impl<Ids: MapEnum, T> core::ops::IndexMut<Ids> for EnumMap<Ids, T> {
    fn index_mut(&mut self, index: Ids) -> &mut Self::Output {
        &mut self.0.as_mut()[index.get_index()]
    }
}

impl<'i, Ids: MapEnum, T> core::ops::Index<&'i Ids> for EnumMap<Ids, T> {
    type Output = T;
    fn index(&self, index: &'i Ids) -> &Self::Output {
        &self.0.as_ref()[index.get_index()]
    }
}

impl<'i, Ids: MapEnum, T> core::ops::IndexMut<&'i Ids> for EnumMap<Ids, T> {
    fn index_mut(&mut self, index: &'i Ids) -> &mut Self::Output {
        &mut self.0.as_mut()[index.get_index()]
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: Debug>>, T> Debug for EnumMap<Ids, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EnumMap<{}>(", core::any::type_name::<Ids>())?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: Copy>>, T> Copy for EnumMap<Ids, T> {}

impl<Ids: MapEnum<Array: Array<Map<T>: Clone>>, T> Clone for EnumMap<Ids, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: Default>>, T> Default for EnumMap<Ids, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: ConstDefault>>, T> ConstDefault for EnumMap<Ids, T> {
    const DEFAULT: Self = Self(ConstDefault::DEFAULT);
}

unsafe impl<Ids: MapEnum<Array: Array<Map<T>: Zeroable>>, T> Zeroable for EnumMap<Ids, T> {}

unsafe impl<Ids: MapEnum<Array: Array<Map<T>: Pod>> + 'static, T: 'static> Pod for EnumMap<Ids, T> {}

unsafe impl<Ids: MapEnum, T: TransparentWrapper<U>, U> TransparentWrapper<EnumMap<Ids, U>> for EnumMap<Ids, T> {}

impl<Ids: MapEnum<Array: Array<Map<T>: Eq>>, T> Eq for EnumMap<Ids, T> {}

impl<Ids: MapEnum<Array: Array<Map<T>: PartialEq>>, T> PartialEq for EnumMap<Ids, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: Ord>>, T> Ord for EnumMap<Ids, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<Ids: MapEnum<Array: Array<Map<T>: PartialOrd>>, T> PartialOrd for EnumMap<Ids, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

mod iter;

#[cfg(test)]
mod test {
    use crate::{MapEnum, PlainMapEnum, int::PrimaryInt};

    #[derive(MapEnum)]
    enum Nature {
        V0,
        V1,
    }

    #[derive(MapEnum)]
    enum Alter {
        V0 = 2,
        V1,
    }

    #[derive(MapEnum)]
    #[discriminant]
    enum Discriminant {
        V0,
        V1(#[allow(dead_code)] char),
    }

    #[derive(MapEnum)]
    #[discriminant]
    enum Generic<T> {
        V0(T),
        V1,
    }

    #[derive(MapEnum)]
    #[discriminant]
    enum GenericWhere<T>
    where
        T: PrimaryInt,
    {
        V0(T),
        V1,
    }

    #[test]
    fn nature() {
        assert_eq!(Nature::V0.get_index(), 0);
        assert_eq!(Nature::V1.get_index(), 1);
    }

    #[test]
    fn alter() {
        assert_eq!(Alter::V0.get_index(), 0);
        assert_eq!(Alter::V1.get_index(), 1);
    }

    #[test]
    fn discriminant() {
        assert_eq!(Discriminant::V0.get_index(), 0);
        assert_eq!(Discriminant::V1('d').get_index(), 1);
    }

    #[test]
    fn generic() {
        assert_eq!(Generic::V0('d').get_index(), 0);
        assert_eq!(Generic::<char>::V1.get_index(), 1);
    }

    #[test]
    fn generic_where() {
        assert_eq!(GenericWhere::V0(3).get_index(), 0);
        assert_eq!(GenericWhere::<u8>::V1.get_index(), 1);
    }
}
