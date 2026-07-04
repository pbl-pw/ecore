use core::{alloc::Layout, ptr::NonNull};

use super::{ElementCount, FreeBlockManager};

/// Designated victim [FreeBlockManager] wrapper, add designated victim like `dlmalloc` or `half tree`
pub struct DesignatedVictim<const MIN_SIZE: usize, const MAX_SIZE: usize, Manager: ?Sized> {
    dv: Option<NonNull<u8>>,
    manager: Manager,
}

impl<const MIN_SIZE: usize, const MAX_SIZE: usize, Manager> DesignatedVictim<MIN_SIZE, MAX_SIZE, Manager> {
    pub const fn new(manager: Manager) -> Self {
        Self { dv: None, manager }
    }
}

impl<const MIN_SIZE: usize, const MAX_SIZE: usize, Manager: ?Sized + FreeBlockManager> DesignatedVictim<MIN_SIZE, MAX_SIZE, Manager> {
    const MIN_COUNT: ElementCount<Manager::StateElement, Manager::Element> =
        ElementCount::for_layout::<Manager::Node>(unsafe { Layout::from_size_align_unchecked(MIN_SIZE, align_of::<Manager::Element>()) }).unwrap();

    const MAX_COUNT: ElementCount<Manager::StateElement, Manager::Element> =
        ElementCount::for_layout::<Manager::Node>(unsafe { Layout::from_size_align_unchecked(MAX_SIZE, align_of::<Manager::Element>()) }).unwrap();
}

impl<const MIN_SIZE: usize, const MAX_SIZE: usize, Manager: ?Sized + FreeBlockManager> FreeBlockManager for DesignatedVictim<MIN_SIZE, MAX_SIZE, Manager> {
    type StateElement = Manager::StateElement;
    type Element = Manager::Element;
    type Node = Manager::Node;

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = Manager::MAX_ELEMENT_COUNT;

    fn address_range(&self) -> (usize, usize) {
        self.manager.address_range()
    }

    fn take_out(&mut self, element_count: ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>> {
        match self.dv {
            Some(dv) if ElementCount::get_from_block(dv) >= element_count => {
                self.dv = None;
                Some(dv.cast())
            }
            _ => self.manager.take_out(element_count),
        }
    }

    unsafe fn register(&mut self, node: NonNull<Self::Node>) {
        if ElementCount::get_from_block(node.cast()) <= Self::MAX_COUNT {
            match self.dv {
                None => return self.dv = Some(node.cast()),

                Some(dv) if ElementCount::get_from_block(dv) < Self::MIN_COUNT => {
                    unsafe { self.manager.register(dv.cast()) }
                    return self.dv = Some(node.cast());
                }
                _ => {}
            }
        }
        unsafe { self.manager.register(node) }
    }

    unsafe fn unregister(&mut self, node: NonNull<Self::Node>) {
        match self.dv {
            Some(dv) if node.cast::<u8>() == dv => self.dv = None,
            _ => unsafe { self.manager.unregister(node) },
        }
    }

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        unsafe { self.manager.init(ptr) }
    }

    unsafe fn extend(&mut self, ptr: NonNull<Self::Element>) {
        unsafe { self.manager.extend(ptr) }
    }

    fn after_allocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.manager.after_allocate(ptr, layout)
    }

    fn before_deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.manager.before_deallocate(ptr, layout)
    }
}
