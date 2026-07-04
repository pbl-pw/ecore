#![cfg(not(miri))]

use ecore::{
    bitint::{BitInt, i1, i7, u1, u7},
    int::{BasicInt, BasicSInt, BasicUInt},
};

#[track_caller]
fn should_panic<R>(f: impl std::panic::UnwindSafe + FnOnce() -> R) {
    use std::{
        cell::Cell,
        panic::{PanicHookInfo, catch_unwind, set_hook, take_hook},
        sync::{Arc, Once},
    };
    thread_local! {
        static SHOULD_PANIC: Cell<bool> = Cell::new(false);
    }
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let orig = take_hook();
        let orig = Arc::new(orig);

        set_hook(Box::new(move |info: &PanicHookInfo| {
            if !SHOULD_PANIC.with(|c| c.get()) {
                (orig.as_ref())(info);
            }
        }));
    });
    SHOULD_PANIC.with(|c| c.set(true));
    let result = catch_unwind(f);
    SHOULD_PANIC.set(false);
    match result {
        Ok(_) => panic!("expr should panic but it's not"),
        Err(_) => {}
    }
}

#[allow(non_camel_case_types)]
type u0 = BitInt<u8, 0>;
#[allow(non_camel_case_types)]
type i0 = BitInt<i8, 0>;

#[test]
fn value() {
    for i in i8::MIN..=-1 {
        assert_eq!(i0::new(i), None);
        assert_eq!(i0::try_new(i), Err(i));
    }
    assert_eq!(i0::new(0), Some(i0::ZERO));
    assert_eq!(i0::try_new(0), Ok(i0::ZERO));
    for i in 1..=i8::MAX {
        assert_eq!(i0::new(i), None);
        assert_eq!(i0::try_new(i), Err(i));
    }

    assert_eq!(u0::new(0), Some(u0::ZERO));
    assert_eq!(u0::try_new(0), Ok(u0::ZERO));
    for i in 1..=u8::MAX {
        assert_eq!(u0::new(i), None);
        assert_eq!(u0::try_new(i), Err(i));
    }

    for i in i8::MIN..=-2 {
        assert_eq!(i1::new(i), None);
        assert_eq!(i1::try_new(i), Err(i));
    }
    for i in -1..=0 {
        assert_eq!(i1::new(i).unwrap().value(), i);
        assert_eq!(i1::try_new(i).unwrap().value(), i)
    }
    for i in 1..=i8::MAX {
        assert_eq!(i1::new(i), None);
        assert_eq!(i1::try_new(i), Err(i))
    }

    for i in i8::MIN..=-65 {
        assert_eq!(i7::new(i), None);
        assert_eq!(i7::try_new(i), Err(i))
    }
    for i in -64i8..=63 {
        assert_eq!(i7::new(i).unwrap().value(), i);
        assert_eq!(i7::try_new(i).unwrap().value(), i)
    }
    for i in 64i8..=i8::MAX {
        assert_eq!(i7::new(i), None);
        assert_eq!(i7::try_new(i), Err(i))
    }
    for i in i8::MIN..=i8::MAX {
        let v = i7::cast_new(i).value();
        assert_eq!(v, if i & 0b0100_0000 != 0 { i | 0b1000_0000u8 as i8 } else { i & 0b0111_1111 })
    }

    for i in u8::MIN..=127 {
        assert_eq!(u7::new(i).unwrap().value(), i);
        assert_eq!(u7::try_new(i).unwrap().value(), i);
    }
    for i in 128..=u8::MAX {
        assert_eq!(u7::new(i), None);
        assert_eq!(u7::try_new(i), Err(i));
    }
    for i in u8::MIN..=u8::MAX {
        assert_eq!(u7::cast_new(i).value(), i & 0b0111_1111);
    }
}

#[test]
fn add() {
    assert_eq!(u0::ZERO + u0::ZERO, u0::ZERO);
    assert_eq!(u0::ZERO.checked_add(u0::ZERO), Some(u0::ZERO));
    assert_eq!(u0::ZERO.overflowing_add(u0::ZERO), (u0::ZERO, false));
    assert_eq!(u0::ZERO.saturating_add(u0::ZERO), u0::ZERO);
    assert_eq!(unsafe { u0::ZERO.unchecked_add(u0::ZERO) }, u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_add(u0::ZERO), u0::ZERO);

    assert_eq!(i0::ZERO + i0::ZERO, i0::ZERO);
    assert_eq!(i0::ZERO.checked_add(i0::ZERO), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_add(i0::ZERO), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_add(i0::ZERO), i0::ZERO);
    assert_eq!(unsafe { i0::ZERO.unchecked_add(i0::ZERO) }, i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_add(i0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 + zo, n1);
    assert_eq!(zo + zo, zo);
    should_panic(|| n1 + n1);
    assert_eq!(n1.checked_add(zo), Some(n1));
    assert_eq!(zo.checked_add(zo), Some(zo));
    assert_eq!(n1.checked_add(n1), None);
    assert_eq!(n1.overflowing_add(zo), (n1, false));
    assert_eq!(zo.overflowing_add(zo), (zo, false));
    assert_eq!(n1.overflowing_add(n1), (zo, true));
    assert_eq!(n1.saturating_add(zo), n1);
    assert_eq!(zo.saturating_add(zo), zo);
    assert_eq!(n1.saturating_add(n1), n1);
    assert_eq!(unsafe { n1.unchecked_add(zo) }, n1);
    assert_eq!(unsafe { zo.unchecked_add(zo) }, zo);
    should_panic(|| unsafe { n1.unchecked_add(n1) });
    assert_eq!(n1.wrapping_add(zo), n1);
    assert_eq!(zo.wrapping_add(zo), zo);
    assert_eq!(n1.wrapping_add(n1), zo);

    for i in -64..=63 {
        let start = if i < 0 { -64 - i } else { -64 };
        let end = if i < 0 { 63 } else { 63 - i };
        let a = i7::new(i).unwrap();
        for j in -64..start {
            let b = i7::new(j).unwrap();
            should_panic(|| a + b);
            assert_eq!(a.checked_add(b), None);
            assert_eq!(a.overflowing_add(b), (i7::cast_new(i + j), true));
            assert_eq!(a.saturating_add(b), i7::MIN);
            should_panic(|| unsafe { a.unchecked_add(b) });
            assert_eq!(a.wrapping_add(b), i7::cast_new(i + j));
        }
        for j in start..=end {
            let b = i7::new(j).unwrap();
            let sum = i7::new(i + j).unwrap();
            assert_eq!(a + b, sum);
            assert_eq!(a.checked_add(b), Some(sum));
            assert_eq!(a.overflowing_add(b), (sum, false));
            assert_eq!(a.saturating_add(b), sum);
            assert_eq!(unsafe { a.unchecked_add(b) }, sum);
            assert_eq!(a.wrapping_add(b), sum);
        }
        for j in end + 1..=63 {
            let b = i7::new(j).unwrap();
            should_panic(|| a + b);
            assert_eq!(a.checked_add(b), None);
            assert_eq!(a.overflowing_add(b), (i7::cast_new(i + j), true));
            assert_eq!(a.saturating_add(b), i7::MAX);
            should_panic(|| unsafe { a.unchecked_add(b) });
            assert_eq!(a.wrapping_add(b), i7::new(i + j | 0b1000_0000u8 as i8).unwrap());
        }
    }

    for i in 0..=127 {
        let end = 127 - i; // 127 ~ 0
        let a = u7::new(i).unwrap();
        for j in 0..=end {
            let b = u7::new(j).unwrap();
            let sum = u7::new(i + j).unwrap();
            assert_eq!(a + b, sum);
            assert_eq!(a.checked_add(b), Some(sum));
            assert_eq!(a.overflowing_add(b), (sum, false));
            assert_eq!(a.saturating_add(b), sum);
            assert_eq!(unsafe { a.unchecked_add(b) }, sum);
            assert_eq!(a.wrapping_add(b), sum);
        }
        for j in end + 1..=127 {
            let b = u7::new(j).unwrap();
            should_panic(|| a + b);
            assert_eq!(a.checked_add(b), None);
            assert_eq!(a.overflowing_add(b), (u7::cast_new(i + j), true));
            assert_eq!(a.saturating_add(b), u7::MAX);
            should_panic(|| unsafe { a.unchecked_add(b) });
            assert_eq!(a.wrapping_add(b), u7::new(i + j & 0b0111_1111).unwrap());
        }
    }
}

#[test]
fn bitand() {
    assert_eq!(u0::ZERO & u0::ZERO, u0::ZERO);

    assert_eq!(i0::ZERO & i0::ZERO, i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 & n1, n1);
    assert_eq!(n1 & zo, zo);
    assert_eq!(zo & zo, zo);

    for i in -64..=63 {
        for j in -64..=63 {
            assert_eq!(i7::new(i).unwrap() & i7::new(j).unwrap(), i7::new(i & j).unwrap());
        }
    }

    for i in 0..=127 {
        for j in 0..=127 {
            assert_eq!(u7::new(i).unwrap() & u7::new(j).unwrap(), u7::new(i & j).unwrap());
        }
    }
}

#[test]
fn bitor() {
    assert_eq!(u0::ZERO | u0::ZERO, u0::ZERO);

    assert_eq!(i0::ZERO | i0::ZERO, i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 | n1, n1);
    assert_eq!(n1 | zo, n1);
    assert_eq!(zo | zo, zo);

    for i in -64..=63 {
        for j in -64..=63 {
            assert_eq!(i7::new(i).unwrap() | i7::new(j).unwrap(), i7::new(i | j).unwrap());
        }
    }

    for i in 0..=127 {
        for j in 0..=127 {
            assert_eq!(u7::new(i).unwrap() | u7::new(j).unwrap(), u7::new(i | j).unwrap());
        }
    }
}

#[test]
fn bitxor() {
    assert_eq!(u0::ZERO ^ u0::ZERO, u0::ZERO);

    assert_eq!(i0::ZERO ^ i0::ZERO, i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 ^ n1, zo);
    assert_eq!(n1 ^ zo, n1);
    assert_eq!(zo ^ zo, zo);

    for i in -64..=63 {
        for j in -64..=63 {
            assert_eq!(i7::new(i).unwrap() ^ i7::new(j).unwrap(), i7::new(i ^ j).unwrap());
        }
    }

    for i in 0..=127 {
        for j in 0..=127 {
            assert_eq!(u7::new(i).unwrap() ^ u7::new(j).unwrap(), u7::new(i ^ j).unwrap());
        }
    }
}

#[test]
fn div() {
    should_panic(|| u0::ZERO / u0::ZERO);
    assert_eq!(u0::ZERO.checked_div(u0::ZERO), None);
    assert_eq!(u0::ZERO.checked_div_euclid(u0::ZERO), None);
    assert_eq!(u0::ZERO.overflowing_div(u0::ZERO), (u0::ZERO, true));
    assert_eq!(u0::ZERO.overflowing_div_euclid(u0::ZERO), (u0::ZERO, true));
    assert_eq!(u0::ZERO.saturating_div(u0::ZERO), u0::ZERO);
    should_panic(|| u0::ZERO.wrapping_div(u0::ZERO));
    should_panic(|| u0::ZERO.wrapping_div_euclid(u0::ZERO));

    should_panic(|| i0::ZERO / i0::ZERO);
    assert_eq!(i0::ZERO.checked_div(i0::ZERO), None);
    assert_eq!(i0::ZERO.checked_div_euclid(i0::ZERO), None);
    assert_eq!(i0::ZERO.overflowing_div(i0::ZERO), (i0::ZERO, true));
    assert_eq!(i0::ZERO.overflowing_div_euclid(i0::ZERO), (i0::ZERO, true));
    assert_eq!(i0::ZERO.saturating_div(i0::ZERO), i0::ZERO);
    should_panic(|| i0::ZERO.wrapping_div(i0::ZERO));
    should_panic(|| i0::ZERO.wrapping_div_euclid(i0::ZERO));

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1 / n1);
    should_panic(|| n1 / zo);
    assert_eq!(zo / n1, zo);
    should_panic(|| zo / zo);
    assert_eq!(n1.checked_div(n1), None);
    assert_eq!(n1.checked_div(zo), None);
    assert_eq!(zo.checked_div(n1), Some(zo));
    assert_eq!(zo.checked_div(zo), None);
    assert_eq!(n1.checked_div_euclid(n1), None);
    assert_eq!(n1.checked_div_euclid(zo), None);
    assert_eq!(zo.checked_div_euclid(n1), Some(zo));
    assert_eq!(zo.checked_div_euclid(zo), None);
    assert_eq!(n1.overflowing_div(n1), (n1, true));
    assert_eq!(n1.overflowing_div(zo), (zo, true));
    assert_eq!(zo.overflowing_div(n1), (zo, false));
    assert_eq!(zo.overflowing_div(zo), (zo, true));
    assert_eq!(n1.overflowing_div_euclid(n1), (n1, true));
    assert_eq!(n1.overflowing_div_euclid(zo), (zo, true));
    assert_eq!(zo.overflowing_div_euclid(n1), (zo, false));
    assert_eq!(zo.overflowing_div_euclid(zo), (zo, true));
    assert_eq!(n1.saturating_div(n1), zo);
    assert_eq!(n1.saturating_div(zo), zo);
    assert_eq!(zo.saturating_div(n1), zo);
    assert_eq!(zo.saturating_div(zo), zo);
    assert_eq!(n1.wrapping_div(n1), n1);
    should_panic(|| n1.wrapping_div(zo));
    assert_eq!(zo.wrapping_div(n1), zo);
    should_panic(|| zo.wrapping_div(zo));
    assert_eq!(n1.wrapping_div_euclid(n1), n1);
    should_panic(|| n1.wrapping_div_euclid(zo));
    assert_eq!(zo.wrapping_div_euclid(n1), zo);
    should_panic(|| zo.wrapping_div_euclid(zo));

    for i in -64..=63 {
        let a = i7::new(i).unwrap();
        for j in -64..=63 {
            let b = i7::new(j).unwrap();
            if j == 0 || i == -64 && j == -1 {
                should_panic(|| a / b);
                assert_eq!(a.checked_div(b), None);
                assert_eq!(a.checked_div_euclid(b), None);
                assert_eq!(a.overflowing_div(b), (if j == 0 { i7::ZERO } else { i7::MIN }, true));
                assert_eq!(a.overflowing_div_euclid(b), (if j == 0 { i7::ZERO } else { i7::MIN }, true));
                assert_eq!(a.saturating_div(b), i7::MAX);
                assert_eq!(if j == 0 { should_panic(|| a.wrapping_div(b)) } else { assert_eq!(a.wrapping_div(b), i7::MIN) }, ());
                assert_eq!(if j == 0 { should_panic(|| a.wrapping_div_euclid(b)) } else { assert_eq!(a.wrapping_div_euclid(b), i7::MIN) }, ());
            } else {
                let q = i7::new(i / j).unwrap();
                let qe = i7::new(i.div_euclid(j)).unwrap();
                assert_eq!(a / b, q);
                assert_eq!(a.checked_div(b), Some(q));
                assert_eq!(a.checked_div_euclid(b), Some(qe));
                assert_eq!(a.overflowing_div(b), (q, false));
                assert_eq!(a.overflowing_div_euclid(b), (qe, false));
                assert_eq!(a.saturating_div(b), q);
                assert_eq!(a.wrapping_div(b), q);
                assert_eq!(a.wrapping_div_euclid(b), qe);
            }
        }
    }

    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        for j in 0..=127 {
            let b = u7::new(j).unwrap();
            if j == 0 {
                should_panic(|| a / b);
                assert_eq!(a.checked_div(b), None);
                assert_eq!(a.checked_div_euclid(b), None);
                assert_eq!(a.overflowing_div(b), (u7::ZERO, true));
                assert_eq!(a.overflowing_div_euclid(b), (u7::ZERO, true));
                assert_eq!(a.saturating_div(b), u7::MAX);
                should_panic(|| a.wrapping_div(b));
                should_panic(|| a.wrapping_div_euclid(b));
                should_panic(|| a.div_ceil(b));
            } else {
                let q = u7::new(i / j).unwrap();
                let qe = u7::new(i.div_euclid(j)).unwrap();
                assert_eq!(a / b, q);
                assert_eq!(a.checked_div(b), Some(q));
                assert_eq!(a.checked_div_euclid(b), Some(qe));
                assert_eq!(a.overflowing_div(b), (q, false));
                assert_eq!(a.overflowing_div_euclid(b), (qe, false));
                assert_eq!(a.saturating_div(b), q);
                assert_eq!(a.wrapping_div(b), q);
                assert_eq!(a.wrapping_div_euclid(b), qe);
                assert_eq!(a.div_ceil(b), u7::new(i.div_ceil(j)).unwrap());
            }
        }
    }
}

