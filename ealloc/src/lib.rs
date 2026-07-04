#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

use core::{
    alloc::Layout,
    cell::UnsafeCell,
    ptr::{NonNull, copy_nonoverlapping},
};

use ecore::int::PrimaryUInt;

pub use self::{
    block::{ElementCount, simple::SimpleFirstFit},
    dv::DesignatedVictim,
    first_fit::FirstFit,
    half_tree::HalfTree,
    ptr::{FixedBase, FixedBasePtr},
    tlsf::{Tlsf, TlsfParms},
};

mod block;
mod dv;
mod first_fit;
mod half_tree;
mod ptr;
mod tlsf;

/// Error returned when memory allocation fails.
///
/// This is a zero-sized type indicating the allocator could not satisfy the request.
pub struct AllocError;

/// Custom memory allocator trait, similar to [`core::alloc::Allocator`].
///
/// # Safety
///
/// Implementors must ensure memory safety for all allocation and deallocation operations.
pub unsafe trait Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;

    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator with the given `layout` and not yet deallocated.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

    /// Allocates zero-initialized memory with the given layout.
    /// The default implementation calls [`allocate`](Self::allocate) then zeroes the memory.
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut block = self.allocate(layout)?;
        unsafe { block.as_mut().fill(0) };
        Ok(block)
    }

    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator with `old_layout` and not yet deallocated.
    /// `new_layout.size()` must be >= `old_layout.size()`.
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() >= old_layout.size(), "`new_layout.size()` must be greater than or equal to `old_layout.size()`");
        let new_ptr = self.allocate(new_layout)?;
        unsafe { copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr().cast(), old_layout.size()) }
        unsafe { self.deallocate(ptr, old_layout) }
        Ok(new_ptr)
    }

    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator with `old_layout` and not yet deallocated.
    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() >= old_layout.size(), "`new_layout.size()` must be greater than or equal to `old_layout.size()`");
        let new_ptr = self.allocate_zeroed(new_layout)?;
        unsafe { copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr().cast(), old_layout.size()) }
        unsafe { self.deallocate(ptr, old_layout) }
        Ok(new_ptr)
    }

    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator with `old_layout` and not yet deallocated.
    /// `new_layout.size()` must be <= `old_layout.size()`.
    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() <= old_layout.size(), "`new_layout.size()` must be smaller than or equal to `old_layout.size()`");
        let new_ptr = self.allocate(new_layout)?;
        unsafe { copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr().cast(), new_layout.size()) }
        unsafe { self.deallocate(ptr, old_layout) }
        Ok(new_ptr)
    }

    /// Returns a reference to `self`, useful for passing the allocator by reference.
    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}

/// Free memory block manager for [LeanFlexAllocator], don't care about how block be splited or merged
pub trait FreeBlockManager {
    /// Block state element, block state exactually use 2 [Self::StateElement], must ensure `size_of::<Self::StateElement>() <= size_of::<usize>()`,
    /// * when `size_of::<Self::StateElement>() < size_of::<usize>()` the extreme block size is `(Self::StateElement::BITS - 1) * size_of::<Self::Element>`
    /// * when `size_of::<Self::StateElement>() == size_of::<usize>()` the extreme block size is `isize::MAX`
    /// * [Self::MAX_ELEMENT_COUNT] limit the max block size which always <= extreme block size
    type StateElement: PrimaryUInt;
    /// Block element, block underlying is `[Self::Element]`, block align >= `align_of::<Self::Element>`, block size is multiple of `size_of::<Self::Element>()`,
    /// must ensure `size_of::<Self::Element>() % align_of::<Self::Element>() == 0 && align_of::<Self::Element>() >= 2 && size_of::<Self::Element>() != 0`.
    /// bigger size_of::<Self::Element> lead to bigger internal fragmentation, but can manage bigger memory region with same [Self::StateElement]
    type Element;
    /// Free block node, manager store [Self::Node] at free block head, then block size always >= `size_of::<Self::Node>()`
    type Node;

    /// max element count for single region, and is max allocable element count for single allocation,
    /// memory region will be splited (when enxtend or init) if region element count bigger than [Self::MAX_ELEMENT_COUNT]
    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element>;

