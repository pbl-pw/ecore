use ealloc::{Allocator, FirstFit, HalfTree, LeanFlexAllocator, SimpleFirstFit, Tlsf, TlsfParms};
use portable_atomic::AtomicBool;
use rand::random_range;
use std::{alloc::Layout, num::NonZero, ptr::NonNull, thread};

fn gen_random_layout(max_align: usize, max_size: usize) -> Layout {
    let align = 1 << random_range(0..=max_align.ilog2());
    let size = random_range(1..=max_size);
    Layout::from_size_align(size, align).unwrap()
}

fn test_alloc(allocator: &impl Allocator, done: &AtomicBool, max_align: usize, max_size: usize) {
    let mut allocated = Vec::new();

    while !done.load(portable_atomic::Ordering::Relaxed) {
        let op = random_range(0..4u8);
        match op {
            0 => {
                // allocate
                let layout = gen_random_layout(max_align, max_size);
                if let Ok(ptr) = allocator.allocate(layout) {
                    allocated.push((ptr, layout));
                }
            }
            1 => {
                // deallocate
                if !allocated.is_empty() {
                    let idx = random_range(0..allocated.len());
                    let (ptr, layout) = allocated.swap_remove(idx);
                    unsafe { allocator.deallocate(ptr.cast(), layout) }
                }
            }
            2 => {
                // grow
                if !allocated.is_empty() {
                    let idx = random_range(0..allocated.len());
                    let (ptr, old_layout) = allocated[idx];
                    let new_size = old_layout.size() + random_range(0..=max_size - old_layout.size());
                    let new_layout = Layout::from_size_align(new_size, old_layout.align()).unwrap();
                    if let Ok(new_ptr) = unsafe { allocator.grow(ptr.cast(), old_layout, new_layout) } {
                        allocated[idx] = (new_ptr, new_layout);
                    }
                }
            }
            3 => {
                // shrink
                if !allocated.is_empty() {
                    let idx = random_range(0..allocated.len());
                    let (ptr, old_layout) = allocated[idx];
                    let new_size = random_range(0..=old_layout.size());
                    let new_layout = Layout::from_size_align(new_size, old_layout.align()).unwrap();
                    if let Ok(new_ptr) = unsafe { allocator.shrink(ptr.cast(), old_layout, new_layout) } {
                        allocated[idx] = (new_ptr, new_layout);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[ignore = "long time"]
#[test]
fn allocator_simple() {
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(SimpleFirstFit::<u16, usize>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        #[cfg(target_pointer_width = "64")]
        let t2 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(SimpleFirstFit::<u32, usize>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        let t3 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(SimpleFirstFit::<usize, usize>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        t1.join().unwrap();
        #[cfg(target_pointer_width = "64")]
        t2.join().unwrap();
        t3.join().unwrap();
    });
}

#[ignore = "long time"]
#[test]
fn allocator_first_fit() {
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(FirstFit::<u16, usize, NonZero<u16>>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        #[cfg(target_pointer_width = "64")]
        let t2 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(FirstFit::<u32, usize, NonZero<u32>>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        let t3 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(FirstFit::<usize, usize, NonNull<usize>>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        t1.join().unwrap();
        #[cfg(target_pointer_width = "64")]
        t2.join().unwrap();
        t3.join().unwrap();
    });
}

#[ignore = "long time"]
#[test]
fn allocator_tlsf() {
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        let t1 = s.spawn(|| {
            type Parms = TlsfParms<u16, usize, NonZero<u16>, u16, u8>;
            let mut allocator = LeanFlexAllocator::new_without_mutex(Tlsf::<Parms, 16, 8>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        #[cfg(target_pointer_width = "64")]
        let t2 = s.spawn(|| {
            type Parms = TlsfParms<u32, usize, NonZero<u32>, u32, u8>;
            let mut allocator = LeanFlexAllocator::new_without_mutex(Tlsf::<Parms, 32, 8>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        let t3 = s.spawn(|| {
            type Parms = TlsfParms<usize, usize, NonNull<usize>, usize, u8>;
            let mut allocator = LeanFlexAllocator::new_without_mutex(Tlsf::<Parms, { usize::BITS as usize }, 8>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        t1.join().unwrap();
        #[cfg(target_pointer_width = "64")]
        t2.join().unwrap();
        t3.join().unwrap();
    });
}

#[ignore = "long time"]
#[test]
fn allocator_halftree() {
    let done = AtomicBool::new(false);
    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(HalfTree::<u16, usize, NonZero<u16>, 15>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        #[cfg(target_pointer_width = "64")]
        let t2 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(HalfTree::<u32, usize, NonZero<u32>, 15>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        let t3 = s.spawn(|| {
            let mut allocator = LeanFlexAllocator::new_without_mutex(HalfTree::<usize, usize, NonNull<usize>, 15>::new());
            let mut buf = [0usize; 4024];
            unsafe { allocator.init_with_slice(&mut buf) }.unwrap();
            test_alloc(&allocator, &done, align_of::<usize>() * 4, 1024);
        });
        t1.join().unwrap();
        #[cfg(target_pointer_width = "64")]
        t2.join().unwrap();
        t3.join().unwrap();
    })
}