#[test]
fn ilog() {
    should_panic(|| u0::ZERO.ilog(u0::ZERO));
    assert_eq!(u0::ZERO.checked_ilog(u0::ZERO), None);

    should_panic(|| i0::ZERO.ilog(i0::ZERO));
    assert_eq!(i0::ZERO.checked_ilog(i0::ZERO), None);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1.ilog(n1));
    should_panic(|| n1.ilog(zo));
    should_panic(|| zo.ilog(zo));
    should_panic(|| zo.ilog(zo));
    assert_eq!(n1.checked_ilog(n1), None);
    assert_eq!(n1.checked_ilog(zo), None);
    assert_eq!(zo.checked_ilog(n1), None);
    assert_eq!(zo.checked_ilog(zo), None);

    for i in -64..1 {
        let a = i7::new(i).unwrap();
        for j in -64..=63 {
            let b = i7::new(j).unwrap();
            should_panic(|| a.ilog(b));
            assert_eq!(a.checked_ilog(b), None)
        }
    }
    for i in 1..=63 {
        let a = i7::new(i).unwrap();
        for j in -64..2 {
            let b = i7::new(j).unwrap();
            should_panic(|| a.ilog(b));
            assert_eq!(a.checked_ilog(b), None);
        }
        for j in 2..=63 {
            let b = i7::new(j).unwrap();
            let n = i.ilog(j);
            assert_eq!(a.ilog(b), n);
            assert_eq!(a.checked_ilog(b), Some(n));
        }
    }

    for j in 0..=127 {
        let b = u7::new(j).unwrap();
        should_panic(|| u7::ZERO.ilog(b));
        assert_eq!(u7::ZERO.checked_ilog(b), None);
    }
    for i in 1..=127 {
        let a = u7::new(i).unwrap();
        for j in 0..2 {
            let b = u7::new(j).unwrap();
            should_panic(|| a.ilog(b));
            assert_eq!(a.checked_ilog(b), None);
        }
        for j in 2..=127 {
            let b = u7::new(j).unwrap();
            let n = i.ilog(j);
            assert_eq!(a.ilog(b), n);
            assert_eq!(a.checked_ilog(b), Some(n));
        }
    }
}

#[test]
fn ilog2() {
    should_panic(|| u0::ZERO.ilog2());
    assert_eq!(u0::ZERO.checked_ilog2(), None);

    should_panic(|| i0::ZERO.ilog2());
    assert_eq!(i0::ZERO.checked_ilog2(), None);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1.ilog2());
    should_panic(|| n1.ilog2());
    should_panic(|| zo.ilog2());
    should_panic(|| zo.ilog2());
    assert_eq!(n1.checked_ilog2(), None);
    assert_eq!(n1.checked_ilog2(), None);
    assert_eq!(zo.checked_ilog2(), None);
    assert_eq!(zo.checked_ilog2(), None);

    for i in -64..1 {
        let a = i7::new(i).unwrap();
        should_panic(|| a.ilog2());
        assert_eq!(a.checked_ilog2(), None)
    }
    for i in 1..=63 {
        let a = i7::new(i).unwrap();
        let n = i.ilog2();
        assert_eq!(a.ilog2(), n);
        assert_eq!(a.checked_ilog2(), Some(n));
    }

    for i in 1..=127 {
        let a = u7::new(i).unwrap();
        let n = i.ilog2();
        assert_eq!(a.ilog2(), n);
        assert_eq!(a.checked_ilog2(), Some(n));
    }
}

