use core::{
    alloc::{GlobalAlloc, Layout},
    hint::{assert_unchecked, unreachable_unchecked},
    marker::PhantomData,
    mem::{offset_of, transmute},
    num::NonZero,
    ops::{Range, Rem},
    ptr::NonNull,
};

use ecore::int::PrimaryUInt;

use super::{AllocError, Allocator, AllocatorMutex, FreeBlockManager, LeanFlexAllocator, ptr::Ptr};

pub use self::joint::ElementCount;
use self::joint::{Joint, State};

mod joint;

const fn assert_element_type<Element>() {
    const {
        assert!(size_of::<Element>().is_multiple_of(align_of::<Element>()) && align_of::<Element>() >= 2 && size_of::<Element>() != 0);
    }
}

#[repr(C, packed)]
struct VirtualLayout<StateElement, Element> {
    joint: Joint<StateElement, Element>,
    block: Block<StateElement, Element>,
}

#[repr(transparent)]
struct Block<StateElement, Element> {
    elements: [Element; 0], // make thin pointer by using array instead of slice, actually is [Element]
    _mark: PhantomData<StateElement>,
}

impl<StateElement: PrimaryUInt, Element> Ptr<Block<StateElement, Element>> {
    const ELEMENT_ALIGN: usize = {
        assert_element_type::<Element>();
        align_of::<Element>()
    };

    const HEAD_OFFSET: usize = size_of::<State<StateElement, Element>>();
    const JOINT_OFFSET: usize = {
        assert!(
            offset_of!(VirtualLayout<StateElement,Element>, block) == Joint::<StateElement, Element>::SIZE
                && Joint::<StateElement, Element>::ALIGN <= Self::ELEMENT_ALIGN
                && Joint::<StateElement, Element>::SIZE == 2 * Self::HEAD_OFFSET
        );
        Joint::<StateElement, Element>::SIZE
    };

    fn head_state(self) -> Ptr<State<StateElement, Element>> {
        self.byte_sub(Self::HEAD_OFFSET).cast()
    }

    fn low_joint(self) -> Ptr<Joint<StateElement, Element>> {
        self.byte_sub(Self::JOINT_OFFSET).cast()
    }

    fn lower_block(self) -> Self {
        self.byte_sub(self.low_joint().low_state().bytes_len()).cast()
    }

    fn higher_block(self) -> Self {
        self.byte_add(self.head_state().bytes_len()).cast()
    }

    fn high_joint(self) -> Ptr<Joint<StateElement, Element>> {
        self.higher_block().low_joint()
    }

    fn tail_state(self) -> Ptr<State<StateElement, Element>> {
        self.high_joint().low_state()
    }

    fn element_count(self) -> ElementCount<StateElement, Element> {
        self.head_state().element_count()
    }

    /// block must mark used, because optimize trick
    fn user_block(block: Option<Self>) -> Result<NonNull<[u8]>, AllocError> {
        let Some(block) = block else { return Err(AllocError) };
        Ok(NonNull::slice_from_raw_parts(block.raw().cast(), block.head_state().usable_bytes_len()))
    }

    fn user_ptr(block: Option<Self>) -> *mut u8 {
        unsafe { transmute(block) }
    }

    #[inline(always)]
    fn node<Node>(self) -> NonNull<Node> {
        self.raw().cast()
    }
}

type BlockPtr<Manager> = Ptr<Block<<Manager as FreeBlockManager>::StateElement, <Manager as FreeBlockManager>::Element>>;

#[repr(transparent)]
pub struct RawAllocator<Manager: ?Sized + FreeBlockManager> {
    #[cfg(not(feature = "test-utils"))]
    manager: Manager,
    #[cfg(feature = "test-utils")]
    manager: validator::Validator<Manager>,
}

impl<Manager: FreeBlockManager> RawAllocator<Manager> {
    pub const fn new(manager: Manager) -> Self {
        #[cfg(not(feature = "test-utils"))]
        let out = Self { manager };
        #[cfg(feature = "test-utils")]
        let out = Self { manager: validator::Validator::new(manager) };
        out
    }
}

impl<StateElement: PrimaryUInt, Element, Manager: FreeBlockManager<StateElement = StateElement, Element = Element> + ?Sized> RawAllocator<Manager> {
    const ELEMENT_ALIGN: usize = BlockPtr::<Manager>::ELEMENT_ALIGN;
    const MIN_ELEMENT_COUNT: ElementCount<StateElement, Element> = ElementCount::min_for::<Manager::Node>();

