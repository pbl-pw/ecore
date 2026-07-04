use ecore::{
    bitfld::prelude::*,
    bitint::{i2, i3, u1, u2, u3, u4, u6, u7},
    int::ranged::{RInt, RRU16},
    nbool::NBool,
};

#[bitfld(u1)]
pub enum EExhau {
    A = 1,
    B = 0,
}

#[bitfld(u2)]
#[derive(Debug, PartialEq, Eq)]
pub enum ENExhau {
    A = 1,
    B = 0,
}

#[bitfld(u32, overlay)]
#[derive(Debug, Default, Eq, PartialEq)]
struct TeBs {
    ok: bitfld!(u8, 0..=7),

    failed: bitfld!(u7, 8:7),

    nfal: bitfld!(Unchecked<RInt<u7, RRU16<0, 123>>>, 8..=14),

    pbool: bitfld!(bool, 8),

    nbool: bitfld!(NBool, 8),

    ary: bitfld!([bool; 4], 8..=11),

    exhau: bitfld!(EExhau, 8),

    nexhau: bitfld!(Unchecked<ENExhau>, 8..=9),

    neg: bitfld!(i3, 4:3),
}

#[test]
fn rw() {
    let bs = TeBs(0);
    assert_eq!(bs.ok().read(), 0);
    assert_eq!(bs.ok().with(255), TeBs(255));
    assert_eq!(bs.failed().with(u7::new(127).unwrap()), TeBs(127 << 8));
    assert_eq!(bs.nfal().with(Unchecked::from(123)), TeBs(123 << 8));
    assert_eq!(bs.pbool().with(true), TeBs(256));
    assert_eq!(bs.pbool().with(false), TeBs(0));
    assert_eq!(bs.nbool().with(true), TeBs(0));
    assert_eq!(bs.nbool().with(false), TeBs(256));
    assert_eq!(bs.nbool().with(false).ok().with(255), TeBs(511));
    assert_eq!(bs.ary().cget::<0>().with(true), TeBs(1 << 8));
    assert_eq!(bs.ary().cget::<2>().with(true), TeBs(1 << 10));
    assert_eq!(bs.exhau().with(EExhau::A), TeBs(1 << 8));
    assert_eq!(bs.nexhau().with(ENExhau::A), TeBs(1 << 8));

    assert_eq!(bs.neg().with(i3::MIN), TeBs(0b100 << 4));
    assert_eq!(TeBs(0b100 << 4).neg().read(), i3::MIN);
    assert_eq!(bs.neg().with(i3::MAX), TeBs(0b011 << 4));
    assert_eq!(TeBs(0b011 << 4).neg().read(), i3::MAX);

    assert_eq!(bs.ok().const_with(255), TeBs(255));
    assert_eq!(bs.failed().const_with(u7::new(127).unwrap()), TeBs(127 << 8));
    assert_eq!(bs.nfal().const_with(Unchecked::from(123)), TeBs(123 << 8));
    assert_eq!(bs.pbool().const_with(true), TeBs(256));
    assert_eq!(bs.pbool().const_with(false), TeBs(0));
    assert_eq!(bs.nbool().const_with(NBool::new(true)), TeBs(0));
    assert_eq!(bs.nbool().const_with(NBool::new(false)), TeBs(256));
    assert_eq!(bs.nbool().const_with(NBool::new(false)).ok().const_with(255), TeBs(511));
    assert_eq!(bs.ary()[0].const_with(true), TeBs(1 << 8));
    assert_eq!(bs.ary()[2].const_with(true), TeBs(1 << 10));
    assert_eq!(bs.exhau().const_with(EExhau::A), TeBs(1 << 8));
    assert_eq!(bs.nexhau().const_with(ENExhau::A), TeBs(1 << 8));

    assert_eq!(bs.neg().const_with(i3::MIN), TeBs(0b100 << 4));
    assert_eq!(TeBs(0b100 << 4).neg().const_read(), i3::MIN);
    assert_eq!(bs.neg().const_with(i3::MAX), TeBs(0b011 << 4));
    assert_eq!(TeBs(0b011 << 4).neg().const_read(), i3::MAX);

    let mut bs = TeBs(0);
    bs.mut_fld(TeBs::failed).write(u7::new(99).unwrap());
    assert_eq!(bs, TeBs(99 << 8));
    assert_eq!(bs.failed().read().value(), 99);
    bs.mut_fld(TeBs::failed).write(u7::new(127).unwrap());
    assert_eq!(bs, TeBs(127 << 8));
    bs = TeBs(127 << 8);
    assert_eq!(bs.failed().read().value(), 127);

    bs.mut_fld(TeBs::failed).as_mut().write(u7::new(34).unwrap());
    assert_eq!(bs, TeBs(34 << 8));
    assert_eq!(bs.failed().as_ref().read().value(), 34);
    bs.mut_fld(TeBs::failed).as_mut().write(u7::new(127).unwrap());
    assert_eq!(bs, TeBs(127 << 8));
    bs = TeBs(127 << 8);
    assert_eq!(bs.failed().as_ref().read().value(), 127);

    let mut bs = TeBs(0);
    bs.mut_fld(TeBs::neg).as_mut().write(i3::MIN);
    assert_eq!(bs, TeBs(0b100 << 4));
    assert_eq!(bs.neg().as_ref().read(), i3::MIN);
    bs.mut_fld(TeBs::neg).as_mut().write(i3::MAX);
    assert_eq!(bs, TeBs(0b011 << 4));
    assert_eq!(bs.neg().as_ref().read(), i3::MAX);

    #[cfg(not(miri))]
    {
        let mut bs = TeBs(0);
        bs.mut_fld(TeBs::failed).as_mut().as_mut().write(u7::new(34).unwrap());
        assert_eq!(bs, TeBs(34 << 8));
        assert_eq!(bs.failed().as_ref().as_ref().read().value(), 34);
        bs.mut_fld(TeBs::failed).as_mut().as_mut().write(u7::new(127).unwrap());
        assert_eq!(bs, TeBs(127 << 8));
        bs = TeBs(127 << 8);
        assert_eq!(bs.failed().as_ref().as_ref().read().value(), 127);

        let mut bs = TeBs(0);
        bs.mut_fld(TeBs::neg).as_mut().as_mut().write(i3::MIN);
        assert_eq!(bs, TeBs(0b100 << 4));
        assert_eq!(bs.neg().as_ref().as_ref().read(), i3::MIN);
        bs.mut_fld(TeBs::neg).as_mut().as_mut().write(i3::MAX);
        assert_eq!(bs, TeBs(0b011 << 4));
        assert_eq!(bs.neg().as_ref().as_ref().read(), i3::MAX);
    }
}

