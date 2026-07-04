use core::{hint::unreachable_unchecked, marker::PhantomData, ptr::NonNull};

use const_default::ConstDefault;
use ecore::int::PrimaryUInt;

use crate::{
    ElementCount, FreeBlockManager,
    ptr::{OptimizedPtr, Ptr},
};

/// First fit allocator, always find first fit block, simple and maybe very slow
/// * `StateElement` see [FreeBlockManager::StateElement]
/// * `Element` [FreeBlockManager::Element]
/// * `OptiPtr` ptr for free block, maybe narrowed, see `OptimizedPtr`, affects [FreeBlockManager::address_range] and [FreeBlockManager::Node],
///   actually [FreeBlockManager::Node] is single `OptiPtr`
pub struct FirstFit<StateElement, Element, OptiPtr: OptimizedPtr<Element>> {
    head: Option<OptiPtr>,
    base: OptiPtr::VirtualBase,
    mark: PhantomData<StateElement>,
}

impl<StateElement, Element, OptiPtr: OptimizedPtr<Element>> Default for FirstFit<StateElement, Element, OptiPtr> {
    fn default() -> Self {
        Self::new()
    }
}

impl<StateElement, Element, OptiPtr: OptimizedPtr<Element>> FirstFit<StateElement, Element, OptiPtr> {
    pub const fn new() -> Self {
        Self { head: None, base: ConstDefault::DEFAULT, mark: PhantomData }
    }

    const fn base(&self) -> Base<Element, OptiPtr> {
        Base { element: PhantomData, base: self.base }
    }

    fn head_link(&mut self) -> Ptr<Option<OptiPtr>> {
        Ptr::new(NonNull::from(&mut self.head))
    }
}

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>> FreeBlockManager for FirstFit<StateElement, Element, OptiPtr> {
    type StateElement = StateElement;
    type Element = Element;
    type Node = Node<OptiPtr>;

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = ElementCount::MAX;

    fn take_out(&mut self, element_count: crate::ElementCount<Self::StateElement, Self::Element>) -> Option<core::ptr::NonNull<Self::Node>> {
        let base = self.base();
        let mut link = self.head_link();
        while let Some(optr) = link.read() {
            let itnode = base.get_node(optr);
            if ElementCount::get_from_block(itnode.raw().cast()) >= element_count {
                *link.as_mut() = itnode.link().read();
                return Some(itnode.raw());
            }
            link = itnode.link();
        }
        None
    }

    unsafe fn register(&mut self, node: core::ptr::NonNull<Self::Node>) {
        let base = self.base();
        let node = Ptr::new(node);
        let mut head_link = self.head_link();
        *node.link().as_mut() = head_link.read();
        *head_link.as_mut() = Some(base.new_ptr(node));
    }

    unsafe fn unregister(&mut self, node: core::ptr::NonNull<Self::Node>) {
        let base = self.base();
        let node = Ptr::new(node);
        let mut link = self.head_link();
        while let Some(optr) = link.read() {
            let itnode = base.get_node(optr);
            if itnode == node {
                *link.as_mut() = itnode.link().read();
                return;
            }
            link = itnode.link();
        }
        unsafe { unreachable_unchecked() }
    }

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        self.base = OptiPtr::new_base(ptr);
        unsafe { self.register(ptr.cast()) };
    }

    fn address_range(&self) -> (usize, usize) {
        OptiPtr::address_range(self.base)
    }
}

#[derive(Clone, Copy)]
pub struct Node<OptiPtr> {
    next: Option<OptiPtr>,
}

impl<OptiPtr> Ptr<Node<OptiPtr>> {
    fn link(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().next })
    }
}

#[repr(transparent)]
struct Base<Element, OptiPtr: OptimizedPtr<Element>> {
    element: PhantomData<Element>,
    base: OptiPtr::VirtualBase,
}

impl<Element, OptiPtr: OptimizedPtr<Element>> Copy for Base<Element, OptiPtr> {}
impl<Element, OptiPtr: OptimizedPtr<Element>> Clone for Base<Element, OptiPtr> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Element, OptiPtr: OptimizedPtr<Element>> Base<Element, OptiPtr> {
    fn get_node(self, opti_ptr: OptiPtr) -> Ptr<Node<OptiPtr>> {
        Ptr::new(opti_ptr.get(self.base).cast())
    }

    fn new_ptr(self, node: Ptr<Node<OptiPtr>>) -> OptiPtr {
        OptiPtr::new(self.base, node.raw().cast())
    }
}