#[test]
fn ilog10() {
    should_panic(|| u0::ZERO.ilog10());
    assert_eq!(u0::ZERO.checked_ilog10(), None);

    should_panic(|| i0::ZERO.ilog10());
    assert_eq!(i0::ZERO.checked_ilog10(), None);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1.ilog10());
    should_panic(|| n1.ilog10());
    should_panic(|| zo.ilog10());
    should_panic(|| zo.ilog10());
    assert_eq!(n1.checked_ilog10(), None);
    assert_eq!(n1.checked_ilog10(), None);
    assert_eq!(zo.checked_ilog10(), None);
    assert_eq!(zo.checked_ilog10(), None);

    for i in -64..1 {
        let a = i7::new(i).unwrap();
        should_panic(|| a.ilog10());
        assert_eq!(a.checked_ilog10(), None)
    }
    for i in 1..=63 {
        let a = i7::new(i).unwrap();
        let n = i.ilog10();
        assert_eq!(a.ilog10(), n);
        assert_eq!(a.checked_ilog10(), Some(n));
    }

    for i in 1..=127 {
        let a = u7::new(i).unwrap();
        let n = i.ilog10();
        assert_eq!(a.ilog10(), n);
        assert_eq!(a.checked_ilog10(), Some(n));
    }
}

#[test]
fn mul() {
    assert_eq!(u0::ZERO * u0::ZERO, u0::ZERO);
    assert_eq!(u0::ZERO.checked_mul(u0::ZERO), Some(u0::ZERO));
    assert_eq!(u0::ZERO.overflowing_mul(u0::ZERO), (u0::ZERO, false));
    assert_eq!(u0::ZERO.saturating_mul(u0::ZERO), u0::ZERO);
    assert_eq!(unsafe { u0::ZERO.unchecked_mul(u0::ZERO) }, u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_mul(u0::ZERO), u0::ZERO);

    assert_eq!(i0::ZERO * i0::ZERO, i0::ZERO);
    assert_eq!(i0::ZERO.checked_mul(i0::ZERO), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_mul(i0::ZERO), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_mul(i0::ZERO), i0::ZERO);
    assert_eq!(unsafe { i0::ZERO.unchecked_mul(i0::ZERO) }, i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_mul(i0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1 * n1);
    assert_eq!(n1 * zo, zo);
    assert_eq!(zo * zo, zo);
    assert_eq!(n1.checked_mul(n1), None);
    assert_eq!(n1.checked_mul(zo), Some(zo));
    assert_eq!(zo.checked_mul(zo), Some(zo));
    assert_eq!(n1.overflowing_mul(n1), (n1, true));
    assert_eq!(n1.overflowing_mul(zo), (zo, false));
    assert_eq!(zo.overflowing_mul(zo), (zo, false));
    assert_eq!(n1.saturating_mul(n1), zo);
    assert_eq!(n1.saturating_mul(zo), zo);
    assert_eq!(zo.saturating_mul(zo), zo);
    should_panic(|| unsafe { n1.unchecked_mul(n1) });
    assert_eq!(unsafe { n1.unchecked_mul(zo) }, zo);
    assert_eq!(unsafe { zo.unchecked_mul(zo) }, zo);
    assert_eq!(n1.wrapping_mul(n1), n1);
    assert_eq!(n1.wrapping_mul(zo), zo);
    assert_eq!(zo.wrapping_mul(zo), zo);

    for i in -64..=63i8 {
        let a = i7::new(i).unwrap();
        for j in -64..=63 {
            let b = i7::new(j).unwrap();
            if let Some(v) = i.checked_mul(j)
                && v >= -64
                && v <= 63
            {
                let pod = i7::new(i * j).unwrap();
                assert_eq!(a * b, pod);
                assert_eq!(a.checked_mul(b), Some(pod));
                assert_eq!(a.overflowing_mul(b), (pod, false));
                assert_eq!(a.saturating_mul(b), pod);
                assert_eq!(unsafe { a.unchecked_mul(b) }, pod);
                assert_eq!(a.wrapping_mul(b), pod);
            } else {
                let wmul = i7::cast_new(i.wrapping_mul(j));
                should_panic(|| a * b);
                assert_eq!(a.checked_mul(b), None);
                assert_eq!(a.overflowing_mul(b), (wmul, true));
                assert_eq!(a.saturating_mul(b), if i < 0 && j > 0 || i > 0 && j < 0 { i7::MIN } else { i7::MAX });
                should_panic(|| unsafe { a.unchecked_mul(b) });
                assert_eq!(a.wrapping_mul(b), wmul);
            }
        }
    }

    for i in 0..=127u8 {
        let a = u7::new(i).unwrap();
        for j in 0..=127 {
            let b = u7::new(j).unwrap();
            if let Some(v) = i.checked_mul(j)
                && v <= 127
            {
                let pod = u7::new(i * j).unwrap();
                assert_eq!(a * b, pod);
                assert_eq!(a.checked_mul(b), Some(pod));
                assert_eq!(a.overflowing_mul(b), (pod, false));
                assert_eq!(a.saturating_mul(b), pod);
                assert_eq!(unsafe { a.unchecked_mul(b) }, pod);
                assert_eq!(a.wrapping_mul(b), pod);
            } else {
                let wmul = u7::cast_new(i.wrapping_mul(j));
                should_panic(|| a * b);
                assert_eq!(a.checked_mul(b), None);
                assert_eq!(a.overflowing_mul(b), (wmul, true));
                assert_eq!(a.saturating_mul(b), u7::MAX);
                should_panic(|| unsafe { a.unchecked_mul(b) });
                assert_eq!(a.wrapping_mul(b), wmul);
            }
        }
    }
}

#[test]
fn not() {
    assert_eq!(!u0::ZERO, u0::ZERO);

    assert_eq!(!i0::ZERO, i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(!n1, zo);
    assert_eq!(!zo, n1);

    for i in -64..=63 {
        assert_eq!(!i7::new(i).unwrap(), i7::new(!i).unwrap());
    }

    for i in 0..=127 {
        assert_eq!(!u7::new(i).unwrap(), u7::cast_new(!i));
    }
}

#[test]
fn rem() {
    should_panic(|| u0::ZERO % u0::ZERO);
    should_panic(|| u0::ZERO.rem_euclid(u0::ZERO));
    assert_eq!(u0::ZERO.checked_rem(u0::ZERO), None);
    assert_eq!(u0::ZERO.checked_rem_euclid(u0::ZERO), None);
    assert_eq!(u0::ZERO.overflowing_rem(u0::ZERO), (u0::ZERO, true));
    assert_eq!(u0::ZERO.overflowing_rem_euclid(u0::ZERO), (u0::ZERO, true));
    should_panic(|| u0::ZERO.wrapping_rem(u0::ZERO));
    should_panic(|| u0::ZERO.wrapping_rem_euclid(u0::ZERO));

    should_panic(|| i0::ZERO % i0::ZERO);
    should_panic(|| i0::ZERO.rem_euclid(i0::ZERO));
    assert_eq!(i0::ZERO.checked_rem(i0::ZERO), None);
    assert_eq!(i0::ZERO.checked_rem_euclid(i0::ZERO), None);
    assert_eq!(i0::ZERO.overflowing_rem(i0::ZERO), (i0::ZERO, true));
    assert_eq!(i0::ZERO.overflowing_rem_euclid(i0::ZERO), (i0::ZERO, true));
    should_panic(|| i0::ZERO.wrapping_rem(i0::ZERO));
    should_panic(|| i0::ZERO.wrapping_rem_euclid(i0::ZERO));

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 % n1, zo);
    should_panic(|| n1 % zo);
    assert_eq!(zo % n1, zo);
    should_panic(|| zo % zo);
    assert_eq!(n1.rem_euclid(n1), zo);
    should_panic(|| n1.rem_euclid(zo));
    assert_eq!(zo.rem_euclid(n1), zo);
    should_panic(|| zo.rem_euclid(zo));
    assert_eq!(n1.checked_rem(n1), Some(zo));
    assert_eq!(n1.checked_rem(zo), None);
    assert_eq!(zo.checked_rem(n1), Some(zo));
    assert_eq!(zo.checked_rem(zo), None);
    assert_eq!(n1.checked_rem_euclid(n1), Some(zo));
    assert_eq!(n1.checked_rem_euclid(zo), None);
    assert_eq!(zo.checked_rem_euclid(n1), Some(zo));
    assert_eq!(zo.checked_rem_euclid(zo), None);
    assert_eq!(n1.overflowing_rem(n1), (zo, false));
    assert_eq!(n1.overflowing_rem(zo), (zo, true));
    assert_eq!(zo.overflowing_rem(n1), (zo, false));
    assert_eq!(zo.overflowing_rem(zo), (zo, true));
    assert_eq!(n1.overflowing_rem_euclid(n1), (zo, false));
    assert_eq!(n1.overflowing_rem_euclid(zo), (zo, true));
    assert_eq!(zo.overflowing_rem_euclid(n1), (zo, false));
    assert_eq!(zo.overflowing_rem_euclid(zo), (zo, true));
    assert_eq!(n1.wrapping_rem(n1), zo);
    should_panic(|| n1.wrapping_rem(zo));
    assert_eq!(zo.wrapping_rem(n1), zo);
    should_panic(|| zo.wrapping_rem(zo));
    assert_eq!(n1.wrapping_rem_euclid(n1), zo);
    should_panic(|| n1.wrapping_rem_euclid(zo));
    assert_eq!(zo.wrapping_rem_euclid(n1), zo);
    should_panic(|| zo.wrapping_rem_euclid(zo));

    for i in -64..=63 {
        let a = i7::new(i).unwrap();
        for j in -64..=63 {
            let b = i7::new(j).unwrap();
            if j == 0 {
                should_panic(|| a % b);
                should_panic(|| a.rem_euclid(b));
                assert_eq!(a.checked_rem(b), None);
                assert_eq!(a.checked_rem_euclid(b), None);
                assert_eq!(a.overflowing_rem(b), (i7::ZERO, true));
                assert_eq!(a.overflowing_rem_euclid(b), (i7::ZERO, true));
                should_panic(|| a.wrapping_rem(b));
                should_panic(|| a.wrapping_rem_euclid(b));
            } else {
                let rem = i7::new(i % j).unwrap();
                let reme = i7::new(i.rem_euclid(j)).unwrap();
                assert_eq!(a % b, rem);
                assert_eq!(a.rem_euclid(b), reme);
                assert_eq!(a.checked_rem(b), Some(rem));
                assert_eq!(a.checked_rem_euclid(b), Some(reme));
                assert_eq!(a.overflowing_rem(b), (rem, false));
                assert_eq!(a.overflowing_rem_euclid(b), (reme, false));
                assert_eq!(a.wrapping_rem(b), rem);
                assert_eq!(a.wrapping_rem_euclid(b), reme);
            }
        }
    }

    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        for j in 0..=127 {
            let b = u7::new(j).unwrap();
            if j == 0 {
                should_panic(|| a % b);
                should_panic(|| a.rem_euclid(b));
                assert_eq!(a.checked_rem(b), None);
                assert_eq!(a.checked_rem_euclid(b), None);
                assert_eq!(a.overflowing_rem(b), (u7::ZERO, true));
                assert_eq!(a.overflowing_rem_euclid(b), (u7::ZERO, true));
                should_panic(|| a.wrapping_rem(b));
                should_panic(|| a.wrapping_rem_euclid(b));
            } else {
                let rem = u7::new(i % j).unwrap();
                let reme = u7::new(i.rem_euclid(j)).unwrap();
                assert_eq!(a % b, rem);
                assert_eq!(a.rem_euclid(b), reme);
                assert_eq!(a.checked_rem(b), Some(rem));
                assert_eq!(a.checked_rem_euclid(b), Some(reme));
                assert_eq!(a.overflowing_rem(b), (rem, false));
                assert_eq!(a.overflowing_rem_euclid(b), (reme, false));
                assert_eq!(a.wrapping_rem(b), rem);
                assert_eq!(a.wrapping_rem_euclid(b), reme);
            }
        }
    }
}

