use std::{
    cmp::{PartialEq, Eq, PartialOrd,Ordering},
};

/// An Option-like type which is only compares equal if it contains a value.
#[derive(Debug, Copy, Clone, Hash)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum MaybeCmp<T> {
    Just(T),
    Nothing,
}




impl<T> Eq for MaybeCmp<T>
where
    T: Eq,
{}

impl<T> PartialEq for MaybeCmp<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (MaybeCmp::Just(l),MaybeCmp::Just(r))=>l==r,
            _=>false,
        }
    }
}

impl<T> PartialOrd for MaybeCmp<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self,other) {
            (MaybeCmp::Just(l),MaybeCmp::Just(r))=>l.partial_cmp(r),
            _=>None,
        }
    }
}


//#[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod tets{
    use super::*;
    #[test]
    fn comparisons(){
        assert_eq!(MaybeCmp::Just(0),MaybeCmp::Just(0));

        assert_ne!(MaybeCmp::Just(0),MaybeCmp::Just(1));
        assert_ne!(MaybeCmp::Nothing,MaybeCmp::Just(0));
        assert_ne!(MaybeCmp::Just(0),MaybeCmp::Nothing);
        assert_ne!(MaybeCmp::<()>::Nothing,MaybeCmp::Nothing);

        assert_eq!(MaybeCmp::Just(0).partial_cmp(&MaybeCmp::Just(0)),Some(Ordering::Equal));
        assert_eq!(MaybeCmp::Just(0).partial_cmp(&MaybeCmp::Just(1)),Some(Ordering::Less));
        assert_eq!(MaybeCmp::Nothing.partial_cmp(&MaybeCmp::Just(0)),None);
        assert_eq!(MaybeCmp::Just(0).partial_cmp(&MaybeCmp::Nothing),None);
        assert_eq!(MaybeCmp::<()>::Nothing.partial_cmp(&MaybeCmp::Nothing),None);
    }
}