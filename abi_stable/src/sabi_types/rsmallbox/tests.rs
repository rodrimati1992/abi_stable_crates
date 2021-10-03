use super::*;

use std::sync::Arc;

#[test]
fn construct() {
    let arc = Arc::new(100);

    {
        let box_ = RSmallBox::<_, [usize; 4]>::new(arc.clone());

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), true);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), false);
        assert_eq!(Arc::strong_count(&arc), 2);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let box_ = RSmallBox::<_, [u8; 2]>::new(arc.clone());

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), false);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), true);
        assert_eq!(Arc::strong_count(&arc), 2);
    }

    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn from_move_ptr() {
    let arc = Arc::new(100);

    {
        let box_ = RBox::with_move_ptr(
            ManuallyDrop::new(RBox::new(arc.clone())),
            RSmallBox::<_, [u8; 2]>::from_move_ptr,
        );

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), false);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), true);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let box_ = RBox::with_move_ptr(
            ManuallyDrop::new(RBox::new(arc.clone())),
            RSmallBox::<_, [usize; 1]>::from_move_ptr,
        );

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), true);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), false);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }

    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn from_rbox() {
    let arc = Arc::new(100);

    {
        let box_: RSmallBox<_, [u8; 2]> = RBox::new(arc.clone()).into();

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), false);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), true);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let box_: RSmallBox<_, [usize; 1]> = RBox::new(arc.clone()).into();

        assert_eq!(&*box_, &arc);
        assert_eq!(box_.piped_ref(RSmallBox::is_inline), true);
        assert_eq!(box_.piped_ref(RSmallBox::is_heap_allocated), false);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }

    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn into_rbox() {
    let arc = Arc::new(100);

    {
        let box_: RBox<_> = RSmallBox::<_, [u8; 2]>::new(arc.clone()).into();

        assert_eq!(&*box_, &arc);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let box_: RBox<_> = RSmallBox::<_, [usize; 1]>::new(arc.clone()).into();

        assert_eq!(&*box_, &arc);
        assert_eq!(Arc::strong_count(&*box_), 2);
        assert_eq!(Arc::strong_count(&arc), 2);
    }

    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn into_inner() {
    let arc = Arc::new(100);
    {
        let box_ = RSmallBox::<_, [usize; 4]>::new(arc.clone());
        assert_eq!(Arc::strong_count(&arc), 2);
        assert_eq!(box_.piped(RSmallBox::into_inner), arc);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let box_ = RSmallBox::<_, [u8; 2]>::new(arc.clone());
        assert_eq!(Arc::strong_count(&arc), 2);
        assert_eq!(box_.piped(RSmallBox::into_inner), arc);
    }
    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn alignments() {
    use super::alignment::*;
    assert_eq!(mem::align_of::<AlignTo1<[u8; 1]>>(), 1);
    assert_eq!(mem::align_of::<AlignTo2<[u8; 1]>>(), 2);
    assert_eq!(mem::align_of::<AlignTo4<[u8; 1]>>(), 4);
    assert_eq!(mem::align_of::<AlignTo8<[u8; 1]>>(), 8);
    assert_eq!(mem::align_of::<AlignTo16<[u8; 1]>>(), 16);
    assert_eq!(mem::align_of::<AlignTo32<[u8; 1]>>(), 32);
    assert_eq!(mem::align_of::<AlignTo64<[u8; 1]>>(), 64);
    assert_eq!(mem::align_of::<AlignTo128<[u8; 1]>>(), 128);
}
