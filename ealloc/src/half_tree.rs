use core::{
    hint::{assert_unchecked, unreachable_unchecked},
    marker::PhantomData,
    ptr::NonNull,
};

use const_default::ConstDefault;

use ecore::int::{CInt, PrimaryUInt};

use super::{
    ElementCount, FreeBlockManager,
    ptr::{OptimizedPtr, Ptr},
};

/// [Half Tree allocator](https://trepo.tuni.fi/bitstream/handle/10024/140229/AuvinenEetu.pdf),
/// **not include DV, requires wrapped by** [crate::DesignatedVictim] **to get DV**
/// * `StateElement` see [FreeBlockManager::StateElement]
/// * `Element` [FreeBlockManager::Element]
/// * `OptiPtr` ptr for free block, maybe narrowed, see `OptimizedPtr`, affects [FreeBlockManager::address_range] and [FreeBlockManager::Node],
///   actually [FreeBlockManager::Node] is three `OptiPtr`
pub struct HalfTree<StateElement, Element, OptiPtr: OptimizedPtr<Element>, const BINS_COUNT: usize> {
    bins: [Option<OptiPtr>; BINS_COUNT],
    bins_map: StateElement,
    base: OptiPtr::VirtualBase,
}

#[derive(Clone, Copy)]
pub struct Node<OptiPtr> {
    llink: Option<OptiPtr>,
    rlink: Option<OptiPtr>,
    siblink: Option<OptiPtr>,
}
impl<OptiPtr> Node<OptiPtr> {
    const NULL: Self = Self { llink: None, rlink: None, siblink: None };
}
impl<OptiPtr> Ptr<Node<OptiPtr>> {
    fn llink(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().llink })
    }

    fn rlink(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().rlink })
    }

    fn siblink(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().siblink })
    }
}

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, const BINS_COUNT: usize> Default
    for HalfTree<StateElement, Element, OptiPtr, BINS_COUNT>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, const BINS_COUNT: usize> HalfTree<StateElement, Element, OptiPtr, BINS_COUNT> {
    const MIN_ELEMENT_COUNT: usize = ElementCount::<StateElement, Element>::min_for::<Node<OptiPtr>>().to_count();

    pub const fn new() -> Self {
        const { assert!((BINS_COUNT as u32) < StateElement::BITS) };
        Self { bins: [None; BINS_COUNT], bins_map: StateElement::ZERO, base: ConstDefault::DEFAULT }
    }

    fn get_element_count(node: Ptr<Node<OptiPtr>>) -> usize {
        ElementCount::<StateElement, Element>::get_from_block(node.raw().cast()).to_count()
    }

    fn base(&self) -> Base<Element, OptiPtr> {
        Base { element: PhantomData, base: self.base }
    }

    /// unlink link's node must ensure link is not empty
    fn unlink_node(base: Base<Element, OptiPtr>, mut link: Ptr<Option<OptiPtr>>) {
        let node = base.get_node(link.read().unwrap_or_else(|| unsafe { unreachable_unchecked() }));
        if let Some(optr) = node.siblink().read() {
            let sibnode = base.get_node(optr);
            *sibnode.llink().as_mut() = node.llink().read();
            *sibnode.rlink().as_mut() = node.rlink().read();
            *link.as_mut() = Some(optr); // replace with sibling
        } else {
            let mut leaf_link = link;
            let mut leaf_node = node;
            loop {
                if let Some(loptr) = leaf_node.llink().read() {
                    leaf_link = leaf_node.llink();
                    leaf_node = base.get_node(loptr)
                } else if let Some(roptr) = leaf_node.rlink().read() {
                    leaf_link = leaf_node.rlink();
                    leaf_node = base.get_node(roptr)
                } else {
                    break;
                }
            }
            if leaf_link != link {
                *link.as_mut() = leaf_link.as_mut().take(); // leaf_link maybe node's llink or rlink, should take first
                *leaf_node.llink().as_mut() = node.llink().read();
                *leaf_node.rlink().as_mut() = node.rlink().read();
            } else {
                *leaf_link.as_mut() = None;
            }
        }
    }
}

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, const BINS_COUNT: usize> FreeBlockManager
    for HalfTree<StateElement, Element, OptiPtr, BINS_COUNT>
{
    type StateElement = StateElement;
    type Element = Element;
    type Node = Node<OptiPtr>;

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = ElementCount::max_for_bits::<BINS_COUNT>();

    fn take_out(&mut self, element_count: super::ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>> {
        let base = self.base();
        let Self { bins, bins_map, .. } = self;
        let count = element_count.to_count();
        unsafe { assert_unchecked(count >= Self::MIN_ELEMENT_COUNT && count <= Self::MAX_ELEMENT_COUNT.to_count()) };
        let bid = count.ilog2(); // most set bit
        {
            let bin = Ptr::from(bins.get_mut(bid as usize).unwrap_or_else(|| unsafe { unreachable_unchecked() }));
            let mut link = bin;
            let mut pos = bid;
            while let Some(optr) = link.read() {
                let node = base.get_node(optr);
                if Self::get_element_count(node) < count {
                    let Some(next_pos) = pos.checked_sub(1) else { break };
                    if node.rlink().read().is_some() {
                        link = node.rlink();
                    } else if count & 1 << next_pos != 0 {
                        break;
                    } else if node.llink().read().is_some() {
                        link = node.llink();
                    } else {
                        break;
                    }
                    pos = next_pos;
                } else {
                    Self::unlink_node(base, link);
                    if bin.read().is_none() {
                        *bins_map &= !(StateElement::ONE << bid);
                    }
                    return Some(node.raw());
                }
            }
        }
        let bid = bid + 1;
        let bid = CInt::trailing_zeros(*bins_map >> bid) + bid;
        let bin = Ptr::from(bins.get_mut(bid as usize)?);
        let optr = bin.read().unwrap_or_else(|| unsafe { unreachable_unchecked() });
        Self::unlink_node(base, bin);
        if bin.read().is_none() {
            *bins_map &= !(StateElement::ONE << bid);
        }
        Some(base.get_node(optr).raw())
    }

    unsafe fn register(&mut self, node: NonNull<Self::Node>) {
        let base = self.base();
        let mut node = Ptr::new(node);
        let count = Self::get_element_count(node);
        unsafe { assert_unchecked(count >= Self::MIN_ELEMENT_COUNT && count <= Self::MAX_ELEMENT_COUNT.to_count()) };
        let mut pos = count.ilog2(); // most set bit
        self.bins_map |= StateElement::ONE << pos;
        let bin = self.bins.get_mut(pos as usize).unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let Some(optr) = *bin else {
            *node.as_mut() = Node::NULL;
            *bin = Some(base.new_ptr(node));
            return;
        };
        let mut itnode = base.get_node(optr);
        loop {
            if Self::get_element_count(itnode) == count {
                *node.as_mut() = Node::NULL;
                *node.siblink().as_mut() = itnode.siblink().read();
                *itnode.siblink().as_mut() = Some(base.new_ptr(node));
                return;
            }
            unsafe { assert_unchecked(pos != 0) }; //count always equals to block count when pos == 0
            pos -= 1;
            let optr = if count & 1 << pos == 0 {
                if let Some(loptr) = itnode.llink().read() {
                    loptr
                } else {
                    *node.as_mut() = Node::NULL;
                    *itnode.llink().as_mut() = Some(base.new_ptr(node));
                    return;
                }
            } else {
                if let Some(roptr) = itnode.rlink().read() {
                    roptr
                } else {
                    *node.as_mut() = Node::NULL;
                    *itnode.rlink().as_mut() = Some(base.new_ptr(node));
                    return;
                }
            };
            itnode = base.get_node(optr);
        }
    }

    unsafe fn unregister(&mut self, node: NonNull<Self::Node>) {
        let base = self.base();
        let Self { bins, bins_map, .. } = self;
        let node = Ptr::new(node);
        let count = Self::get_element_count(node);
        unsafe { assert_unchecked(count >= Self::MIN_ELEMENT_COUNT && count <= Self::MAX_ELEMENT_COUNT.to_count()) };
        let bid = count.ilog2(); // most set bit
        let bin = Ptr::from(bins.get_mut(bid as usize).unwrap_or_else(|| unsafe { unreachable_unchecked() }));
        let mut link = bin;
        let mut pos = bid;
        loop {
            let mut itnode = base.get_node(link.read().unwrap_or_else(|| unsafe { unreachable_unchecked() }));
            if Self::get_element_count(itnode) == count {
                if itnode == node {
                    Self::unlink_node(base, link);
                    if bin.read().is_none() {
                        *bins_map &= !(StateElement::ONE << bid);
                    }
                    return;
                } else {
                    let optr = base.new_ptr(node);
                    loop {
                        let sibling = itnode.siblink().read().unwrap_or_else(|| unsafe { unreachable_unchecked() });
                        if sibling == optr {
                            *itnode.siblink().as_mut() = node.siblink().read();
                            return;
                        } else {
                            itnode = base.get_node(sibling);
                        }
                    }
                }
            }
            unsafe { assert_unchecked(pos != 0) }; // must founded when pos == 0
            pos -= 1;
            link = if count & 1 << pos == 0 { itnode.llink() } else { itnode.rlink() };
        }
    }

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        self.base = OptiPtr::new_base(ptr);
        unsafe { self.register(ptr.cast()) };
    }

    unsafe fn extend(&mut self, ptr: NonNull<Self::Element>) {
        unsafe { self.register(ptr.cast()) };
    }

    fn address_range(&self) -> (usize, usize) {
        OptiPtr::address_range(self.base)
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
