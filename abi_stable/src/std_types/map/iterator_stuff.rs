use super::*;

use crate::{
    erased_types::{IteratorItem,InterfaceType},
    type_level::bools::*,
};

macro_rules! declare_iter_interface {
    (
        $k:ident=>$v:ident;
        $(#[$attr:meta])*
        interface=$interface:ident;
        type Item=$item:ty;
    ) => (
        $(#[$attr])*
        #[repr(C)]
        #[derive(StableAbi)]
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
    /// The InterfaceType of the `Iter` RHashMap iterator.
    #[sabi(impl_InterfaceType(Iterator,Clone))]
    interface=RefIterInterface;
    type Item=Tuple2<&'a K,&'a V>;
}


declare_iter_interface!{
    K=>V;
    /// The InterfaceType of the `IterMut` RHashMap iterator.
    #[sabi(impl_InterfaceType(Iterator))]
    interface=MutIterInterface;
    type Item=Tuple2<&'a K,&'a mut V>;
}


declare_iter_interface!{
    K=>V;
    /// The InterfaceType of the `Drain` RHashMap iterator.
    #[sabi(impl_InterfaceType(Iterator))]
    interface=ValIterInterface;
    type Item=Tuple2<K,V>;
    
}



///////////////////////////////////////////////////////////////////////////////

type IntoIterInner<'a,K,V>=
    DynTrait<'a,RBox<()>,ValIterInterface<K,V>>;



/// An iterator that yields all the entries of an RHashMap,
/// deallocating the hashmap afterwards.
///
/// This is an `Iterator<Item= Tuple2< K, V > >+!Send+!Sync`
#[repr(transparent)]
#[derive(StableAbi)]
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
    fn iter(&self)->&IntoIterInner<'_,K,V>{
        unsafe{ transmute_reference::<IntoIterInner<'static,u32,u32>,_>(&self.iter) }
    }
    #[inline]
    fn iter_mut(&mut self)->&mut IntoIterInner<'_,K,V>{
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