    /// align > Self::ELEMENT_SA
    const fn gen_over_count(layout: Layout) -> Option<OveralignParms<StateElement, Element>> {
        let Some(request_count) = ElementCount::for_layout::<Manager::Node>(layout) else { return None };
        let align = layout.align();
        unsafe { assert_unchecked(align.is_multiple_of(Self::ELEMENT_ALIGN)) };
        let Some(extra) = ElementCount::from_bytes_floor(align) else { return None };
        let over_count = request_count.add(Self::MIN_ELEMENT_COUNT).add(extra);
        let align = unsafe { NonZero::new_unchecked(align) };
        Some(OveralignParms { over_count, request_count, align })
    }

    #[inline(always)]
    const fn raw_block(node: NonNull<Manager::Node>) -> BlockPtr<Manager> {
        Ptr::new(node.cast())
    }

    /// find and unregister free block for pool
    fn take_out_block(&mut self, element_count: ElementCount<StateElement, Element>) -> Option<BlockPtr<Manager>> {
        let block = self.manager.take_out(element_count).map(Self::raw_block)?;
        debug_assert!(block.element_count() >= element_count);
        Some(block)
    }

    fn bound_region(ptr: NonNull<[u8]>, range: (usize, usize)) -> NonNull<[u8]> {
        let (min, max) = range;
        let begin = ptr.as_ptr().cast::<u8>().map_addr(|addr| addr.max(min));
        let end = ptr.as_ptr().cast::<u8>().wrapping_add(ptr.len()).map_addr(|addr| addr.min(max));
        NonNull::slice_from_raw_parts(unsafe { NonNull::new_unchecked(begin) }, end.addr().saturating_sub(begin.addr()))
    }

    /// init memory region as allocatable block
    fn init_region(ptr: NonNull<[u8]>) -> Result<(BlockPtr<Manager>, NonNull<[u8]>), ()> {
        let block = ptr
            .cast::<u8>()
            .map_addr(|addr| unsafe { NonZero::new_unchecked((addr.get() + Joint::<StateElement, Element>::SIZE).next_multiple_of(Self::ELEMENT_ALIGN)) });
        let Some(element_count) = ptr.len().checked_sub(unsafe { block.offset_from(ptr.cast()) as usize }).map(ElementCount::max_for_bytes) else {
            return Err(());
        };
        let element_count = if element_count >= Manager::MAX_ELEMENT_COUNT { Manager::MAX_ELEMENT_COUNT } else { element_count };
        let true = element_count >= Self::MIN_ELEMENT_COUNT else { return Err(()) };
        let not_free = State::new(ElementCount::ZERO, false);
        let is_free = State::new(element_count, true);
        let block: BlockPtr<Manager> = Ptr::new(block.cast());
        *block.low_joint().as_mut() = Joint { low_state: not_free, high_state: is_free };
        *block.high_joint().as_mut() = Joint { low_state: is_free, high_state: not_free };
        let used = block.higher_block().raw().addr().get() - ptr.addr().get();
        let remain = ptr.len() - used;
        Ok((block, NonNull::slice_from_raw_parts(unsafe { ptr.cast::<u8>().add(used) }, remain)))
    }

    /// Extend new allocatable region, return `(added_blocks_num, used_memory_range)` and if returns `(0, _)` means failed
    unsafe fn extend(&mut self, ptr: NonNull<[u8]>) -> (usize, Range<NonNull<u8>>) {
        let mut remain = RawAllocator::<Manager>::bound_region(ptr, self.manager.address_range());
        let mut start = remain.cast();
        let mut extended = 0;
        while let Ok((block, next_remain)) = RawAllocator::<Manager>::init_region(remain) {
            if extended == 0 {
                start = block.low_joint().raw().cast();
            }
            unsafe { self.manager.extend(block.node()) };
            remain = next_remain;
            extended += 1;
        }
        (extended, start..remain.cast())
    }

    /// use unregistered free block as allocated block, maybe spilt and register new block
    fn use_block(&mut self, block: BlockPtr<Manager>, element_count: ElementCount<StateElement, Element>) -> BlockPtr<Manager> {
        let mut head_state = block.head_state();
        debug_assert!(head_state.element_count() >= element_count);
        let higher_element_count = head_state.element_count() - element_count;
        if higher_element_count >= Self::MIN_ELEMENT_COUNT {
            let state = State::new(element_count, false);
            let higher_state = State::new(higher_element_count, true);
            *head_state.as_mut() = state;
            *block.high_joint().as_mut() = Joint { low_state: state, high_state: higher_state };
            let higher_block = block.higher_block();
            *higher_block.tail_state().as_mut() = higher_state;
            unsafe { self.manager.register(higher_block.node()) };
            block
        } else {
            let state = State::new(head_state.element_count(), false);
            *head_state.as_mut() = state;
            *block.tail_state().as_mut() = state;
            block
        }
    }

