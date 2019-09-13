use std::marker::PhantomData;

use crate::{
    GetStaticEquivalent,
    StableAbi,
    InterfaceType,
    type_level::{
        impl_enum::{Implemented,Unimplemented}
    },
};


////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(
    Send,Sync,Clone,Default,
    Display,Debug,Serialize,Deserialize,
    Eq,PartialEq,Ord,PartialOrd,Hash,
    Iterator,DoubleEndedIterator,
    FmtWrite,
    IoWrite,IoSeek,IoRead,IoBufRead,Error
))]
pub struct AllTraitsImpld;

#[test]
fn assert_all_traits_impld(){
    let _:<AllTraitsImpld as InterfaceType>::Send               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Sync               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Clone              =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Default            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Display            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Debug              =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Serialize          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Deserialize        =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Eq                 =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Ord                =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::PartialOrd         =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Hash               =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Iterator           =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::DoubleEndedIterator=Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::FmtWrite           =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoWrite            =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoSeek             =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoRead             =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::IoBufRead          =Implemented::NEW;
    let _:<AllTraitsImpld as InterfaceType>::Error              =Implemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(StableAbi)]
// #[sabi(debug_print)]
#[sabi(impl_InterfaceType())]
pub struct NoTraitsImpld<T>(PhantomData<T>);


#[test]
fn assert_all_traits_unimpld(){
    let _:<NoTraitsImpld<()> as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<NoTraitsImpld<()> as InterfaceType>::Error              =Unimplemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Debug,Display))]
pub struct FmtInterface<T>(PhantomData<T>)
where T:std::fmt::Debug;


#[test]
fn assert_fmt_traits_impld(){
    let _:<FmtInterface<()> as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Display            =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Debug              =Implemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<FmtInterface<()> as InterfaceType>::Error              =Unimplemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Hash,Ord))]
pub struct HashOrdInterface<T>(PhantomData<T>)
where T:std::fmt::Debug;


#[test]
fn assert_hash_ord_impld(){
    let _:<HashOrdInterface<()> as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Eq                 =Implemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Ord                =Implemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::PartialOrd         =Implemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Hash               =Implemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<HashOrdInterface<()> as InterfaceType>::Error              =Unimplemented::NEW;
}



////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Eq))]
pub struct OnlyEq;

#[test]
fn assert_only_eq(){
    let _:<OnlyEq as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Eq                 =Implemented::NEW;
    let _:<OnlyEq as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<OnlyEq as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<OnlyEq as InterfaceType>::Error              =Unimplemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(PartialOrd))]
pub struct OnlyPartialOrd;

#[test]
fn assert_only_partial_ord(){
    let _:<OnlyPartialOrd as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::PartialEq          =Implemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::PartialOrd         =Implemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<OnlyPartialOrd as InterfaceType>::Error              =Unimplemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Error))]
pub struct OnlyError;

#[test]
fn assert_only_error(){
    let _:<OnlyError as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Display            =Implemented::NEW;
    let _:<OnlyError as InterfaceType>::Debug              =Implemented::NEW;
    let _:<OnlyError as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Iterator           =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<OnlyError as InterfaceType>::Error              =Implemented::NEW;
}



////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(Iterator))]
pub struct OnlyIter;

#[test]
fn assert_only_iter(){
    let _:<OnlyIter as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Iterator           =Implemented::NEW;
    let _:<OnlyIter as InterfaceType>::DoubleEndedIterator=Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<OnlyIter as InterfaceType>::Error              =Unimplemented::NEW;
}


////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(GetStaticEquivalent)]
#[sabi(impl_InterfaceType(DoubleEndedIterator))]
pub struct OnlyDEIter;

#[test]
fn assert_only_de_iter(){
    let _:<OnlyDEIter as InterfaceType>::Send               =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Sync               =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Clone              =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Default            =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Display            =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Debug              =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Serialize          =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Deserialize        =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Eq                 =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::PartialEq          =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Ord                =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::PartialOrd         =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Hash               =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Iterator           =Implemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::DoubleEndedIterator=Implemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::FmtWrite           =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::IoWrite            =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::IoSeek             =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::IoRead             =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::IoBufRead          =Unimplemented::NEW;
    let _:<OnlyDEIter as InterfaceType>::Error              =Unimplemented::NEW;
}