#[test]
fn shl() {
    should_panic(|| u0::ZERO << 0u32);
    assert_eq!(u0::ZERO.checked_shl(0), None);
    assert_eq!(u0::ZERO.overflowing_shl(0), (u0::ZERO, true));
    assert_eq!(u0::ZERO.unbounded_shl(0), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_shl(0), u0::ZERO);
    should_panic(|| u0::ZERO << 1u32);
    assert_eq!(u0::ZERO.checked_shl(1), None);
    assert_eq!(u0::ZERO.overflowing_shl(1), (u0::ZERO, true));
    assert_eq!(u0::ZERO.unbounded_shl(1), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_shl(1), u0::ZERO);

    should_panic(|| i0::ZERO << 0u32);
    assert_eq!(i0::ZERO.checked_shl(0), None);
    assert_eq!(i0::ZERO.overflowing_shl(0), (i0::ZERO, true));
    assert_eq!(i0::ZERO.unbounded_shl(0), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_shl(0), i0::ZERO);
    should_panic(|| i0::ZERO << 1u32);
    assert_eq!(i0::ZERO.checked_shl(1), None);
    assert_eq!(i0::ZERO.overflowing_shl(1), (i0::ZERO, true));
    assert_eq!(i0::ZERO.unbounded_shl(1), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_shl(1), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 << 0u32, n1);
    should_panic(|| n1 << 1u32);
    assert_eq!(zo << 0u32, zo);
    should_panic(|| zo << 1u32);
    assert_eq!(n1.checked_shl(0), Some(n1));
    assert_eq!(n1.checked_shl(1), None);
    assert_eq!(zo.checked_shl(0), Some(zo));
    assert_eq!(zo.checked_shl(1), None);
    assert_eq!(n1.overflowing_shl(0), (n1, false));
    assert_eq!(n1.overflowing_shl(1), (zo, true));
    assert_eq!(zo.overflowing_shl(0), (zo, false));
    assert_eq!(zo.overflowing_shl(1), (zo, true));
    assert_eq!(n1.unbounded_shl(0), n1);
    assert_eq!(n1.unbounded_shl(1), zo);
    assert_eq!(zo.unbounded_shl(0), zo);
    assert_eq!(zo.unbounded_shl(1), zo);
    assert_eq!(n1.wrapping_shl(0), n1);
    assert_eq!(n1.wrapping_shl(1), n1);
    assert_eq!(zo.wrapping_shl(0), zo);
    assert_eq!(zo.wrapping_shl(1), zo);

    for i in -64..=63 {
        let v = i7::new(i).unwrap();
        for shift in 0..=6u32 {
            let shl = i7::cast_new(i << shift);
            assert_eq!(v << shift, shl);
            assert_eq!(v.checked_shl(shift), Some(shl));
            assert_eq!(v.overflowing_shl(shift), (shl, false));
            assert_eq!(v.unbounded_shl(shift), shl);
            assert_eq!(v.wrapping_shl(shift), shl);
        }
        for shift in 7..=32u32 {
            should_panic(|| v << shift);
            assert_eq!(v.checked_shl(shift), None);
            assert_eq!(v.overflowing_shl(shift), (i7::ZERO, true));
            assert_eq!(v.unbounded_shl(shift), i7::ZERO);
            let _ = v.wrapping_shl(shift);
        }
    }

    for i in 0..=127 {
        let v = u7::new(i).unwrap();
        for shift in 0..=6u32 {
            let shl = u7::cast_new(i << shift);
            assert_eq!(v << shift, shl);
            assert_eq!(v.checked_shl(shift), Some(shl));
            assert_eq!(v.overflowing_shl(shift), (shl, false));
            assert_eq!(v.unbounded_shl(shift), shl);
            assert_eq!(v.wrapping_shl(shift), shl);
        }
        for shift in 7..=32u32 {
            should_panic(|| v << shift);
            assert_eq!(v.checked_shl(shift), None);
            assert_eq!(v.overflowing_shl(shift), (u7::ZERO, true));
            assert_eq!(v.unbounded_shl(shift), u7::ZERO);
            let _ = v.wrapping_shl(shift);
        }
    }
}

#[test]
fn shr() {
    should_panic(|| u0::ZERO >> 0u32);
    assert_eq!(u0::ZERO.checked_shr(0), None);
    assert_eq!(u0::ZERO.overflowing_shr(0), (u0::ZERO, true));
    assert_eq!(u0::ZERO.unbounded_shr(0), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_shr(0), u0::ZERO);
    should_panic(|| u0::ZERO >> 1u32);
    assert_eq!(u0::ZERO.checked_shr(1), None);
    assert_eq!(u0::ZERO.overflowing_shr(1), (u0::ZERO, true));
    assert_eq!(u0::ZERO.unbounded_shr(1), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_shr(1), u0::ZERO);

    should_panic(|| i0::ZERO >> 0u32);
    assert_eq!(i0::ZERO.checked_shr(0), None);
    assert_eq!(i0::ZERO.overflowing_shr(0), (i0::ZERO, true));
    assert_eq!(i0::ZERO.unbounded_shr(0), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_shr(0), i0::ZERO);
    should_panic(|| i0::ZERO >> 1u32);
    assert_eq!(i0::ZERO.checked_shr(1), None);
    assert_eq!(i0::ZERO.overflowing_shr(1), (i0::ZERO, true));
    assert_eq!(i0::ZERO.unbounded_shr(1), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_shr(1), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 >> 0u32, n1);
    should_panic(|| n1 >> 1u32);
    assert_eq!(zo >> 0u32, zo);
    should_panic(|| zo >> 1u32);
    assert_eq!(n1.checked_shr(0), Some(n1));
    assert_eq!(n1.checked_shr(1), None);
    assert_eq!(zo.checked_shr(0), Some(zo));
    assert_eq!(zo.checked_shr(1), None);
    assert_eq!(n1.overflowing_shr(0), (n1, false));
    assert_eq!(n1.overflowing_shr(1), (n1, true));
    assert_eq!(zo.overflowing_shr(0), (zo, false));
    assert_eq!(zo.overflowing_shr(1), (zo, true));
    assert_eq!(n1.unbounded_shr(0), n1);
    assert_eq!(n1.unbounded_shr(1), n1);
    assert_eq!(zo.unbounded_shr(0), zo);
    assert_eq!(zo.unbounded_shr(1), zo);
    assert_eq!(n1.wrapping_shr(0), n1);
    assert_eq!(n1.wrapping_shr(1), n1);
    assert_eq!(zo.wrapping_shr(0), zo);
    assert_eq!(zo.wrapping_shr(1), zo);

    for i in -64..=63 {
        let v = i7::new(i).unwrap();
        for shift in 0..=6u32 {
            let shr = i7::new(i >> shift).unwrap();
            assert_eq!(v >> shift, shr);
            assert_eq!(v.checked_shr(shift), Some(shr));
            assert_eq!(v.overflowing_shr(shift), (shr, false));
            assert_eq!(v.unbounded_shr(shift), shr);
            assert_eq!(v.wrapping_shr(shift), shr);
        }
        for shift in 7..=32u32 {
            should_panic(|| v >> shift);
            assert_eq!(v.checked_shr(shift), None);
            assert_eq!(v.overflowing_shr(shift), (if i < 0 { i7::ALL_ONE } else { i7::ZERO }, true));
            assert_eq!(v.unbounded_shr(shift), i7::new(i.unbounded_shr(shift)).unwrap());
            assert_eq!(v.wrapping_shr(shift), i7::new(i.wrapping_shr(shift)).unwrap());
        }
    }

    for i in 0..=127 {
        let v = u7::new(i).unwrap();
        for shift in 0..=6u32 {
            let shr = u7::new(i >> shift).unwrap();
            assert_eq!(v >> shift, shr);
            assert_eq!(v.checked_shr(shift), Some(shr));
            assert_eq!(v.overflowing_shr(shift), (shr, false));
            assert_eq!(v.unbounded_shr(shift), shr);
            assert_eq!(v.wrapping_shr(shift), shr);
        }
        for shift in 7..=32u32 {
            should_panic(|| v >> shift);
            assert_eq!(v.checked_shr(shift), None);
            assert_eq!(v.overflowing_shr(shift), (u7::ZERO, true));
            assert_eq!(v.unbounded_shr(shift), u7::ZERO);
            assert_eq!(v.wrapping_shr(shift), u7::new(i.wrapping_shr(shift)).unwrap());
        }
    }
}