    /// align <= Self::ELEMENT_SA
    fn raw_allocate(&mut self, element_count: ElementCount<StateElement, Element>) -> Option<BlockPtr<Manager>> {
        let block = self.take_out_block(element_count)?;
        Some(self.use_block(block, element_count))
    }

    /// align > Self::ELEMENT_SA
    fn over_align_alloc(&mut self, parms: OveralignParms<StateElement, Element>) -> Option<BlockPtr<Manager>> {
        let block = self.take_out_block(parms.over_count)?;
        let mid_block = block.map_addr(|addr| unsafe {
            let align = parms.align.get();
            assert_unchecked(align.is_power_of_two());
            NonZero::new_unchecked((addr.get() + Self::MIN_ELEMENT_COUNT.to_bytes()).next_multiple_of(align))
        });
        let offset = mid_block.byte_offset_from(block);
        unsafe { assert_unchecked(offset % Self::ELEMENT_ALIGN == 0) };
        let low_count = unsafe { ElementCount::from_bytes_floor(offset).unwrap_unchecked() };
        let mut head_state = block.head_state();
        let mid_state = State::new(head_state.element_count() - low_count, false);
        let low_state = State::new(low_count, true);
        *head_state.as_mut() = low_state;
        *mid_block.low_joint().as_mut() = Joint { low_state, high_state: mid_state };
        // *mid_block.tail_state().as_mut() = mid_state; // `use_block` will fill tail
        let out = self.use_block(mid_block, parms.request_count);
        unsafe { self.manager.register(block.node()) };
        Some(out)
    }

    /// Try reallocate allocated block
    fn try_realloc(&mut self, block: BlockPtr<Manager>, element_count: ElementCount<StateElement, Element>) -> Result<BlockPtr<Manager>, ()> {
        let mut head_state = block.head_state();
        let true = element_count != head_state.element_count() else { return Ok(block) };
        let higher_block = block.higher_block();
        let higher_state = higher_block.head_state().read();
        if higher_state.is_free() {
            let max_count = head_state.element_count() + higher_state.element_count();
            let true = element_count <= max_count else { return Err(()) };
            unsafe { self.manager.unregister(higher_block.node()) };
            let state = State::new(max_count, false);
            *head_state.as_mut() = state;
            // *block.tail_state().as_mut() = state; // `use_block` will fill tail
        } else if element_count > head_state.element_count() {
            return Err(());
        }
        Ok(self.use_block(block, element_count))
    }

    /// De-allocate memory, data ptr must alllocated by this allocator
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>) {
        let block = Self::raw_block(ptr.cast());
        let count = block.element_count();
        let count = {
            let higher_block = block.higher_block();
            let higher_state = higher_block.head_state().read();
            if higher_state.is_free() {
                unsafe { self.manager.unregister(higher_block.node()) };
                count + higher_state.element_count()
            } else {
                count
            }
        };
        let (block, count) = {
            let lower_state = block.low_joint().low_state();
            if lower_state.is_free() {
                let lower_block = block.lower_block();
                unsafe { self.manager.unregister(lower_block.node()) };
                (lower_block, count + lower_state.element_count())
            } else {
                (block, count)
            }
        };
        let state = State::new(count, true);
        *block.head_state().as_mut() = state;
        *block.tail_state().as_mut() = state;
        unsafe { self.manager.register(block.node()) };
    }

    #[inline(always)]
    fn validate(&mut self, #[allow(unused)] addr: NonZero<usize>) {
        #[cfg(feature = "test-utils")]
        self.manager.validate(addr);
    }
}

