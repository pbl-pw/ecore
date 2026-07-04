type BitsRange = ecore::range::BitsRange<usize>;

#[test]
fn copy_le_bits() {
    let src = [0xFF; size_of::<usize>()];
    for offset in 0..=7 {
        for i in 1..=usize::BITS - offset {
            let mut tgt = [0; size_of::<usize>()];
            assert!(BitsRange::new(offset, i).unwrap().copy_le_bits(&src, &mut tgt).is_ok());
            assert_eq!(&tgt, &(usize::MAX >> (usize::BITS - i) << offset).to_le_bytes());
        }
    }
    let src = [0; size_of::<usize>()];
    for offset in 0..=7 {
        for i in 1..=usize::BITS - offset {
            let mut tgt = [0xFF; size_of::<usize>()];
            assert!(BitsRange::new(offset, i).unwrap().copy_le_bits(&src, &mut tgt).is_ok());
            assert_eq!(&tgt, &(!(usize::MAX >> (usize::BITS - i) << offset)).to_le_bytes());
        }
    }
}

#[test]
fn copy_be_bits() {
    let src = [0xFF; size_of::<usize>()];
    for offset in 0..=7 {
        for i in 1..=usize::BITS - offset {
            let mut tgt = [0; size_of::<usize>()];
            assert!(BitsRange::new(offset, i).unwrap().copy_be_bits(&src, &mut tgt).is_ok());
            assert_eq!(&tgt, &(usize::MAX >> (usize::BITS - i) << offset).to_be_bytes());
        }
    }
    let src = [0; size_of::<usize>()];
    for offset in 0..=7 {
        for i in 1..=usize::BITS - offset {
            let mut tgt = [0xFF; size_of::<usize>()];
            assert!(BitsRange::new(offset, i).unwrap().copy_be_bits(&src, &mut tgt).is_ok());
            assert_eq!(&tgt, &(!(usize::MAX >> (usize::BITS - i) << offset)).to_be_bytes());
        }
    }
}
