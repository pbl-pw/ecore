use core::{
    hint::{assert_unchecked, unreachable_unchecked},
    marker::PhantomData,
    ptr::NonNull,
};

use const_default::ConstDefault;
use ecore::int::{BasicInt as _, CInt, PrimaryUInt};

use crate::{
    ElementCount, FreeBlockManager,
    ptr::{OptimizedPtr, Ptr},
};

/// Parms for tlsf allocator
/// * `StateElement` see [FreeBlockManager::StateElement]
/// * `Element` [FreeBlockManager::Element]
/// * `OptiPtr` ptr for free block, maybe narrowed, see `OptimizedPtr`, affects [FreeBlockManager::address_range] and [FreeBlockManager::Node],
///   actually [FreeBlockManager::Node] is two `OptiPtr`
/// * `FlMap` first level unsigned integer type
/// * `SlMap` second level unsigned integer type
pub struct TlsfParms<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, FlMap: PrimaryUInt, SlMap: PrimaryUInt = u8>(
    PhantomData<(StateElement, Element, OptiPtr, FlMap, SlMap)>,
);

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, FlMap: PrimaryUInt, SlMap: PrimaryUInt>
    TlsfParms<StateElement, Element, OptiPtr, FlMap, SlMap>
{
    /// auto calculate FL and SL for [Tlsf]
    /// * `size` free memory region size will be managed, not require exactally size, can be upper bound
    /// * note, for not continued multi memory regions, `size` is max size of the regions, not the sum size of multi memory region
    /// * return `(FL, SL)`
    pub const fn calc_two_level_for_size(size: usize) -> (usize, usize) {
        let sl = SlMap::BITS as usize;
        let max_count = ElementCount::<StateElement, Element>::max_for_bytes(size).to_count();
        let fl = max_count.ilog2() as usize + 1;
        if max_count <= raw_max_element_count(fl, sl) { (fl, sl) } else { (fl + 1, sl) }
    }
}

impl<StateElement: PrimaryUInt, Element, OptiPtr: OptimizedPtr<Element>, FlMap: PrimaryUInt, SlMap: PrimaryUInt> AsTlsfParms
    for TlsfParms<StateElement, Element, OptiPtr, FlMap, SlMap>
{
    type StateElement = StateElement;
    type Element = Element;
    type OptiPtr = OptiPtr;
    type FlMap = FlMap;
    type SlMap = SlMap;
}

/// Tlsf allocator
/// * `Parms` must be [TlsfParms]
/// * `FL` first level count, can calculate from [TlsfParms::calc_two_level_for_size]
/// * `SL` second level count, must `is_power_of_two`, can calculate from [TlsfParms::calc_two_level_for_size]
pub struct Tlsf<Parms: AsTlsfParms, const FL: usize, const SL: usize> {
    head: [[Option<Parms::OptiPtr>; SL]; FL],
    fl_map: Parms::FlMap,
    sl_map: [Parms::SlMap; FL],
    base: <Parms::OptiPtr as OptimizedPtr<Parms::Element>>::VirtualBase,
}

impl<Parms: AsTlsfParms, const FL: usize, const SL: usize> Default for Tlsf<Parms, FL, SL> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Parms: AsTlsfParms, const FL: usize, const SL: usize> Tlsf<Parms, FL, SL> {
    const SL_BITS: usize = SL.ilog2() as usize;
    const RAW_MAX_ELEMENT_COUNT: usize = raw_max_element_count(FL, SL);
    pub const MAX_BLOCK_SIZE: usize = Self::MAX_ELEMENT_COUNT.to_bytes();

    pub const fn new() -> Self {
        const { assert!(SL.is_power_of_two() && usize::BITS as usize >= FL && <Parms::FlMap>::BITS as usize >= FL && <Parms::SlMap>::BITS as usize >= SL) }
        Self { head: [[None; SL]; FL], fl_map: <Parms::FlMap>::ZERO, sl_map: [<Parms::SlMap>::ZERO; FL], base: ConstDefault::DEFAULT }
    }