#[test]
fn sub() {
    assert_eq!(u0::ZERO - u0::ZERO, u0::ZERO);
    assert_eq!(u0::ZERO.checked_sub(u0::ZERO), Some(u0::ZERO));
    assert_eq!(u0::ZERO.overflowing_sub(u0::ZERO), (u0::ZERO, false));
    assert_eq!(u0::ZERO.saturating_sub(u0::ZERO), u0::ZERO);
    assert_eq!(unsafe { u0::ZERO.unchecked_sub(u0::ZERO) }, u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_sub(u0::ZERO), u0::ZERO);

    assert_eq!(i0::ZERO - i0::ZERO, i0::ZERO);
    assert_eq!(i0::ZERO.checked_sub(i0::ZERO), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_sub(i0::ZERO), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_sub(i0::ZERO), i0::ZERO);
    assert_eq!(unsafe { i0::ZERO.unchecked_sub(i0::ZERO) }, i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_sub(i0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1 - zo, n1);
    assert_eq!(n1 - n1, zo);
    assert_eq!(zo - zo, zo);
    should_panic(|| zo - n1);
    assert_eq!(n1.checked_sub(zo), Some(n1));
    assert_eq!(n1.checked_sub(n1), Some(zo));
    assert_eq!(zo.checked_sub(zo), Some(zo));
    assert_eq!(zo.checked_sub(n1), None);
    assert_eq!(n1.overflowing_sub(zo), (n1, false));
    assert_eq!(n1.overflowing_sub(n1), (zo, false));
    assert_eq!(zo.overflowing_sub(zo), (zo, false));
    assert_eq!(zo.overflowing_sub(n1), (n1, true));
    assert_eq!(n1.saturating_sub(zo), n1);
    assert_eq!(n1.saturating_sub(n1), zo);
    assert_eq!(zo.saturating_sub(zo), zo);
    assert_eq!(zo.saturating_sub(n1), zo);
    assert_eq!(unsafe { n1.unchecked_sub(zo) }, n1);
    assert_eq!(unsafe { n1.unchecked_sub(n1) }, zo);
    assert_eq!(unsafe { zo.unchecked_sub(zo) }, zo);
    should_panic(|| unsafe { zo.unchecked_sub(n1) });
    assert_eq!(n1.wrapping_sub(zo), n1);
    assert_eq!(n1.wrapping_sub(n1), zo);
    assert_eq!(zo.wrapping_sub(zo), zo);
    assert_eq!(zo.wrapping_sub(n1), n1);

    for i in -64..=63 {
        let start = if i < 0 { -64 } else { i - 63 };
        let end = if i < 0 { i + 64 } else { 63 };
        let a = i7::new(i).unwrap();
        for j in -64..start {
            let b = i7::new(j).unwrap();
            let wsub = i7::cast_new(i - j);
            should_panic(|| a - b);
            assert_eq!(a.checked_sub(b), None);
            assert_eq!(a.overflowing_sub(b), (wsub, true));
            assert_eq!(a.saturating_sub(b), i7::MAX);
            should_panic(|| unsafe { a.unchecked_sub(b) });
            assert_eq!(a.wrapping_sub(b), wsub);
        }
        for j in start..=end {
            let b = i7::new(j).unwrap();
            let sub = i7::new(i - j).unwrap();
            assert_eq!(a - b, sub);
            assert_eq!(a.checked_sub(b), Some(sub));
            assert_eq!(a.overflowing_sub(b), (sub, false));
            assert_eq!(a.saturating_sub(b), sub);
            assert_eq!(unsafe { a.unchecked_sub(b) }, sub);
            assert_eq!(a.wrapping_sub(b), sub);
        }
        for j in end + 1..=63 {
            let b = i7::new(j).unwrap();
            let wsub = i7::cast_new(i - j);
            should_panic(|| a - b);
            assert_eq!(a.checked_sub(b), None);
            assert_eq!(a.overflowing_sub(b), (wsub, true));
            assert_eq!(a.saturating_sub(b), i7::MIN);
            should_panic(|| unsafe { a.unchecked_sub(b) });
            assert_eq!(a.wrapping_sub(b), wsub);
        }
    }

    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        for j in 0..=i {
            let b = u7::new(j).unwrap();
            let sub = u7::new(i - j).unwrap();
            assert_eq!(a - b, sub);
            assert_eq!(a.checked_sub(b), Some(sub));
            assert_eq!(a.overflowing_sub(b), (sub, false));
            assert_eq!(a.saturating_sub(b), sub);
            assert_eq!(unsafe { a.unchecked_sub(b) }, sub);
            assert_eq!(a.wrapping_sub(b), sub);
        }
        for j in i + 1..=127 {
            let b = u7::new(j).unwrap();
            let wsub = u7::new(i.wrapping_sub(j) & 0b0111_1111).unwrap();
            should_panic(|| a - b);
            assert_eq!(a.checked_sub(b), None);
            assert_eq!(a.overflowing_sub(b), (wsub, true));
            assert_eq!(a.saturating_sub(b), u7::MIN);
            should_panic(|| unsafe { a.unchecked_sub(b) });
            assert_eq!(a.wrapping_sub(b), wsub);
        }
    }
}

#[test]
fn neg() {
    assert_eq!(!u0::ZERO, u0::ZERO);
    assert_eq!(u0::ZERO.checked_neg(), Some(u0::ZERO));
    assert_eq!(u0::ZERO.overflowing_neg(), (u0::ZERO, false));
    assert_eq!(u0::ZERO.wrapping_neg(), u0::ZERO);

    assert_eq!(!i0::ZERO, i0::ZERO);
    assert_eq!(i0::ZERO.checked_neg(), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_neg(), (i0::ZERO, false));
    assert_eq!(i0::ZERO.wrapping_neg(), i0::ZERO);
    assert_eq!(i0::ZERO.saturating_neg(), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| -n1);
    assert_eq!(-zo, zo);
    assert_eq!(n1.checked_neg(), None);
    assert_eq!(zo.checked_neg(), Some(zo));
    assert_eq!(n1.overflowing_neg(), (n1, true));
    assert_eq!(zo.overflowing_neg(), (zo, false));
    assert_eq!(n1.wrapping_neg(), n1);
    assert_eq!(zo.wrapping_neg(), zo);
    assert_eq!(n1.saturating_neg(), zo);
    assert_eq!(zo.saturating_neg(), zo);

    let v = i7::new(-64).unwrap();
    should_panic(|| -v);
    assert_eq!(v.checked_neg(), None);
    assert_eq!(v.overflowing_neg(), (i7::MIN, true));
    assert_eq!(v.wrapping_neg(), i7::MIN);
    assert_eq!(v.saturating_neg(), i7::MAX);
    for i in -63..=63 {
        let a = i7::new(i).unwrap();
        let b = i7::new(-i).unwrap();
        assert_eq!(-a, b);
        assert_eq!(a.checked_neg(), Some(b));
        assert_eq!(a.overflowing_neg(), (b, false));
        assert_eq!(a.wrapping_neg(), b);
        assert_eq!(a.saturating_neg(), b);
    }

    assert_eq!(u7::ZERO.checked_neg(), Some(u7::ZERO));
    assert_eq!(u7::ZERO.overflowing_neg(), (u7::ZERO, false));
    for i in 1..=127 {
        let a = u7::new(i).unwrap();
        let wneg = u7::new((-(i as i8)) as u8 & 0b0111_1111).unwrap();
        assert_eq!(a.checked_neg(), None);
        assert_eq!(a.overflowing_neg(), (wneg, true));
        assert_eq!(a.wrapping_neg(), wneg);
    }
}

#[test]
fn abs_diff() {
    assert_eq!(u0::ZERO.abs_diff(u0::ZERO), u0::ZERO);

    assert_eq!(i0::ZERO.abs_diff(i0::ZERO), u0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.abs_diff(n1), u1::new(0).unwrap());
    assert_eq!(n1.abs_diff(zo), u1::new(1).unwrap());
    assert_eq!(zo.abs_diff(zo), u1::new(0).unwrap());
    assert_eq!(zo.abs_diff(n1), u1::new(1).unwrap());

    for i in -64..=63 {
        for j in -64..=63 {
            assert_eq!(i7::new(i).unwrap().abs_diff(i7::new(j).unwrap()), u7::new(i.abs_diff(j)).unwrap());
        }
    }

    for i in 0..=127 {
        for j in 0..=127 {
            assert_eq!(u7::new(i).unwrap().abs_diff(u7::new(j).unwrap()), u7::new(i.abs_diff(j)).unwrap());
        }
    }
}