#[bitfld(u6, tag(u2), payload(u4))]
#[derive(PartialEq, Eq, Debug)]
enum ReLayout {
    Off,
    A(u3),
    B(Unchecked<ENExhau>),
    C(bool),
}

#[bitfld(u6, tag(u2, 4:2), payload(u4,0:4))]
#[derive(PartialEq, Eq, Debug)]
enum ReLayout1 {
    Off,
    A(u3),
    B(Unchecked<ENExhau>),
    C(bool),
}

#[bitfld(u8, tag(u2, 1..=2), payload(u4,4..8))]
#[derive(PartialEq, Eq, Debug)]
enum ReLayout2 {
    Off,
    A(u3),
    B(Unchecked<ENExhau>),
    C(bool),
}

#[bitfld(u6, tag(i2, 4..6), payload(u4, 0..4))]
#[derive(PartialEq, Eq, Debug)]
enum ReLayoutN {
    Off = 0,
    A(u3) = -1,
    B(Unchecked<ENExhau>) = -2,
    C(bool) = 1,
}

#[test]
fn enum_re_layout() {
    macro_rules! checkv {
        ($enum:ident, $($v:tt)+) => {
            assert_eq!(Uncheckable::try_from_raw_value(Uncheckable::raw_value($enum::$($v)+)), Ok($enum::$($v)+));
            assert_eq!($enum::try_from_bits($enum::$($v)+.into_bits()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 1, 4>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 1, 4>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 0, 2>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 0, 2>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 0, 9>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 0, 9>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 3, 11>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 3, 11>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 11, 7>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 11, 7>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 12, 6>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 12, 6>()), Ok($enum::$($v)+));
            assert_eq!(BitsEnumReLayout::<_, u16, 7, 0>::new($enum::$($v)+).get(), $enum::$($v)+);
            assert_eq!($enum::try_from_layout($enum::$($v)+.into_layout::<u16, 7, 0>()), Ok($enum::$($v)+));
        };
    }
    macro_rules! check {
        ($enum:ident) => {
            checkv!($enum, Off);
            for i in 0..8 {
                let v = u3::new(i).unwrap();
                checkv!($enum, A(v));
            }
            checkv!($enum, B(ENExhau::A.into()));
            checkv!($enum, B(ENExhau::B.into()));
            checkv!($enum, C(false));
            checkv!($enum, C(true));
        };
    }
    check!(ReLayout);
    check!(ReLayout1);
    check!(ReLayout2);
    check!(ReLayoutN);
}