impl<Mutex: AllocatorMutex, StateElement: PrimaryUInt, Element, Manager: FreeBlockManager<StateElement = StateElement, Element = Element> + ?Sized>
    LeanFlexAllocator<Mutex, Manager>
{
    #[allow(clippy::mut_from_ref)]
    const fn raw<'g>(&self, _: &'g Mutex::Guard<'_>) -> &'g mut RawAllocator<Manager> {
        unsafe { &mut *self.raw.get() }
    }

    /// Try reallocate allocated block
    fn try_realloc(&self, block: BlockPtr<Manager>, layout: Layout) -> Result<BlockPtr<Manager>, ()> {
        let align = layout.align();
        unsafe { assert_unchecked(align.is_power_of_two()) };
        let true = block.raw().addr().get().rem(layout.align()) == 0 else { return Err(()) };
        let Some(element_count) = ElementCount::for_layout::<Manager::Node>(layout) else { return Err(()) };
        let guard = self.mutex.lock();
        let ptr = self.raw(&guard).try_realloc(block, element_count)?;
        self.raw(&guard).manager.after_allocate(ptr.raw().cast(), layout);
        self.raw(&guard).validate(ptr.raw().addr());
        Ok(ptr)
    }

    /// Init allocator, must call `init`(or `init_with_..`) once and only once before call any other method
    /// * only init first avaiable block, if memory len larger than max block size, use `owned_extend` to extend tail unused memory after init
    /// * return tail unused memory ptr
    /// # Safety
    ///
    /// Must be called once and only once before any other method. `ptr` must point to valid,
    /// properly aligned memory within the manager's address range.
    pub unsafe fn init(&mut self, ptr: NonNull<[u8]>) -> Result<Option<NonNull<[u8]>>, MemoryRegionInitError> {
        let remain = RawAllocator::<Manager>::bound_region(ptr, self.raw.get_mut().manager.address_range());
        let Ok((block, remain)) = RawAllocator::<Manager>::init_region(remain) else { return Err(MemoryRegionInitError) };
        unsafe { self.raw.get_mut().manager.init(block.node()) };
        Ok(if remain.is_empty() { None } else { Some(remain) })
    }

    /// Init allocator with a range region, alias for `init`, must call `init`(or `init_with_..`) once and only once before call any other method
    /// * only init first avaiable block, if memory len larger than max block size, use `owned_extend` to extend tail unused memory after init
    /// * return tail unused memory ptr
    /// # Safety
    ///
    /// See [`init`](Self::init). `range` must be within valid memory.
    #[inline(always)]
    pub unsafe fn init_with_range<T: Copy>(&mut self, range: Range<NonNull<T>>) -> Result<Option<NonNull<[u8]>>, MemoryRegionInitError> {
        let len = unsafe { range.end.byte_offset_from_unsigned(range.start) };
        unsafe { self.init(NonNull::slice_from_raw_parts(range.start.cast::<u8>(), len)) }
    }

    /// Init allocator with a slice region, alias for `init`, must call `init`(or `init_with_..`) once and only once before call any other method
    /// * only init first avaiable block, if memory len larger than max block size, use `owned_extend` to extend tail unused memory after init
    /// * return tail unused memory ptr
    /// # Safety
    ///
    /// See [`init`](Self::init). `slc` must point to valid, properly aligned memory.
    #[inline(always)]
    pub unsafe fn init_with_slice<T: Copy>(&mut self, slc: &mut [T]) -> Result<Option<NonNull<[u8]>>, MemoryRegionInitError> {
        let len = size_of_val(slc);
        unsafe { self.init(NonNull::slice_from_raw_parts(NonNull::from(slc).cast::<u8>(), len)) }
    }

    /// Extend new allocatable region, return `(added_blocks_num, used_memory_range)` and if returns `(0, _)` means failed
    /// # Safety
    ///
    /// `ptr` must point to valid memory within the allocator's address range.
    pub unsafe fn owned_extend(&mut self, ptr: NonNull<[u8]>) -> (usize, Range<NonNull<u8>>) {
        unsafe { self.raw.get_mut().extend(ptr) }
    }

    /// Extend new allocatable region, return `(added_blocks_num, used_memory_range)` and if returns `(0, _)` means failed
    /// # Safety
    ///
    /// `ptr` must point to valid memory within the allocator's address range.
    pub unsafe fn extend(&self, ptr: NonNull<[u8]>) -> (usize, Range<NonNull<u8>>) {
        unsafe { self.raw(&self.mutex.lock()).extend(ptr) }
    }

    /// Extend new allocatable slice region, return `(added_blocks_num, used_memory_range)` and if returns `(0, _)` means failed
    /// # Safety
    ///
    /// `slc` must point to valid memory within the allocator's address range.
    #[inline(always)]
    pub unsafe fn extend_with_slice<T: Copy>(&self, slc: &mut [T]) -> (usize, Range<NonNull<u8>>) {
        let len = core::mem::size_of_val(slc);
        unsafe { self.extend(NonNull::slice_from_raw_parts(NonNull::from(slc).cast::<u8>(), len)) }
    }

    /// Allocate memory
    fn allocate(&self, layout: Layout) -> Option<BlockPtr<Manager>> {
        let (ptr, guard) = if layout.align() <= RawAllocator::<Manager>::ELEMENT_ALIGN {
            let count = ElementCount::for_layout::<Manager::Node>(layout)?;
            let guard = self.mutex.lock();
            (self.raw(&guard).raw_allocate(count)?, guard)
        } else {
            let parms = RawAllocator::<Manager>::gen_over_count(layout)?;
            let guard = self.mutex.lock();
            (self.raw(&guard).over_align_alloc(parms)?, guard)
        };
        self.raw(&guard).manager.after_allocate(ptr.raw().cast(), layout);
        self.raw(&guard).validate(ptr.raw().addr());
        Some(ptr)
    }

    /// Allocate memory for type `T`
    pub fn allocate_type<T>(&self) -> Result<NonNull<T>, AllocError> {
        let guard = self.mutex.lock();
        let ptr = if align_of::<T>() <= RawAllocator::<Manager>::ELEMENT_ALIGN {
            let count = const { ElementCount::for_layout::<Manager::Node>(Layout::new::<T>()).unwrap() };
            self.raw(&guard).raw_allocate(count).map(|ptr| ptr.raw().cast()).ok_or(AllocError)?
        } else {
            let parms = const { RawAllocator::<Manager>::gen_over_count(Layout::new::<T>()).unwrap() };
            self.raw(&guard).over_align_alloc(parms).map(|ptr| ptr.raw().cast()).ok_or(AllocError)?
        };
        self.raw(&guard).manager.after_allocate(ptr.cast(), const { Layout::new::<T>() });
        self.raw(&guard).validate(ptr.addr());
        Ok(ptr)
    }

    /// Deallocate memory, ptr must allocated by this allocator before
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let guard = self.mutex.lock();
        self.raw(&guard).manager.before_deallocate(ptr, layout);
        unsafe { self.raw(&guard).deallocate(ptr) };
        self.raw(&guard).validate(ptr.addr());
    }

    /// Deallocate memory for type `T`
    /// # Safety
    ///
    /// `ptr` must have been allocated by this allocator and not yet deallocated.
    pub unsafe fn deallocate_type<T>(&self, ptr: NonNull<T>) {
        unsafe { self.deallocate(ptr.cast(), const { Layout::new::<T>() }) };
    }

    /// Re-allocate memory, ptr must alllocated by this allocator before, block will allocate at same address if avaiable
    unsafe fn reallocate(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Option<BlockPtr<Manager>> {
        match self.try_realloc(RawAllocator::<Manager>::raw_block(ptr.cast()), new_layout) {
            Ok(block) => Some(block),
            Err(_) => {
                let block = self.allocate(new_layout)?;
                unsafe { ptr.copy_to_nonoverlapping(block.raw().cast(), old_layout.size().min(new_layout.size())) };
                unsafe { self.deallocate(ptr, old_layout) };
                Some(block)
            }
        }
    }
}

