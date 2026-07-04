use core::{num::NonZero, ptr::NonNull};

use ecore::int::PrimaryUInt;

use crate::FreeBlockManager;

use super::{Block, ElementCount, Ptr};

pub struct Validator<Manager: ?Sized + FreeBlockManager> {
    regions: Vec<BlockRegion<Manager::StateElement, Manager::Element>>,
    manager: Manager,
}

impl<Manager: FreeBlockManager> Validator<Manager> {
    pub const fn new(manager: Manager) -> Self {
        Self { manager, regions: Vec::new() }
    }
}

impl<Manager: ?Sized + FreeBlockManager> FreeBlockManager for Validator<Manager> {
    type StateElement = Manager::StateElement;
    type Element = Manager::Element;
    type Node = Manager::Node;

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = Manager::MAX_ELEMENT_COUNT;

    fn take_out(&mut self, element_count: ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>> {
        self.manager.take_out(element_count)
    }

    unsafe fn register(&mut self, node: NonNull<Self::Node>) {
        unsafe { self.manager.register(node) }
    }

    unsafe fn unregister(&mut self, node: NonNull<Self::Node>) {
        unsafe { self.manager.unregister(node) }
    }

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        self.regions.push(BlockRegion::new(Ptr::new(ptr.cast())));
        unsafe { self.manager.init(ptr) };
    }

    unsafe fn extend(&mut self, ptr: NonNull<Self::Element>) {
        self.regions.push(BlockRegion::new(Ptr::new(ptr.cast())));
        unsafe { self.manager.extend(ptr) };
    }

    fn after_allocate(&mut self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        self.manager.after_allocate(ptr, layout);
        let len = ElementCount::<Manager::StateElement, Manager::Element>::get_from_block(ptr).usable_bytes_len();
        assert!(len >= layout.size());
        for i in 0..len {
            unsafe { ptr.add(i).write(rand::random()) };
        }
    }

    fn before_deallocate(&mut self, ptr: NonNull<u8>, layout: core::alloc::Layout) {
        self.manager.before_deallocate(ptr, layout)
    }

    fn address_range(&self) -> (usize, usize) {
        self.manager.address_range()
    }

    fn validate(&mut self, addr: NonZero<usize>) {
        for region in self.regions.iter() {
            if region.includes_addr(addr) {
                assert_eq!(region.validate(), Ok(()));
                break;
            }
        }
    }
}

struct BlockRegion<StateElement, Element> {
    first: Ptr<Block<StateElement, Element>>,
    last: Ptr<Block<StateElement, Element>>,
}

impl<StateElement: PrimaryUInt, Element> BlockRegion<StateElement, Element> {
    fn new(root: Ptr<Block<StateElement, Element>>) -> Self {
        Self { first: root, last: root.higher_block() }
    }

    fn includes_addr(&self, addr: NonZero<usize>) -> bool {
        self.first.raw().addr() <= addr && addr <= self.last.raw().addr()
    }

    /// validate region
    fn validate(&self) -> Result<(), ()> {
        let first = self.first;
        let low_state = first.low_joint().low_state();
        if low_state.element_count() != ElementCount::ZERO {
            return Err(());
        }
        if low_state.is_free() {
            return Err(());
        }
        let last_state = self.last.head_state();
        if last_state.element_count() != ElementCount::ZERO {
            return Err(());
        }
        if last_state.is_free() {
            return Err(());
        }
        let mut used = 0;
        let mut unused = 0;
        let mut block = first;
        let mut pre_is_free = false;
        while block.element_count() != ElementCount::ZERO {
            let cur_is_free = block.head_state().is_free();
            if cur_is_free && pre_is_free {
                return Err(());
            }
            *if cur_is_free { &mut unused } else { &mut used } += block.element_count().to_bytes();
            pre_is_free = cur_is_free;
            block = block.higher_block();
            if block.raw().addr() > self.last.raw().addr() {
                return Err(());
            }
        }
        if block.head_state().is_free() {
            return Err(());
        }
        if block.raw().addr() != self.last.raw().addr() {
            return Err(());
        }
        let Some(total) = self.last.raw().addr().get().checked_sub(self.first.raw().addr().get()) else {
            return Err(());
        };
        if total % size_of::<Element>() != 0 {
            return Err(());
        }
        if total != used + unused {
            return Err(());
        }
        Ok(())
    }
}