    const fn base(&self) -> Base<Parms::Element, Parms::OptiPtr> {
        Base { element: PhantomData, base: self.base }
    }

    const fn map_index(element_count: ElementCount<Parms::StateElement, Parms::Element>) -> Index {
        let count = element_count.to_count();
        unsafe { assert_unchecked(count != 0 && count <= Self::RAW_MAX_ELEMENT_COUNT) };
        let rfl = count.ilog2() as usize;
        // sl = (count - 2.pow(rfl)) / ((2.pow(rfl + 1) - 2.pow(rfl)) / SL) = (count - 2.pow(rfl)) / (2.pow(rfl) / SL)
        if rfl < Self::SL_BITS {
            Index { fl: 0, sl: count }
        } else {
            let bits = rfl - Self::SL_BITS;
            Index { fl: bits + 1, sl: (count - (1 << rfl)) >> bits }
        }
    }

    const fn map_ceil_index(element_count: ElementCount<Parms::StateElement, Parms::Element>) -> Index {
        let count = element_count.to_count();
        unsafe { assert_unchecked(count != 0 && count <= Self::RAW_MAX_ELEMENT_COUNT) };
        let rfl = count.ilog2() as usize;
        if rfl < Self::SL_BITS {
            Index { fl: 0, sl: count }
        } else {
            let count = count - (1 << rfl);
            let bits = rfl - Self::SL_BITS;
            let sl = count >> bits;
            let fl = bits + 1;
            if count == sl << bits {
                Index { fl, sl }
            } else {
                let sl = sl + 1;
                if sl >= SL { Index { fl: fl + 1, sl: 0 } } else { Index { fl, sl } }
            }
        }
    }

    fn search_index(&self, element_count: ElementCount<Parms::StateElement, Parms::Element>) -> Option<Index> {
        let index = Self::map_ceil_index(element_count);
        unsafe { assert_unchecked(index.fl < FL && index.sl < SL) };
        if self.fl_map >> index.fl & <Parms::FlMap>::ONE == <Parms::FlMap>::ONE {
            let sl_map = self.sl_map[index.fl];
            if sl_map >> index.sl & <Parms::SlMap>::ONE == <Parms::SlMap>::ONE {
                return Some(index);
            }
            let sl = index.sl + 1;
            if sl < SL {
                let sl = CInt::trailing_zeros(sl_map >> sl) as usize + sl;
                if sl < SL {
                    return Some(Index { fl: index.fl, sl });
                }
            }
        }
        let fl = index.fl + 1;
        let fl = CInt::trailing_zeros(self.fl_map >> fl) as usize + fl;
        if fl < FL {
            let sl = CInt::trailing_zeros(self.sl_map[fl]) as usize;
            debug_assert!(sl < SL);
            Some(Index { fl, sl })
        } else {
            None
        }
    }

    /// must ensure map of index is seted, because using 'xor' instead of 'and not' to clear
    fn unset_map(&mut self, index: Index) {
        let sl_map = &mut self.sl_map[index.fl];
        *sl_map ^= <Parms::SlMap>::ONE << index.sl;
        if *sl_map == <Parms::SlMap>::ZERO {
            self.fl_map ^= <Parms::FlMap>::ONE << index.fl;
        }
    }
}

impl<Parms: AsTlsfParms, const FL: usize, const SL: usize> FreeBlockManager for Tlsf<Parms, FL, SL> {
    type StateElement = Parms::StateElement;
    type Element = Parms::Element;
    type Node = Node<Parms::OptiPtr>;

    const MAX_ELEMENT_COUNT: ElementCount<Self::StateElement, Self::Element> = ElementCount::max_for_count(Self::RAW_MAX_ELEMENT_COUNT);

    fn take_out(&mut self, element_count: ElementCount<Self::StateElement, Self::Element>) -> Option<NonNull<Self::Node>> {
        let base = self.base();
        let index = self.search_index(element_count)?;
        unsafe { assert_unchecked(index.fl < FL && index.sl < SL) };
        let link = &mut self.head[index.fl][index.sl];
        if let Some(head_opti_ptr) = *link {
            let node = base.get_node(head_opti_ptr);
            *link = node.pnext().read();
            if let Some(next) = *link {
                base.get_node(next).pprev().write(None);
            } else {
                self.unset_map(index);
            }
            Some(node.raw())
        } else {
            unsafe { unreachable_unchecked() }
        }
    }

