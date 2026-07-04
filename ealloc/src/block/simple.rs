use core::{marker::PhantomData, ptr::NonNull};

use ecore::int::PrimaryUInt;

use crate::ptr::OptimizedPtr as _;

use super::{Block, ElementCount, FreeBlockManager, Ptr};

/// Simple first fit free block manager, always search free block from memory region head (no free block link list),
/// slowest, lowest internal fragment, lowest code size, and only support single memory region(extend just ignored)
pub struct SimpleFirstFit<StateElement, Element> {
    mark: PhantomData<(StateElement, Element)>,
    first: Option<NonNull<()>>,
}

impl<StateElement, Element> Default for SimpleFirstFit<StateElement, Element> {
    fn default() -> Self {
        Self::new()
    }
}

impl<StateElement, Element> SimpleFirstFit<StateElement, Element> {
    pub const fn new() -> Self {
        Self { mark: PhantomData, first: None }
    }
}

impl<StateElement: PrimaryUInt, Element> FreeBlockManager for SimpleFirstFit<StateElement, Element> {
    type StateElement = StateElement;
    type Element = Element;
    type Node = ();

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = ElementCount::MAX;

    fn address_range(&self) -> (usize, usize) {
        NonNull::<Self::Element>::address_range(())
    }

    fn take_out(&mut self, element_count: ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>> {
        let mut block = Ptr::<Block<StateElement, Element>>::new(self.first?.cast());
        loop {
            let head = block.head_state();
            let ele_count = head.element_count();
            if ele_count.is_zero() {
                return None;
            };
            if head.is_free() && ele_count >= element_count {
                return Some(block.node());
            };
            block = block.higher_block();
        }
    }

    unsafe fn register(&mut self, _: NonNull<Self::Node>) {}

    unsafe fn unregister(&mut self, _: NonNull<Self::Node>) {}

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        self.first = Some(ptr.cast());
    }
}

#[cfg(test)]
#[test]
fn test_block() {
    use crate::{LeanFlexAllocator, NoopMutex};

    type Alloc = LeanFlexAllocator<NoopMutex, SimpleFirstFit<u16, u64>>;
    let mut buf = [0u64; 1024];
    let mut alc = Alloc::new(NoopMutex::new(), SimpleFirstFit { mark: PhantomData, first: None });
    assert_eq!(unsafe { alc.init_with_slice(&mut buf) }, Ok(None));

    let mut buf = [0u64; 40000];
    let mut alc = Alloc::new(NoopMutex::new(), SimpleFirstFit { mark: PhantomData, first: None });
    let remain = unsafe { alc.init_with_slice(&mut buf) }.unwrap().unwrap();
    let range = &mut buf[32768..].as_mut_ptr_range();
    let used = NonNull::new(range.start).unwrap();
    let end = NonNull::new(range.end).unwrap();
    assert_eq!(remain.addr(), used.addr());
    assert_eq!(unsafe { alc.owned_extend(remain) }, (1, unsafe { used.cast::<u8>().add(4)..end.cast::<u8>() }));
}
