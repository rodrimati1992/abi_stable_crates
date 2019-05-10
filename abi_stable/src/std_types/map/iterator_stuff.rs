use super::*;

use crate::{
    erased_types::IteratorItem,
    type_level::bools::*,
};

macro_rules! declare_iter_interface {
    (
        $k:ident=>$v:ident;
        interface=$interface:ident;
        type Item=$item:ty;
    ) => (
        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(inside_abi_stable_crate)]
        pub struct $interface<$k,$v>(PhantomData<Tuple2<$k,$v>>);

        impl<$k,$v> $interface<$k,$v>{
            pub const NEW:Self=Self(PhantomData);
        }

        
        impl<'a,$k:'a,$v:'a> IteratorItem<'a> for $interface<$k,$v>{
            type Item=$item;
        }
    )
}


declare_iter_interface!{
    K=>V;
    interface=RefIterInterface;
    type Item=Tuple2<&'a K,&'a V>;
}

crate::impl_InterfaceType!{
    impl<K,V> crate::InterfaceType for RefIterInterface<K,V> {
        type Send=False;
        type Sync=False;
        type Iterator=True;
        type Clone=True;
    }
}



declare_iter_interface!{
    K=>V;
    interface=MutIterInterface;
    type Item=Tuple2<&'a K,&'a mut V>;
}

crate::impl_InterfaceType!{
    impl<K,V> crate::InterfaceType for MutIterInterface<K,V> {
        type Send=False;
        type Sync=False;
        type Iterator=True;
    }
}



declare_iter_interface!{
    K=>V;
    interface=ValIterInterface;
    type Item=Tuple2<K,V>;
    
}

crate::impl_InterfaceType!{
    impl<K,V> crate::InterfaceType for ValIterInterface<K,V> {
        type Send=False;
        type Sync=False;
        type Iterator=True;
    }
}


///////////////////////////////////////////////////////////////////////////////

type IntoIterInner<'a,K,V>=
    DynTrait<'a,RBox<()>,ValIterInterface<K,V>>;




#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct IntoIter<K,V>{
    iter:IntoIterInner<'static,u32,u32>,
    // _marker:PhantomData<Tuple2<K,V>>,
    _marker:PhantomData<Tuple3<K,V,UnsafeIgnoredType<std::rc::Rc<()>>>>,
}


impl<K,V> IntoIter<K,V>{
/**

# Safety

This must be called only in `ErasedMap::into_val`.
*/
    pub(super)unsafe fn new<'a>(iter:DynTrait<'a,RBox<()>,ValIterInterface<K,V>>)->Self
    where   
        K:'a,
        V:'a,
    {
        IntoIter{
            iter:mem::transmute::<IntoIterInner<'a,K,V>,IntoIterInner<'static,u32,u32>>(iter),
            _marker:PhantomData,
        }
    }

    #[inline]
    pub fn iter(&self)->&IntoIterInner<'_,K,V>{
        unsafe{ transmute_reference::<IntoIterInner<'static,u32,u32>,_>(&self.iter) }
    }
    #[inline]
    pub fn iter_mut(&mut self)->&mut IntoIterInner<'_,K,V>{
        unsafe{ transmute_mut_reference::<IntoIterInner<'static,u32,u32>,_>(&mut self.iter) }
    }
}


impl<K,V> Iterator for IntoIter<K,V>{
    type Item=Tuple2<K,V>;

    #[inline]
    fn next(&mut self)->Option<Tuple2<K,V>>{
        self.iter_mut().next()
    }

    #[inline]
    fn nth(&mut self,nth:usize)->Option<Tuple2<K,V>>{
        self.iter_mut().nth(nth)
    }

    #[inline]
    fn size_hint(&self)->(usize,Option<usize>){
        self.iter().size_hint()
    }

    #[inline]
    fn count(mut self)->usize{
        self.iter_mut().by_ref().count()
    }

    #[inline]
    fn last(mut self)->Option<Tuple2<K,V>>{
        self.iter_mut().by_ref().last()
    }
}