#[test]
fn pow() {
    should_panic(|| u0::ZERO.pow(0));
    assert_eq!(u0::ZERO.checked_pow(0), None);
    assert_eq!(u0::ZERO.overflowing_pow(0), (u0::ZERO, true));
    assert_eq!(u0::ZERO.saturating_pow(0), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_pow(0), u0::ZERO);
    for i in 1..20 {
        assert_eq!(u0::ZERO.pow(i), u0::ZERO);
        assert_eq!(u0::ZERO.checked_pow(i), Some(u0::ZERO));
        assert_eq!(u0::ZERO.overflowing_pow(i), (u0::ZERO, false));
        assert_eq!(u0::ZERO.saturating_pow(i), u0::ZERO);
        assert_eq!(u0::ZERO.wrapping_pow(i), u0::ZERO);
    }

    should_panic(|| i0::ZERO.pow(0));
    assert_eq!(i0::ZERO.checked_pow(0), None);
    assert_eq!(i0::ZERO.overflowing_pow(0), (i0::ZERO, true));
    assert_eq!(i0::ZERO.saturating_pow(0), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_pow(0), i0::ZERO);
    for i in 1..20 {
        assert_eq!(i0::ZERO.pow(i), i0::ZERO);
        assert_eq!(i0::ZERO.checked_pow(i), Some(i0::ZERO));
        assert_eq!(i0::ZERO.overflowing_pow(i), (i0::ZERO, false));
        assert_eq!(i0::ZERO.saturating_pow(i), i0::ZERO);
        assert_eq!(i0::ZERO.wrapping_pow(i), i0::ZERO);
    }

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1.pow(0));
    should_panic(|| zo.pow(0));
    assert_eq!(n1.checked_pow(0), None);
    assert_eq!(zo.checked_pow(0), None);
    assert_eq!(n1.overflowing_pow(0), (n1, true));
    assert_eq!(zo.overflowing_pow(0), (n1, true));
    assert_eq!(n1.saturating_pow(0), zo);
    assert_eq!(zo.saturating_pow(0), zo);
    assert_eq!(n1.wrapping_pow(0), n1);
    assert_eq!(zo.wrapping_pow(0), n1);
    for i in 1..20 {
        if i & 1 == 0 {
            should_panic(|| n1.pow(i));
            assert_eq!(n1.checked_pow(i), None);
            assert_eq!(n1.overflowing_pow(i), (n1, true));
            assert_eq!(n1.saturating_pow(i), zo);
            assert_eq!(n1.wrapping_pow(i), n1);
        } else {
            assert_eq!(n1.pow(i), n1);
            assert_eq!(n1.checked_pow(i), Some(n1));
            assert_eq!(n1.overflowing_pow(i), (n1, false));
            assert_eq!(n1.saturating_pow(i), n1);
            assert_eq!(n1.wrapping_pow(i), n1);
        }
        assert_eq!(zo.pow(i), zo);
        assert_eq!(zo.checked_pow(i), Some(zo));
        assert_eq!(zo.overflowing_pow(i), (zo, false));
        assert_eq!(zo.saturating_pow(i), zo);
        assert_eq!(zo.wrapping_pow(i), zo);
    }

    for i in -1..=1 {
        let a = i7::new(i).unwrap();
        for j in 0..20 {
            let pow = i7::cast_new(i.pow(j));
            assert_eq!(a.pow(j), pow);
            assert_eq!(a.checked_pow(j), Some(pow));
            assert_eq!(a.overflowing_pow(j), (pow, false));
            assert_eq!(a.saturating_pow(j), pow);
            assert_eq!(a.wrapping_pow(j), pow);
        }
    }
    for i in -64..=63 {
        let a = i7::new(i).unwrap();
        assert_eq!(a.pow(0), i7::ONE);
        assert_eq!(a.pow(1), a);
        assert_eq!(a.checked_pow(0), Some(i7::ONE));
        assert_eq!(a.checked_pow(1), Some(a));
        assert_eq!(a.overflowing_pow(1), (a, false));
        assert_eq!(a.saturating_pow(0), i7::ONE);
        assert_eq!(a.saturating_pow(1), a);
        assert_eq!(a.wrapping_pow(0), i7::ONE);
        assert_eq!(a.wrapping_pow(1), a);
    }
    for i in [-64, 63, -9, 8] {
        let a = i7::new(i).unwrap();
        let wpow = i7::cast_new(i.wrapping_mul(i));
        should_panic(|| a.pow(2));
        assert_eq!(a.checked_pow(2), None);
        assert_eq!(a.overflowing_pow(2), (wpow, true));
        assert_eq!(a.saturating_pow(2), i7::MAX);
        assert_eq!(a.wrapping_pow(2), wpow);
    }
    for i in -7..=7 {
        let a = i7::new(i).unwrap();
        let pow = i7::new(i.pow(2)).unwrap();
        assert_eq!(a.pow(2), pow);
        assert_eq!(a.checked_pow(2), Some(pow));
        assert_eq!(a.overflowing_pow(2), (pow, false));
        assert_eq!(a.saturating_pow(2), pow);
        assert_eq!(a.wrapping_pow(2), pow);
    }
    for i in -4..=3 {
        let a = i7::new(i).unwrap();
        let pow = i7::new(i.pow(3)).unwrap();
        assert_eq!(a.pow(3), pow);
        assert_eq!(a.checked_pow(3), Some(pow));
        assert_eq!(a.overflowing_pow(3), (pow, false));
        assert_eq!(a.saturating_pow(3), pow);
        assert_eq!(a.wrapping_pow(3), pow);
    }

    for i in [0, 1] {
        let a = u7::new(i).unwrap();
        for j in 0..20 {
            let pow = u7::new(i.pow(j)).unwrap();
            assert_eq!(a.pow(j), pow);
            assert_eq!(a.checked_pow(j), Some(pow));
            assert_eq!(a.overflowing_pow(j), (pow, false));
            assert_eq!(a.saturating_pow(j), pow);
            assert_eq!(a.wrapping_pow(j), pow);
        }
    }
    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        assert_eq!(a.pow(0), u7::ONE);
        assert_eq!(a.pow(1), a);
        assert_eq!(a.checked_pow(0), Some(u7::ONE));
        assert_eq!(a.checked_pow(1), Some(a));
        assert_eq!(a.overflowing_pow(1), (a, false));
        assert_eq!(a.saturating_pow(0), u7::ONE);
        assert_eq!(a.saturating_pow(1), a);
        assert_eq!(a.wrapping_pow(0), u7::ONE);
        assert_eq!(a.wrapping_pow(1), a);
    }
    for i in 12..=127 {
        let a = u7::new(i).unwrap();
        let wpow = u7::cast_new(i.wrapping_mul(i));
        should_panic(|| a.pow(2));
        assert_eq!(a.checked_pow(2), None);
        assert_eq!(a.overflowing_pow(2), (wpow, true));
        assert_eq!(a.saturating_pow(2), u7::MAX);
        assert_eq!(a.wrapping_pow(2), wpow);
    }
    for i in 0..=11 {
        let a = u7::new(i).unwrap();
        let pow = u7::new(i.pow(2)).unwrap();
        assert_eq!(a.pow(2), pow);
        assert_eq!(a.checked_pow(2), Some(pow));
        assert_eq!(a.overflowing_pow(2), (pow, false));
        assert_eq!(a.saturating_pow(2), pow);
        assert_eq!(a.wrapping_pow(2), pow);
    }
    for i in 0..=5 {
        let a = u7::new(i).unwrap();
        let pow = u7::new(i.pow(3)).unwrap();
        assert_eq!(a.pow(3), pow);
        assert_eq!(a.checked_pow(3), Some(pow));
        assert_eq!(a.overflowing_pow(3), (pow, false));
        assert_eq!(a.saturating_pow(3), pow);
        assert_eq!(a.wrapping_pow(3), pow);
    }
}

#[test]
fn count_ones() {
    assert_eq!(u0::ZERO.count_ones(), 0);

    assert_eq!(i0::ZERO.count_ones(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.count_ones(), 1);
    assert_eq!(zo.count_ones(), 0);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().count_ones(), (i & 0b0111_1111).count_ones());
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().count_ones(), (i & 0b0111_1111).count_ones());
    }
}

#[test]
fn count_zeros() {
    assert_eq!(u0::ZERO.count_zeros(), 0);

    assert_eq!(i0::ZERO.count_zeros(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.count_zeros(), 0);
    assert_eq!(zo.count_zeros(), 1);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().count_zeros(), (i | 0b1000_0000u8 as i8).count_zeros());
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().count_zeros(), (i | 0b1000_0000).count_zeros());
    }
}

#[test]
fn leading_ones() {
    assert_eq!(u0::ZERO.leading_ones(), 0);

    assert_eq!(i0::ZERO.leading_ones(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.leading_ones(), 1);
    assert_eq!(zo.leading_ones(), 0);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().leading_ones(), (i << 1).leading_ones());
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().leading_ones(), (i << 1).leading_ones());
    }
}

#[test]
fn leading_zeros() {
    assert_eq!(u0::ZERO.leading_zeros(), 0);

    assert_eq!(i0::ZERO.leading_zeros(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.leading_zeros(), 0);
    assert_eq!(zo.leading_zeros(), 1);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().leading_zeros(), (i & 0b0111_1111).leading_zeros() - 1);
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().leading_zeros(), (i & 0b0111_1111).leading_zeros() - 1);
    }
}