unsafe impl<Mutex: AllocatorMutex, Manager: FreeBlockManager + ?Sized> Allocator for LeanFlexAllocator<Mutex, Manager> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Ptr::user_block(Self::allocate(self, layout))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { Self::deallocate(self, ptr, layout) }
    }

    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Ptr::user_block(unsafe { self.reallocate(ptr, old_layout, new_layout) })
    }

    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() >= old_layout.size(), "`new_layout.size()` must be greater than or equal to `old_layout.size()`");
        let new_ptr = unsafe { self.grow(ptr, old_layout, new_layout)? };
        unsafe { new_ptr.as_ptr().cast::<u8>().add(old_layout.size()).write_bytes(0, new_layout.size() - old_layout.size()) };
        Ok(new_ptr)
    }

    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() <= old_layout.size(), "`new_layout.size()` must be smaller than or equal to `old_layout.size()`");
        Ptr::user_block(unsafe { self.reallocate(ptr, old_layout, new_layout) })
    }
}

unsafe impl<Mutex: AllocatorMutex, Manager: FreeBlockManager + ?Sized> GlobalAlloc for LeanFlexAllocator<Mutex, Manager> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Ptr::user_ptr(self.allocate(layout))
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap_or_else(|| unsafe { unreachable_unchecked() });
        unsafe { self.deallocate(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ptr = NonNull::new(ptr).unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        Ptr::user_ptr(unsafe { self.reallocate(ptr, layout, new_layout) })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRegionInitError;

struct OveralignParms<StateElement, Element> {
    over_count: ElementCount<StateElement, Element>,
    request_count: ElementCount<StateElement, Element>,
    align: NonZero<usize>,
}

pub mod simple;

#[cfg(feature = "test-utils")]
mod validator;
