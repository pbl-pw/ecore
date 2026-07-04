#![cfg(not(miri))]

use ecore::{
    bitint::{BitInt, i7, i11, i30, u7, u11, u30},
    int::{BasicInt, CInt},
};

#[allow(non_camel_case_types)]
type i0 = BitInt<isize, 0>;
#[allow(non_camel_case_types)]
type u0 = BitInt<usize, 0>;
#[allow(non_camel_case_types)]
type is7 = BitInt<isize, 7>;
#[allow(non_camel_case_types)]
type us7 = BitInt<usize, 7>;
#[allow(non_camel_case_types)]
type is30 = BitInt<i64, 30>;
#[allow(non_camel_case_types)]
type us30 = BitInt<u64, 30>;

#[test]
#[cfg_attr(miri, ignore)]
fn i16_cast_as() {
    for v in i16::MIN..=i16::MAX {
        assert_eq!(CInt::ex_cast_as::<i16, i0>(v), i0::ZERO);
        assert_eq!(CInt::ex_cast_as::<i16, u0>(v), u0::ZERO);
        assert_eq!(CInt::cast_as::<i16, i8>(v), v as i8);
        assert_eq!(CInt::ex_cast_as::<i16, i7>(v), i7::cast_new(v as i8));
        assert_eq!(CInt::ex_cast_as::<i16, is7>(v), is7::cast_new(v as isize));
        assert_eq!(CInt::cast_as::<i16, u8>(v), v as u8);
        assert_eq!(CInt::ex_cast_as::<i16, u7>(v), u7::cast_new(v as u8));
        assert_eq!(CInt::ex_cast_as::<i16, us7>(v), us7::cast_new(v as usize));
        assert_eq!(CInt::cast_as::<i16, i16>(v), v);
        assert_eq!(CInt::cast_as::<i16, u16>(v), v as u16);
        assert_eq!(CInt::cast_as::<i16, i32>(v), v as i32);
        assert_eq!(CInt::ex_cast_as::<i16, i30>(v), i30::cast_new(v as i32));
        assert_eq!(CInt::ex_cast_as::<i16, is30>(v), is30::cast_new(v as i64));
        assert_eq!(CInt::cast_as::<i16, u32>(v), v as u32);
        assert_eq!(CInt::ex_cast_as::<i16, u30>(v), u30::cast_new(v as u32));
        assert_eq!(CInt::ex_cast_as::<i16, us30>(v), us30::cast_new(v as u64));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_cast_as() {
    for v in u16::MIN..=u16::MAX {
        assert_eq!(CInt::ex_cast_as::<u16, i0>(v), i0::ZERO);
        assert_eq!(CInt::ex_cast_as::<u16, u0>(v), u0::ZERO);
        assert_eq!(CInt::cast_as::<u16, i8>(v), v as i8);
        assert_eq!(CInt::ex_cast_as::<u16, i7>(v), i7::cast_new(v as i8));
        assert_eq!(CInt::ex_cast_as::<u16, is7>(v), is7::cast_new(v as isize));
        assert_eq!(CInt::cast_as::<u16, u8>(v), v as u8);
        assert_eq!(CInt::ex_cast_as::<u16, u7>(v), u7::cast_new(v as u8));
        assert_eq!(CInt::ex_cast_as::<u16, us7>(v), us7::cast_new(v as usize));
        assert_eq!(CInt::cast_as::<u16, i16>(v), v as i16);
        assert_eq!(CInt::cast_as::<u16, u16>(v), v);
        assert_eq!(CInt::cast_as::<u16, i32>(v), v as i32);
        assert_eq!(CInt::ex_cast_as::<u16, i30>(v), i30::cast_new(v as i32));
        assert_eq!(CInt::ex_cast_as::<u16, is30>(v), is30::cast_new(v as i64));
        assert_eq!(CInt::cast_as::<u16, u32>(v), v as u32);
        assert_eq!(CInt::ex_cast_as::<u16, u30>(v), u30::cast_new(v as u32));
        assert_eq!(CInt::ex_cast_as::<u16, us30>(v), us30::cast_new(v as u64));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn i0_cast_as() {
    assert_eq!(CInt::ex_cast_as::<i0, i0>(i0::ZERO), i0::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u0>(i0::ZERO), u0::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, i8>(i0::ZERO), i8::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, i7>(i0::ZERO), i7::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, is7>(i0::ZERO), is7::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u8>(i0::ZERO), u8::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u7>(i0::ZERO), u7::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, us7>(i0::ZERO), us7::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, i16>(i0::ZERO), i16::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u16>(i0::ZERO), u16::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, i32>(i0::ZERO), i32::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, i30>(i0::ZERO), i30::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, is30>(i0::ZERO), is30::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u32>(i0::ZERO), u32::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, u30>(i0::ZERO), u30::ZERO);
    assert_eq!(CInt::ex_cast_as::<i0, us30>(i0::ZERO), us30::ZERO);
}

#[test]
#[cfg_attr(miri, ignore)]
fn u0_cast_as() {
    assert_eq!(CInt::ex_cast_as::<u0, i0>(u0::ZERO), i0::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u0>(u0::ZERO), u0::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, i8>(u0::ZERO), i8::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, i7>(u0::ZERO), i7::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, is7>(u0::ZERO), is7::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u8>(u0::ZERO), u8::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u7>(u0::ZERO), u7::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, us7>(u0::ZERO), us7::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, i16>(u0::ZERO), i16::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u16>(u0::ZERO), u16::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, i32>(u0::ZERO), i32::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, i30>(u0::ZERO), i30::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, is30>(u0::ZERO), is30::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u32>(u0::ZERO), u32::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, u30>(u0::ZERO), u30::ZERO);
    assert_eq!(CInt::ex_cast_as::<u0, us30>(u0::ZERO), us30::ZERO);
}

#[test]
#[cfg_attr(miri, ignore)]
fn i11_cast_as() {
    for v in i11::MIN.value()..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::ex_cast_as::<i11, i11>(v), v);
        assert_eq!(CInt::ex_cast_as::<i11, i0>(v), i0::ZERO);
        assert_eq!(CInt::ex_cast_as::<i11, u0>(v), u0::ZERO);
        assert_eq!(CInt::ex_cast_as::<i11, i8>(v), v.value() as i8);
        assert_eq!(CInt::ex_cast_as::<i11, i7>(v), i7::cast_new(v.value() as i8));
        assert_eq!(CInt::ex_cast_as::<i11, is7>(v), is7::cast_new(v.value() as isize));
        assert_eq!(CInt::ex_cast_as::<i11, u8>(v), v.value() as u8);
        assert_eq!(CInt::ex_cast_as::<i11, u7>(v), u7::cast_new(v.value() as u8));
        assert_eq!(CInt::ex_cast_as::<i11, us7>(v), us7::cast_new(v.value() as usize));
        assert_eq!(CInt::ex_cast_as::<i11, i16>(v), v.value());
        assert_eq!(CInt::ex_cast_as::<i11, u16>(v), v.value() as u16);
        assert_eq!(CInt::ex_cast_as::<i11, i32>(v), v.value() as i32);
        assert_eq!(CInt::ex_cast_as::<i11, i30>(v), i30::cast_new(v.value() as i32));
        assert_eq!(CInt::ex_cast_as::<i11, is30>(v), is30::cast_new(v.value() as i64));
        assert_eq!(CInt::ex_cast_as::<i11, u32>(v), v.value() as u32);
        assert_eq!(CInt::ex_cast_as::<i11, u30>(v), u30::cast_new(v.value() as u32));
        assert_eq!(CInt::ex_cast_as::<i11, us30>(v), us30::cast_new(v.value() as u64));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u11_cast_as() {
    for v in u11::MIN.value()..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::ex_cast_as::<u11, u11>(v), v);
        assert_eq!(CInt::ex_cast_as::<u11, i0>(v), i0::ZERO);
        assert_eq!(CInt::ex_cast_as::<u11, u0>(v), u0::ZERO);
        assert_eq!(CInt::ex_cast_as::<u11, i8>(v), v.value() as i8);
        assert_eq!(CInt::ex_cast_as::<u11, i7>(v), i7::cast_new(v.value() as i8));
        assert_eq!(CInt::ex_cast_as::<u11, is7>(v), is7::cast_new(v.value() as isize));
        assert_eq!(CInt::ex_cast_as::<u11, u8>(v), v.value() as u8);
        assert_eq!(CInt::ex_cast_as::<u11, u7>(v), u7::cast_new(v.value() as u8));
        assert_eq!(CInt::ex_cast_as::<u11, us7>(v), us7::cast_new(v.value() as usize));
        assert_eq!(CInt::ex_cast_as::<u11, i16>(v), v.value() as i16);
        assert_eq!(CInt::ex_cast_as::<u11, u16>(v), v.value());
        assert_eq!(CInt::ex_cast_as::<u11, i32>(v), v.value() as i32);
        assert_eq!(CInt::ex_cast_as::<u11, i30>(v), i30::cast_new(v.value() as i32));
        assert_eq!(CInt::ex_cast_as::<u11, is30>(v), is30::cast_new(v.value() as i64));
        assert_eq!(CInt::ex_cast_as::<u11, u32>(v), v.value() as u32);
        assert_eq!(CInt::ex_cast_as::<u11, u30>(v), u30::cast_new(v.value() as u32));
        assert_eq!(CInt::ex_cast_as::<u11, us30>(v), us30::cast_new(v.value() as u64));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn i16_checked_cast_as() {
    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, i0>(v), None, "v = {}", v);
    }
    assert_eq!(CInt::checked_cast_as::<i16, i0>(0), Some(i0::ZERO), "v = {}", 0);
    for v in 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i0>(v), None, "v = {}", v);
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u0>(v), None, "v = {}", v);
    }
    assert_eq!(CInt::checked_cast_as::<i16, u0>(0), Some(u0::ZERO), "v = {}", 0);
    for v in 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u0>(v), None, "v = {}", v);
    }

    for v in i16::MIN..i7::MIN.value() as i16 {
        assert_eq!(CInt::checked_cast_as::<i16, i7>(v), None, "v = {}", v);
    }
    for v in i7::MIN.value()..=i7::MAX.value() {
        assert_eq!(CInt::checked_cast_as::<i16, i7>(v as i16), Some(i7::new(v).unwrap()));
    }
    for v in i7::MAX.value() as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i7>(v), None);
    }

    for v in i16::MIN..is7::MIN.value() as i16 {
        assert_eq!(CInt::checked_cast_as::<i16, is7>(v), None, "v = {}", v);
    }
    for v in is7::MIN.value()..=is7::MAX.value() {
        assert_eq!(CInt::checked_cast_as::<i16, is7>(v as i16), Some(is7::new(v).unwrap()));
    }
    for v in is7::MAX.value() as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, is7>(v), None);
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u7>(v), None);
    }
    for v in 0..=u7::MAX.value() {
        assert_eq!(CInt::checked_cast_as::<i16, u7>(v as i16), Some(u7::new(v).unwrap()));
    }
    for v in u7::MAX.value() as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u7>(v), None);
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, us7>(v), None);
    }
    for v in 0..=us7::MAX.value() {
        assert_eq!(CInt::checked_cast_as::<i16, us7>(v as i16), Some(us7::new(v).unwrap()));
    }
    for v in us7::MAX.value() as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, us7>(v), None);
    }

    for v in i16::MIN..i8::MIN as i16 {
        assert_eq!(CInt::checked_cast_as::<i16, i8>(v), None);
    }
    for v in i8::MIN..=i8::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i8>(v as i16), Some(v));
    }
    for v in i8::MAX as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i8>(v), None);
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u8>(v), None);
    }
    for v in 0..=u8::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u8>(v as i16), Some(v));
    }
    for v in u8::MAX as i16 + 1..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u8>(v), None);
    }

    for v in i16::MIN..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i16>(v), Some(v));
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u16>(v), None);
    }
    for v in 0..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u16>(v), Some(v as u16));
    }

    for v in i16::MIN..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i30>(v), Some(i30::new(v as i32).unwrap()));
    }

    for v in i16::MIN..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, is30>(v), Some(is30::new(v as i64).unwrap()));
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u30>(v), None);
    }
    for v in 0..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u30>(v), Some(u30::new(v as u32).unwrap()));
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, us30>(v), None);
    }
    for v in 0..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, us30>(v), Some(us30::new(v as u64).unwrap()));
    }

    for v in i16::MIN..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, i32>(v), Some(v as i32));
    }

    for v in i16::MIN..0 {
        assert_eq!(CInt::checked_cast_as::<i16, u32>(v), None);
    }
    for v in 0..=i16::MAX {
        assert_eq!(CInt::checked_cast_as::<i16, u32>(v), Some(v as u32));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u16_checked_cast_as() {
    assert_eq!(CInt::checked_cast_as::<u16, i0>(0), Some(i0::ZERO), "v = {}", 0);
    for v in 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, i0>(v), None, "v = {}", v);
    }

    assert_eq!(CInt::checked_cast_as::<u16, u0>(0), Some(u0::ZERO), "v = {}", 0);
    for v in 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, u0>(v), None, "v = {}", v);
    }

    for v in 0..=i7::MAX.value() as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, i7>(v), Some(i7::new(v as i8).unwrap()));
    }
    for v in i7::MAX.value() as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, i7>(v), None);
    }

    for v in 0..=is7::MAX.value() as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, is7>(v), Some(is7::new(v as isize).unwrap()));
    }
    for v in is7::MAX.value() as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, is7>(v), None);
    }

    for v in 0..=u7::MAX.value() as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, u7>(v), Some(u7::new(v as u8).unwrap()));
    }
    for v in u7::MAX.value() as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, u7>(v), None);
    }

    for v in 0..=us7::MAX.value() as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, us7>(v), Some(us7::new(v as usize).unwrap()));
    }
    for v in us7::MAX.value() as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, us7>(v), None);
    }

    for v in 0..=i8::MAX as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, i8>(v), Some(v as i8));
    }
    for v in i8::MAX as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, i8>(v), None);
    }

    for v in 0..=u8::MAX as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, u8>(v), Some(v as u8));
    }
    for v in u8::MAX as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, u8>(v), None);
    }

    for v in 0..=i16::MAX as u16 {
        assert_eq!(CInt::checked_cast_as::<u16, i16>(v), Some(v as i16));
    }
    for v in i16::MAX as u16 + 1..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, i16>(v), None);
    }

    for v in 0..=u16::MAX {
        assert_eq!(CInt::checked_cast_as::<u16, u16>(v), Some(v));
        assert_eq!(CInt::checked_cast_as::<u16, i30>(v), Some(i30::new(v as i32).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u16, is30>(v), Some(is30::new(v as i64).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u16, u30>(v), Some(u30::new(v as u32).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u16, us30>(v), Some(us30::new(v as u64).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u16, i32>(v), Some(v as i32));
        assert_eq!(CInt::checked_cast_as::<u16, u32>(v), Some(v as u32));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn i0_checked_cast_as() {
    assert_eq!(CInt::checked_cast_as::<i0, i0>(i0::ZERO), Some(i0::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u0>(i0::ZERO), Some(u0::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, i8>(i0::ZERO), Some(i8::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, i7>(i0::ZERO), Some(i7::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, is7>(i0::ZERO), Some(is7::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u8>(i0::ZERO), Some(u8::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u7>(i0::ZERO), Some(u7::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, us7>(i0::ZERO), Some(us7::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, i16>(i0::ZERO), Some(i16::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u16>(i0::ZERO), Some(u16::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, i32>(i0::ZERO), Some(i32::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, i30>(i0::ZERO), Some(i30::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, is30>(i0::ZERO), Some(is30::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u32>(i0::ZERO), Some(u32::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, u30>(i0::ZERO), Some(u30::ZERO));
    assert_eq!(CInt::checked_cast_as::<i0, us30>(i0::ZERO), Some(us30::ZERO));
}

#[test]
#[cfg_attr(miri, ignore)]
fn u0_checked_cast_as() {
    assert_eq!(CInt::checked_cast_as::<u0, i0>(u0::ZERO), Some(i0::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u0>(u0::ZERO), Some(u0::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, i8>(u0::ZERO), Some(i8::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, i7>(u0::ZERO), Some(i7::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, is7>(u0::ZERO), Some(is7::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u8>(u0::ZERO), Some(u8::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u7>(u0::ZERO), Some(u7::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, us7>(u0::ZERO), Some(us7::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, i16>(u0::ZERO), Some(i16::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u16>(u0::ZERO), Some(u16::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, i32>(u0::ZERO), Some(i32::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, i30>(u0::ZERO), Some(i30::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, is30>(u0::ZERO), Some(is30::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u32>(u0::ZERO), Some(u32::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, u30>(u0::ZERO), Some(u30::ZERO));
    assert_eq!(CInt::checked_cast_as::<u0, us30>(u0::ZERO), Some(us30::ZERO));
}

#[test]
#[cfg_attr(miri, ignore)]
fn i11_checked_cast_as() {
    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i0>(v), None, "v = {}", v);
    }
    assert_eq!(CInt::checked_cast_as::<i11, i0>(i11::ZERO), Some(i0::ZERO), "v = {}", 0);
    for v in 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i0>(v), None, "v = {}", v);
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u0>(v), None, "v = {}", v);
    }
    assert_eq!(CInt::checked_cast_as::<i11, u0>(i11::ZERO), Some(u0::ZERO), "v = {}", 0);
    for v in 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u0>(v), None, "v = {}", v);
    }

    for v in i11::MIN.value()..i7::MIN.value() as i16 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i7>(v), None, "v = {}", v);
    }
    for v in i7::MIN.value()..=i7::MAX.value() {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i7>(nv), Some(i7::new(v).unwrap()));
    }
    for v in i7::MAX.value() as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i7>(v), None);
    }

    for v in i11::MIN.value()..is7::MIN.value() as i16 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, is7>(v), None, "v = {}", v);
    }
    for v in is7::MIN.value()..=is7::MAX.value() {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, is7>(nv), Some(is7::new(v).unwrap()));
    }
    for v in is7::MAX.value() as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, is7>(v), None);
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u7>(v), None);
    }
    for v in 0..=u7::MAX.value() {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u7>(nv), Some(u7::new(v).unwrap()));
    }
    for v in u7::MAX.value() as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u7>(v), None);
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, us7>(v), None);
    }
    for v in 0..=us7::MAX.value() {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, us7>(nv), Some(us7::new(v).unwrap()));
    }
    for v in us7::MAX.value() as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, us7>(v), None);
    }

    for v in i11::MIN.value()..i8::MIN as i16 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i8>(v), None);
    }
    for v in i8::MIN..=i8::MAX {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i8>(nv), Some(v));
    }
    for v in i8::MAX as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i8>(v), None);
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u8>(v), None);
    }
    for v in 0..=u8::MAX {
        let nv = i11::new(v as i16).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u8>(nv), Some(v));
    }
    for v in u8::MAX as i16 + 1..=i11::MAX.value() {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u8>(v), None);
    }

    for v in i11::MIN.value()..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i16>(nv), Some(v));
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u16>(v), None);
    }
    for v in 0..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u16>(nv), Some(v as u16));
    }

    for v in i11::MIN.value()..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i30>(nv), Some(i30::new(v as i32).unwrap()));
    }

    for v in i11::MIN.value()..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, is30>(nv), Some(is30::new(v as i64).unwrap()));
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u30>(v), None);
    }
    for v in 0..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u30>(nv), Some(u30::new(v as u32).unwrap()));
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, us30>(v), None);
    }
    for v in 0..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, us30>(nv), Some(us30::new(v as u64).unwrap()));
    }

    for v in i11::MIN.value()..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, i32>(nv), Some(v as i32));
    }

    for v in i11::MIN.value()..0 {
        let v = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u32>(v), None);
    }
    for v in 0..=i11::MAX.value() {
        let nv = i11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<i11, u32>(nv), Some(v as u32));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn u11_checked_cast_as() {
    assert_eq!(CInt::checked_cast_as::<u11, i0>(u11::ZERO), Some(i0::ZERO), "v = {}", 0);
    for v in 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i0>(v), None, "v = {}", v);
    }

    assert_eq!(CInt::checked_cast_as::<u11, u0>(u11::ZERO), Some(u0::ZERO), "v = {}", 0);
    for v in 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, u0>(v), None, "v = {}", v);
    }

    for v in 0..=i7::MAX.value() as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i7>(nv), Some(i7::new(v as i8).unwrap()));
    }
    for v in i7::MAX.value() as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i7>(v), None);
    }

    for v in 0..=is7::MAX.value() as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, is7>(nv), Some(is7::new(v as isize).unwrap()));
    }
    for v in is7::MAX.value() as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, is7>(v), None);
    }

    for v in 0..=u7::MAX.value() as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, u7>(nv), Some(u7::new(v as u8).unwrap()));
    }
    for v in u7::MAX.value() as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, u7>(v), None);
    }

    for v in 0..=us7::MAX.value() as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, us7>(nv), Some(us7::new(v as usize).unwrap()));
    }
    for v in us7::MAX.value() as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, us7>(v), None);
    }

    for v in 0..=i8::MAX as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i8>(nv), Some(v as i8));
    }
    for v in i8::MAX as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i8>(v), None);
    }

    for v in 0..=u8::MAX as u16 {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, u8>(nv), Some(v as u8));
    }
    for v in u8::MAX as u16 + 1..=u11::MAX.value() {
        let v = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, u8>(v), None);
    }

    for v in 0..=u11::MAX.value() {
        let nv = u11::new(v).unwrap();
        assert_eq!(CInt::checked_cast_as::<u11, i16>(nv), Some(v as i16));
        assert_eq!(CInt::checked_cast_as::<u11, u16>(nv), Some(v));
        assert_eq!(CInt::checked_cast_as::<u11, i30>(nv), Some(i30::new(v as i32).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u11, is30>(nv), Some(is30::new(v as i64).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u11, u30>(nv), Some(u30::new(v as u32).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u11, us30>(nv), Some(us30::new(v as u64).unwrap()));
        assert_eq!(CInt::checked_cast_as::<u11, i32>(nv), Some(v as i32));
        assert_eq!(CInt::checked_cast_as::<u11, u32>(nv), Some(v as u32));
    }
}