#[test]
fn midpoint() {
    assert_eq!(u0::ZERO.midpoint(u0::ZERO), u0::ZERO);

    assert_eq!(i0::ZERO.midpoint(i0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(zo.midpoint(zo), zo);
    assert_eq!(zo.midpoint(n1), zo);
    assert_eq!(n1.midpoint(zo), zo);
    assert_eq!(n1.midpoint(n1), n1);

    for i in -64..=63 {
        let a = i7::new(i).unwrap();
        for j in -64..=63 {
            let b = i7::new(j).unwrap();
            assert_eq!(a.midpoint(b), i7::new(i.midpoint(j)).unwrap());
        }
    }

    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        for j in 0..=127 {
            let b = u7::new(j).unwrap();
            assert_eq!(a.midpoint(b), u7::new(i.midpoint(j)).unwrap());
        }
    }
}

#[test]
fn reverse_bits() {
    assert_eq!(u0::ZERO.reverse_bits(), u0::ZERO);

    assert_eq!(i0::ZERO.reverse_bits(), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.reverse_bits(), n1);
    assert_eq!(zo.reverse_bits(), zo);

    assert_eq!(i7::new(0b0_010_1110).unwrap().reverse_bits(), i7::new(0b0_0111_010).unwrap());
    assert_eq!(i7::new(0b1_110_1110u8 as i8).unwrap().reverse_bits(), i7::new(0b0_0111_011).unwrap());
    assert_eq!(i7::new(0b0_010_1111).unwrap().reverse_bits(), i7::new(0b1_1111_010u8 as i8).unwrap());

    assert_eq!(u7::new(0b0_010_1110).unwrap().reverse_bits(), u7::new(0b0_0111_010).unwrap());
    assert_eq!(u7::new(0b0_110_1110).unwrap().reverse_bits(), u7::new(0b0_0111_011).unwrap());
    assert_eq!(u7::new(0b0_010_1111).unwrap().reverse_bits(), u7::new(0b0_1111_010).unwrap());
}

#[test]
fn rotate_left() {
    for i in 0..20 {
        assert_eq!(u0::ZERO.rotate_left(i), u0::ZERO);
    }

    for i in 0..20 {
        assert_eq!(i0::ZERO.rotate_left(i), i0::ZERO);
    }

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    for i in 0..20 {
        assert_eq!(n1.rotate_left(i), n1);
        assert_eq!(zo.rotate_left(i), zo);
    }

    for i in 0..3 {
        let i = i * 7;
        let a = i7::new(0b0_010_1110).unwrap();
        assert_eq!(a.rotate_left(i + 0), a);
        assert_eq!(a.rotate_left(i + 1), i7::new(0b1_101_1100u8 as i8).unwrap());
        assert_eq!(a.rotate_left(i + 2), i7::new(0b0_011_1001).unwrap());
        assert_eq!(a.rotate_left(i + 3), i7::new(0b1_111_0010u8 as i8).unwrap());
        assert_eq!(a.rotate_left(i + 6), i7::new(0b0_001_0111).unwrap());

        let a = i7::new(0b1_110_1110u8 as i8).unwrap();
        assert_eq!(a.rotate_left(i + 0), a);
        assert_eq!(a.rotate_left(i + 1), i7::new(0b1_101_1101u8 as i8).unwrap());
        assert_eq!(a.rotate_left(i + 2), i7::new(0b0_011_1011).unwrap());
        assert_eq!(a.rotate_left(i + 3), i7::new(0b1_111_0110u8 as i8).unwrap());
        assert_eq!(a.rotate_left(i + 6), i7::new(0b0_011_0111).unwrap());

        let a = u7::new(0b0_010_1110).unwrap();
        assert_eq!(a.rotate_left(i + 0), a);
        assert_eq!(a.rotate_left(i + 1), u7::new(0b0_101_1100).unwrap());
        assert_eq!(a.rotate_left(i + 2), u7::new(0b0_011_1001).unwrap());
        assert_eq!(a.rotate_left(i + 3), u7::new(0b0_111_0010).unwrap());
        assert_eq!(a.rotate_left(i + 6), u7::new(0b0_001_0111).unwrap());

        let a = u7::new(0b0_110_1110).unwrap();
        assert_eq!(a.rotate_left(i + 0), a);
        assert_eq!(a.rotate_left(i + 1), u7::new(0b0_101_1101).unwrap());
        assert_eq!(a.rotate_left(i + 2), u7::new(0b0_011_1011).unwrap());
        assert_eq!(a.rotate_left(i + 3), u7::new(0b0_111_0110).unwrap());
        assert_eq!(a.rotate_left(i + 6), u7::new(0b0_011_0111).unwrap());
    }
}

#[test]
fn rotate_right() {
    for i in 0..20 {
        assert_eq!(u0::ZERO.rotate_right(i), u0::ZERO);
    }

    for i in 0..20 {
        assert_eq!(i0::ZERO.rotate_right(i), i0::ZERO);
    }

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    for i in 0..20 {
        assert_eq!(n1.rotate_right(i), n1);
        assert_eq!(zo.rotate_right(i), zo);
    }

    for i in 0..3 {
        let i = i * 7;
        let a = i7::new(0b0_010_1110).unwrap();
        assert_eq!(a.rotate_right(i + 0), a);
        assert_eq!(a.rotate_right(i + 1), i7::new(0b0_001_0111).unwrap());
        assert_eq!(a.rotate_right(i + 2), i7::new(0b1_100_1011u8 as i8).unwrap());
        assert_eq!(a.rotate_right(i + 3), i7::new(0b1_110_0101u8 as i8).unwrap());
        assert_eq!(a.rotate_right(i + 6), i7::new(0b1_101_1100u8 as i8).unwrap());

        let a = i7::new(0b1_110_1110u8 as i8).unwrap();
        assert_eq!(a.rotate_right(i + 0), a);
        assert_eq!(a.rotate_right(i + 1), i7::new(0b0_011_0111).unwrap());
        assert_eq!(a.rotate_right(i + 2), i7::new(0b1_101_1011u8 as i8).unwrap());
        assert_eq!(a.rotate_right(i + 3), i7::new(0b1_110_1101u8 as i8).unwrap());
        assert_eq!(a.rotate_right(i + 6), i7::new(0b1_101_1101u8 as i8).unwrap());

        let a = u7::new(0b0_010_1110).unwrap();
        assert_eq!(a.rotate_right(i + 0), a);
        assert_eq!(a.rotate_right(i + 1), u7::new(0b0_001_0111).unwrap());
        assert_eq!(a.rotate_right(i + 2), u7::new(0b0_100_1011).unwrap());
        assert_eq!(a.rotate_right(i + 3), u7::new(0b0_110_0101).unwrap());
        assert_eq!(a.rotate_right(i + 6), u7::new(0b0_101_1100).unwrap());

        let a = u7::new(0b0_110_1110).unwrap();
        assert_eq!(a.rotate_right(i + 0), a);
        assert_eq!(a.rotate_right(i + 1), u7::new(0b0_011_0111).unwrap());
        assert_eq!(a.rotate_right(i + 2), u7::new(0b0_101_1011).unwrap());
        assert_eq!(a.rotate_right(i + 3), u7::new(0b0_110_1101).unwrap());
        assert_eq!(a.rotate_right(i + 6), u7::new(0b0_101_1101).unwrap());
    }
}

#[test]
fn trailing_ones() {
    assert_eq!(u0::ZERO.trailing_ones(), 0);

    assert_eq!(i0::ZERO.trailing_ones(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.trailing_ones(), 1);
    assert_eq!(zo.trailing_ones(), 0);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().trailing_ones(), (i & 0b0111_1111).trailing_ones());
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().trailing_ones(), i.trailing_ones());
    }
}

#[test]
fn trailing_zeros() {
    assert_eq!(u0::ZERO.trailing_zeros(), 0);

    assert_eq!(i0::ZERO.trailing_zeros(), 0);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(n1.trailing_zeros(), 0);
    assert_eq!(zo.trailing_zeros(), 1);

    for i in -64..=63 {
        assert_eq!(i7::new(i).unwrap().trailing_zeros(), if i == 0 { 7 } else { i.trailing_zeros() });
    }

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().trailing_zeros(), if i == 0 { 7 } else { i.trailing_zeros() });
    }
}