    /// acceptable address range, default full aligned range, maybe changed after init
    fn address_range(&self) -> (usize, usize);

    /// take out a free block (find and unregister)
    fn take_out(&mut self, element_count: ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>>;

    /// register a node to pool, caller must ensure node is free and in memory region managed by this manager
    ///
    /// # Safety
    ///
    /// `node` must point to a valid, free block within the memory region managed by this manager.
    unsafe fn register(&mut self, node: NonNull<Self::Node>);

    /// unregister a node from pool, caller must ensture node is in this pool
    ///
    /// # Safety
    ///
    /// `node` must be currently registered in this pool.
    unsafe fn unregister(&mut self, node: NonNull<Self::Node>);

    /// init manager, must call once and only once before call any other method, default call register.
    /// caller must ensure block body in address range
    ///
    /// # Safety
    ///
    /// Must be called once and only once. `ptr` must point to a valid block within the address range.
    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        unsafe { self.register(ptr.cast()) };
    }

    /// extend with new anther memory region node, default call register,
    /// caller must ensure block body in address range
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid block within the address range.
    unsafe fn extend(&mut self, ptr: NonNull<Self::Element>) {
        unsafe { self.register(ptr.cast()) };
    }

    /// called by allocator just before block return to user, usually used by monitor, default no-op
    fn after_allocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let _ = (ptr, layout);
    }

    /// called by allocator just before deallocate block, usually used by monitor, default no-op
    fn before_deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let _ = (ptr, layout);
    }

    /// validate memory region state, only used by test after block state changed
    #[cfg(feature = "test-utils")]
    fn validate(&mut self, addr: core::num::NonZero<usize>) {
        let _ = addr;
    }
}

/// A mutex-like lock for synchronizing access to the allocator's internal state.
///
/// Implementations include [`NoopMutex`] for single-threaded use and `std::sync::Mutex`.
pub trait AllocatorMutex {
    type Guard<'g>
    where
        Self: 'g;
    #[must_use = "unlock when drop"]
    fn lock(&self) -> Self::Guard<'_>;
}

/// Memory allocator which seperate memory allocation into two layer:
/// * one for common memory block process, take out from [FreeBlockManager] and allocate (maybe split and register to [FreeBlockManager]) to user,
///   and free from user and register (maybe combine block) to [FreeBlockManager],
///   and other common memory allocate/deallocate process which does't implement free block management
/// * one for [FreeBlockManager], which only care about free blocks
pub struct LeanFlexAllocator<Mutex, Manager: ?Sized + FreeBlockManager> {
    mutex: Mutex,
    raw: UnsafeCell<block::RawAllocator<Manager>>,
}

unsafe impl<Mutex: Sync, Manager: ?Sized + FreeBlockManager> Sync for LeanFlexAllocator<Mutex, Manager> {}

impl<Mutex, Manager: FreeBlockManager> LeanFlexAllocator<Mutex, Manager> {
    pub const fn new(mutex: Mutex, manager: Manager) -> Self {
        Self { mutex, raw: UnsafeCell::new(block::RawAllocator::new(manager)) }
    }
}

impl<Manager: FreeBlockManager> LeanFlexAllocator<NoopMutex, Manager> {
    pub const fn new_without_mutex(manager: Manager) -> Self {
        Self { mutex: NoopMutex::new(), raw: UnsafeCell::new(block::RawAllocator::new(manager)) }
    }
}

/// noop [AllocatorMutex], usually for no thread or inside single thread
pub struct NoopMutex(UnsafeCell<()>);

impl Default for NoopMutex {
    fn default() -> Self {
        Self::new()
    }
}

impl NoopMutex {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(()))
    }
}

impl AllocatorMutex for NoopMutex {
    type Guard<'g>
        = ()
    where
        Self: 'g;

    fn lock(&self) -> Self::Guard<'_> {}
}

#[cfg(any(feature = "std", test))]
impl<T: ?Sized> AllocatorMutex for std::sync::Mutex<T> {
    type Guard<'g>
        = std::sync::MutexGuard<'g, T>
    where
        Self: 'g;

    fn lock(&self) -> Self::Guard<'_> {
        Self::lock(self).unwrap()
    }
}