    unsafe fn register(&mut self, node: NonNull<Self::Node>) {
        let base = self.base();
        let index = Self::map_index(ElementCount::get_from_block(node.cast()));
        let node = Ptr::new(node);
        unsafe { assert_unchecked(index.fl < FL && index.sl < SL) };
        let link = &mut self.head[index.fl][index.sl];
        if let Some(head_opti_ptr) = link {
            let opti_ptr = base.new_opti_ptr(node);
            let head = base.get_node(*head_opti_ptr);
            debug_assert!(head.pprev().read().is_none());
            head.pprev().write(Some(opti_ptr));
            node.write(Node { next: Some(*head_opti_ptr), prev: None });
            *head_opti_ptr = opti_ptr;
        } else {
            self.fl_map |= <Parms::FlMap>::ONE << index.fl;
            self.sl_map[index.fl] |= <Parms::SlMap>::ONE << index.sl;
            node.write(Node { next: None, prev: None });
            *link = Some(base.new_opti_ptr(node));
        }
    }

    unsafe fn unregister(&mut self, node: NonNull<Self::Node>) {
        let base = self.base();
        let node = Ptr::new(node);
        if let Some(prev) = node.pprev().read() {
            // not head node just remove node
            let next = node.pnext().read();
            base.get_node(prev).pnext().write(next);
            if let Some(next) = next {
                base.get_node(next).pprev().write(Some(prev));
            }
        } else {
            // is head node remove node and maybe clear map
            let index = Self::map_index(ElementCount::get_from_block(node.raw().cast()));
            unsafe { assert_unchecked(index.fl < FL && index.sl < SL) };
            let link = &mut self.head[index.fl][index.sl];
            *link = node.pnext().read();
            if let Some(next) = *link {
                base.get_node(next).pprev().write(None);
            } else {
                self.unset_map(index);
            }
        }
    }

    unsafe fn init(&mut self, ptr: NonNull<Self::Element>) {
        self.base = <Parms::OptiPtr>::new_base(ptr);
        unsafe { self.register(ptr.cast()) };
    }

    fn address_range(&self) -> (usize, usize) {
        Parms::OptiPtr::address_range(self.base)
    }
}

#[derive(Clone, Copy)]
pub struct Node<OptiPtr> {
    next: Option<OptiPtr>,
    prev: Option<OptiPtr>,
}

impl<OptiPtr> Ptr<Node<OptiPtr>> {
    fn pnext(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().next })
    }

    fn pprev(self) -> Ptr<Option<OptiPtr>> {
        Ptr::from(unsafe { &mut self.raw().as_mut().prev })
    }
}

struct Index {
    fl: usize,
    sl: usize,
}

const fn raw_max_element_count(fl: usize, sl: usize) -> usize {
    debug_assert!(sl.is_power_of_two() && fl >= 1);
    let max_fl = fl - 1;
    // ensure 'map_ceil_index(count)' always return valid index
    // 2.pow(max_fl) + (2.pow(max_fl + 1) - 2.pow(max_fl)) / sl * (sl - 1) = 2.pow(max_fl) + 2.pow(max_fl) / sl * (sl - 1) = 2.pow(max_fl + 1) - 2.pow(max_fl - sl.ilog2())
    (1_usize << max_fl << 1).wrapping_sub(1 << (max_fl - sl.ilog2() as usize))
}

pub trait AsTlsfParms {
    type StateElement: PrimaryUInt;
    type Element;
    type OptiPtr: OptimizedPtr<Self::Element>;
    type FlMap: PrimaryUInt;
    type SlMap: PrimaryUInt;
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

    fn new_opti_ptr(self, node: Ptr<Node<OptiPtr>>) -> OptiPtr {
        OptiPtr::new(self.base, node.raw().cast())
    }
}