#[test]
fn abs() {
    assert_eq!(i0::ZERO.abs(), i0::ZERO);
    assert_eq!(i0::ZERO.checked_abs(), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_abs(), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_abs(), i0::ZERO);
    assert_eq!(i0::ZERO.unsigned_abs(), u0::ZERO);
    assert_eq!(i0::ZERO.wrapping_abs(), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    should_panic(|| n1.abs());
    assert_eq!(zo.abs(), zo);
    assert_eq!(n1.checked_abs(), None);
    assert_eq!(zo.checked_abs(), Some(zo));
    assert_eq!(n1.overflowing_abs(), (n1, true));
    assert_eq!(zo.overflowing_abs(), (zo, false));
    assert_eq!(n1.saturating_abs(), zo);
    assert_eq!(zo.saturating_abs(), zo);
    assert_eq!(n1.unsigned_abs(), u1::ONE);
    assert_eq!(zo.unsigned_abs(), u1::ZERO);
    assert_eq!(n1.wrapping_abs(), n1);
    assert_eq!(zo.wrapping_abs(), zo);

    should_panic(|| i7::MIN.abs());
    assert_eq!(i7::MIN.overflowing_abs(), (i7::MIN, true));
    assert_eq!(i7::MIN.saturating_abs(), i7::MAX);
    assert_eq!(i7::MIN.unsigned_abs(), u7::new(64).unwrap());
    assert_eq!(i7::MIN.wrapping_abs(), i7::MIN);
    for i in -63..=63 {
        let a = i7::new(i).unwrap();
        let abs = i7::new(i.abs()).unwrap();
        assert_eq!(a.abs(), abs);
        assert_eq!(a.checked_abs(), Some(abs));
        assert_eq!(a.overflowing_abs(), (abs, false));
        assert_eq!(a.saturating_abs(), abs);
        assert_eq!(a.unsigned_abs(), u7::new(i.unsigned_abs()).unwrap());
        assert_eq!(a.wrapping_abs(), abs);
    }
}

#[test]
fn add_unsigned() {
    assert_eq!(i0::ZERO.checked_add_unsigned(u0::ZERO), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_add_unsigned(u0::ZERO), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_add_unsigned(u0::ZERO), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_add_unsigned(u0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(zo.checked_add_unsigned(u1::ZERO), Some(zo));
    assert_eq!(zo.checked_add_unsigned(u1::ONE), None);
    assert_eq!(n1.checked_add_unsigned(u1::ZERO), Some(n1));
    assert_eq!(n1.checked_add_unsigned(u1::ONE), Some(zo));
    assert_eq!(zo.overflowing_add_unsigned(u1::ZERO), (zo, false));
    assert_eq!(zo.overflowing_add_unsigned(u1::ONE), (n1, true));
    assert_eq!(n1.overflowing_add_unsigned(u1::ZERO), (n1, false));
    assert_eq!(n1.overflowing_add_unsigned(u1::ONE), (zo, false));
    assert_eq!(zo.saturating_add_unsigned(u1::ZERO), zo);
    assert_eq!(zo.saturating_add_unsigned(u1::ONE), zo);
    assert_eq!(n1.saturating_add_unsigned(u1::ZERO), n1);
    assert_eq!(n1.saturating_add_unsigned(u1::ONE), zo);
    assert_eq!(zo.wrapping_add_unsigned(u1::ZERO), zo);
    assert_eq!(zo.wrapping_add_unsigned(u1::ONE), n1);
    assert_eq!(n1.wrapping_add_unsigned(u1::ZERO), n1);
    assert_eq!(n1.wrapping_add_unsigned(u1::ONE), zo);

    for i in -64..=63 {
        let end = if i < 0 { 63 } else { 63 - i };
        let a = i7::new(i).unwrap();
        for j in 0..=end {
            let b = u7::new(j as u8).unwrap();
            let sum = i7::new(i.checked_add_unsigned(j as u8).unwrap()).unwrap();
            assert_eq!(a.checked_add_unsigned(b), Some(sum));
            assert_eq!(a.overflowing_add_unsigned(b), (sum, false));
            assert_eq!(a.saturating_add_unsigned(b), sum);
            assert_eq!(a.wrapping_add_unsigned(b), sum);
        }
        for j in end + 1..=63 {
            let b = u7::new(j as u8).unwrap();
            let wadd = i7::cast_new(i.checked_add_unsigned(j as u8).unwrap());
            assert_eq!(a.checked_add_unsigned(b), None);
            assert_eq!(a.overflowing_add_unsigned(b), (wadd, true));
            assert_eq!(a.saturating_add_unsigned(b), i7::MAX);
            assert_eq!(a.wrapping_add_unsigned(b), wadd);
        }
    }
}

#[test]
fn sub_unsigned() {
    assert_eq!(i0::ZERO.checked_sub_unsigned(u0::ZERO), Some(i0::ZERO));
    assert_eq!(i0::ZERO.overflowing_sub_unsigned(u0::ZERO), (i0::ZERO, false));
    assert_eq!(i0::ZERO.saturating_sub_unsigned(u0::ZERO), i0::ZERO);
    assert_eq!(i0::ZERO.wrapping_sub_unsigned(u0::ZERO), i0::ZERO);

    let n1 = i1::new(-1).unwrap();
    let zo = i1::new(0).unwrap();
    assert_eq!(zo.checked_sub_unsigned(u1::ZERO), Some(zo));
    assert_eq!(zo.checked_sub_unsigned(u1::ONE), Some(n1));
    assert_eq!(n1.checked_sub_unsigned(u1::ZERO), Some(n1));
    assert_eq!(n1.checked_sub_unsigned(u1::ONE), None);
    assert_eq!(zo.overflowing_sub_unsigned(u1::ZERO), (zo, false));
    assert_eq!(zo.overflowing_sub_unsigned(u1::ONE), (n1, false));
    assert_eq!(n1.overflowing_sub_unsigned(u1::ZERO), (n1, false));
    assert_eq!(n1.overflowing_sub_unsigned(u1::ONE), (zo, true));
    assert_eq!(zo.saturating_sub_unsigned(u1::ZERO), zo);
    assert_eq!(zo.saturating_sub_unsigned(u1::ONE), n1);
    assert_eq!(n1.saturating_sub_unsigned(u1::ZERO), n1);
    assert_eq!(n1.saturating_sub_unsigned(u1::ONE), n1);
    assert_eq!(zo.wrapping_sub_unsigned(u1::ZERO), zo);
    assert_eq!(zo.wrapping_sub_unsigned(u1::ONE), n1);
    assert_eq!(n1.wrapping_sub_unsigned(u1::ZERO), n1);
    assert_eq!(n1.wrapping_sub_unsigned(u1::ONE), zo);

    for i in -64..=63 {
        let end = if i < 0 { i + 64 } else { 63 };
        let a = i7::new(i).unwrap();
        for j in 0..=end {
            let b = u7::new(j as u8).unwrap();
            let sub = i7::new(i.checked_sub_unsigned(j as u8).unwrap()).unwrap();
            assert_eq!(a.checked_sub_unsigned(b), Some(sub));
            assert_eq!(a.overflowing_sub_unsigned(b), (sub, false));
            assert_eq!(a.saturating_sub_unsigned(b), sub);
            assert_eq!(a.wrapping_sub_unsigned(b), sub);
        }
        for j in end + 1..=63 {
            let b = u7::new(j as u8).unwrap();
            let wsub = i7::cast_new(i.checked_sub_unsigned(j as u8).unwrap());
            assert_eq!(a.checked_sub_unsigned(b), None);
            assert_eq!(a.overflowing_sub_unsigned(b), (wsub, true));
            assert_eq!(a.saturating_sub_unsigned(b), i7::MIN);
            assert_eq!(a.wrapping_sub_unsigned(b), wsub);
        }
    }
}

#[test]
fn is_negative() {
    assert_eq!(i0::ZERO.is_negative(), false);

    assert_eq!(i1::new(-1).unwrap().is_negative(), true);
    assert_eq!(i1::ZERO.is_negative(), false);

    for i in -64..=-1 {
        assert_eq!(i7::new(i).unwrap().is_negative(), true);
    }
    for i in 0..=63 {
        assert_eq!(i7::new(i).unwrap().is_negative(), false);
    }
}

#[test]
fn is_positive() {
    assert_eq!(i0::ZERO.is_positive(), false);

    assert_eq!(i1::new(-1).unwrap().is_positive(), false);
    assert_eq!(i1::ZERO.is_positive(), false);

    for i in -64..=-1 {
        assert_eq!(i7::new(i).unwrap().is_positive(), false);
    }
    assert_eq!(i7::ZERO.is_positive(), false);
    for i in 1..=63 {
        assert_eq!(i7::new(i).unwrap().is_positive(), true);
    }
}

#[test]
fn signum() {
    assert_eq!(i0::ZERO.signum(), i8::ZERO);

    assert_eq!(i1::new(-1).unwrap().signum(), -1);
    assert_eq!(i1::ZERO.signum(), 0);

    for i in -64..=-1 {
        assert_eq!(i7::new(i).unwrap().signum(), -1);
    }
    assert_eq!(i7::ZERO.signum(), 0);
    for i in 1..=63 {
        assert_eq!(i7::new(i).unwrap().signum(), 1);
    }
}

#[test]
fn add_signed() {
    assert_eq!(u0::ZERO.checked_add_signed(i0::ZERO), Some(u0::ZERO));
    assert_eq!(u0::ZERO.overflowing_add_signed(i0::ZERO), (u0::ZERO, false));
    assert_eq!(u0::ZERO.saturating_add_signed(i0::ZERO), u0::ZERO);
    assert_eq!(u0::ZERO.wrapping_add_signed(i0::ZERO), u0::ZERO);

    let n1 = i1::new(-1).unwrap();
    assert_eq!(u1::ZERO.checked_add_signed(i1::ZERO), Some(u1::ZERO));
    assert_eq!(u1::ZERO.checked_add_signed(n1), None);
    assert_eq!(u1::ONE.checked_add_signed(i1::ZERO), Some(u1::ONE));
    assert_eq!(u1::ONE.checked_add_signed(n1), Some(u1::ZERO));
    assert_eq!(u1::ZERO.overflowing_add_signed(i1::ZERO), (u1::ZERO, false));
    assert_eq!(u1::ZERO.overflowing_add_signed(n1), (u1::ONE, true));
    assert_eq!(u1::ONE.overflowing_add_signed(i1::ZERO), (u1::ONE, false));
    assert_eq!(u1::ONE.overflowing_add_signed(n1), (u1::ZERO, false));
    assert_eq!(u1::ZERO.saturating_add_signed(i1::ZERO), u1::ZERO);
    assert_eq!(u1::ZERO.saturating_add_signed(n1), u1::ZERO);
    assert_eq!(u1::ONE.saturating_add_signed(i1::ZERO), u1::ONE);
    assert_eq!(u1::ONE.saturating_add_signed(n1), u1::ZERO);
    assert_eq!(u1::ZERO.wrapping_add_signed(i1::ZERO), u1::ZERO);
    assert_eq!(u1::ZERO.wrapping_add_signed(n1), u1::ONE);
    assert_eq!(u1::ONE.wrapping_add_signed(i1::ZERO), u1::ONE);
    assert_eq!(u1::ONE.wrapping_add_signed(n1), u1::ZERO);

    for i in 0..=127 {
        let start = if i <= 64 { -(i as i8) } else { -64 };
        let end = if i <= 64 { 63 } else { 127 - i as i8 };
        let a = u7::new(i).unwrap();
        for j in -64..start {
            let b = i7::new(j).unwrap();
            let wsum = u7::cast_new(i.wrapping_add_signed(j));
            assert_eq!(a.checked_add_signed(b), None);
            assert_eq!(a.overflowing_add_signed(b), (wsum, true));
            assert_eq!(a.saturating_add_signed(b), u7::ZERO);
            assert_eq!(a.wrapping_add_signed(b), wsum);
        }
        for j in start..=end {
            let b = i7::new(j).unwrap();
            let sum = u7::new(i.checked_add_signed(j).unwrap()).unwrap();
            assert_eq!(a.checked_add_signed(b), Some(sum));
            assert_eq!(a.overflowing_add_signed(b), (sum, false));
            assert_eq!(a.saturating_add_signed(b), sum);
            assert_eq!(a.wrapping_add_signed(b), sum);
        }
        for j in end + 1..=63 {
            let b = i7::new(j).unwrap();
            let wsum = u7::cast_new(i.checked_add_signed(j).unwrap());
            assert_eq!(a.checked_add_signed(b), None);
            assert_eq!(a.overflowing_add_signed(b), (wsum, true));
            assert_eq!(a.saturating_add_signed(b), u7::MAX);
            assert_eq!(a.wrapping_add_signed(b), wsum);
        }
    }
}

#[test]
fn next_multiple_of() {
    should_panic(|| u0::ZERO.next_multiple_of(u0::ZERO));
    assert_eq!(u0::ZERO.checked_next_multiple_of(u0::ZERO), None);

    should_panic(|| u1::ZERO.next_multiple_of(u1::ZERO));
    assert_eq!(u1::ZERO.next_multiple_of(u1::ONE), u1::ZERO);
    should_panic(|| u1::ONE.next_multiple_of(u1::ZERO));
    assert_eq!(u1::ONE.next_multiple_of(u1::ONE), u1::ONE);
    assert_eq!(u1::ZERO.checked_next_multiple_of(u1::ZERO), None);
    assert_eq!(u1::ZERO.checked_next_multiple_of(u1::ONE), Some(u1::ZERO));
    assert_eq!(u1::ONE.checked_next_multiple_of(u1::ZERO), None);
    assert_eq!(u1::ONE.checked_next_multiple_of(u1::ONE), Some(u1::ONE));

    for i in 0..=127 {
        let a = u7::new(i).unwrap();
        should_panic(|| a.next_multiple_of(u7::ZERO));
        for j in 1..=127 {
            let b = u7::new(j).unwrap();
            if let Some(mp) = i.checked_next_multiple_of(j)
                && let Some(mp) = u7::new(mp)
            {
                assert_eq!(a.next_multiple_of(b), mp);
                assert_eq!(a.checked_next_multiple_of(b), Some(mp));
            } else {
                should_panic(|| a.next_multiple_of(b));
                assert_eq!(a.checked_next_multiple_of(b), None);
            }
        }
    }
}

#[test]
fn next_power_of_two() {
    should_panic(|| u0::ZERO.next_power_of_two());
    assert_eq!(u0::ZERO.checked_next_power_of_two(), None);

    assert_eq!(u1::ZERO.next_power_of_two(), u1::ONE);
    assert_eq!(u1::ONE.next_power_of_two(), u1::ONE);
    assert_eq!(u1::ZERO.checked_next_power_of_two(), Some(u1::ONE));
    assert_eq!(u1::ONE.checked_next_power_of_two(), Some(u1::ONE));

    for i in 0..=64 {
        let a = u7::new(i).unwrap();
        let pow = u7::new(i.next_power_of_two()).unwrap();
        assert_eq!(a.next_power_of_two(), pow);
        assert_eq!(a.checked_next_power_of_two(), Some(pow));
    }
    for i in 65..=127 {
        let a = u7::new(i).unwrap();
        should_panic(|| a.next_power_of_two());
        assert_eq!(a.checked_next_power_of_two(), None);
    }
}

#[test]
fn is_power_of_two() {
    assert_eq!(u0::ZERO.is_power_of_two(), false);

    assert_eq!(u1::ZERO.is_power_of_two(), false);
    assert_eq!(u1::ONE.is_power_of_two(), true);

    for i in 0..=127 {
        assert_eq!(u7::new(i).unwrap().is_power_of_two(), i.is_power_of_two());
    }
}
