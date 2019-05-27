use std::fmt::Debug;

// #[sabi_trait]
// pub trait RSomething<T>{
//     type Element:Debug;

//     fn get(&self)->&Self::Element;
    
//     fn get_mut(&mut self)->&mut Self::Element;

//     #[sabi(last_prefix_field)]
//     fn into_value(self)->Self::Element;
// }

///////////////////////////////////////////////////////////////////////////
///////                Generated Code                               ///////
///////////////////////////////////////////////////////////////////////////

mod RSomething{
    use super::*;

    use abi_stable::{
        extern_fn_panic_handling,
        pointer_trait::TransmuteElement,
        prefix_type::{PrefixTypeTrait,WithMetadata},
        traits::IntoInner,
        sabi_types::StaticRef,
        reexports::SelfOps,
        utils::{transmute_reference,transmute_mut_reference},
    };

    use core_extensions::utils::transmute_ignore_size;

    use std::{
        marker::PhantomData,
        mem::ManuallyDrop,
        ops::{Deref,DerefMut},
    };

    #[repr(C)]
    pub struct TraitObject<'lt,P,Element,T>{
        vtable:StaticRef<VTable<(),P,Element,T>>,
        ptr:ManuallyDrop<P>,
        _marker:PhantomData<&'lt ()>,
    }

    pub trait Trait<T>{
        type Element:Debug;

        fn get(&self)->&Self::Element;
        fn get_mut(&mut self)->&mut Self::Element;
        fn into_value(self)->Self::Element
        where Self:Sized;
    }

    pub fn new<'lt,P,_Self,T>(ptr_:P)-> TraitObject<'lt,P::TransmutedPtr,_Self::Element,T>
    where 
        P:Deref<Target=_Self>+'lt,
        P:TransmuteElement<()>,
        P::TransmutedPtr:IntoInner<Element=_Self>,
        _Self:Trait<T>+'lt,
    {
        let vtable=MakeVTable::<_Self,P,P::TransmutedPtr,T>::VTABLE;
        unsafe{
            TraitObject{
                vtable:transmute_ignore_size(vtable),
                ptr:ManuallyDrop::new(ptr_.transmute_element(<()>::T)),
                _marker:PhantomData,
            }
        }
    }

    impl<'lt,P,Element,T> TraitObject<'lt,P,Element,T>{
        fn _as_ref(&self)->&PhantomData<()>
        where
            P: Deref<Target=()>
        {
            unsafe{&*((&**self.ptr) as *const () as *const PhantomData<()>)}
        }
        fn _as_mut(&mut self)->&mut PhantomData<()>
        where
            P: DerefMut<Target=()>
        {
            unsafe{&mut *((&mut **self.ptr) as *mut () as *mut PhantomData<()>)}
        }

        pub fn get_(&self)->&Element
        where
            P: Deref<Target=()>
        {
            self.vtable.get().get()(self._as_ref())
        }
        pub fn get_mut_(&mut self)->&mut Element
        where
            P: DerefMut<Target=()>
        {
            self.vtable.get().get_mut()(self._as_mut())
        }
        pub fn into_value_(self)->Element{
            let __method=self.vtable.get().into_value();
            let mut __this=ManuallyDrop::new(self);
            let __this=unsafe{ abi_stable::utils::take_manuallydrop(&mut __this.ptr) };
            __method(__this)
        }
    }

    impl<'lt,P,Element,T> Trait<T> for TraitObject<'lt,P,Element,T>
    where
        P: DerefMut<Target=()>,
        Element:Debug,
    {
        type Element=Element;

        fn get(&self)->&Self::Element{
            self.get_()
        }
        fn get_mut(&mut self)->&mut Self::Element{
            self.get_mut_()
        }
        fn into_value(self)->Self::Element{
            self.into_value_()
        }
    }


    impl<P,Element,T> Drop for TraitObject<'_,P,Element,T>{
        fn drop(&mut self){
            let __method=self.vtable.get().drop_();
            unsafe{
                __method(abi_stable::utils::take_manuallydrop(&mut self.ptr))
            }
        }
    }


    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix(prefix_struct="VTable")))]
    struct VTableVal<_Self,_ErasedPtr,Element,T>{
        _tys:PhantomData<extern "C" fn(_Self,_ErasedPtr,Element,T)>,

        drop_:extern "C" fn(this:_ErasedPtr),

        get:extern "C" fn(this:&PhantomData<_Self>)->&Element,
        
        get_mut:extern "C" fn(this:&mut PhantomData<_Self>)->&mut Element,

        #[sabi(last_prefix_field)]
        into_value:extern "C" fn(this:_ErasedPtr)->Element,
    }


    struct MakeVTable<_Self,_ErasedPtr,_OrigPtr,T>(_Self,_ErasedPtr,_OrigPtr,T);


    impl<'lt,_Self,_ErasedPtr,_OrigPtr,T> MakeVTable<_Self,_ErasedPtr,_OrigPtr,T>
    where 
        _Self:'lt,
        _ErasedPtr:'lt,
        _OrigPtr:'lt,
        T:'lt,
        _OrigPtr:IntoInner<Element=_Self>,
        _Self:Trait<T>,
    {
        const VTABLE: &'lt WithMetadata<VTableVal<_Self,_ErasedPtr,_Self::Element,T>>= 
            &WithMetadata::new(
                PrefixTypeTrait::METADATA,
                VTableVal{
                    _tys:PhantomData,
                    drop_:Self::drop_,
                    get:Self::get,
                    get_mut:Self::get_mut,
                    into_value:Self::into_value,
                }
            );

        extern "C" fn drop_(this:_ErasedPtr){
            extern_fn_panic_handling!{
                unsafe{ 
                    transmute_ignore_size::<_ErasedPtr,_OrigPtr>(this);
                }
            }
        }
    
        extern "C" fn get(this:&PhantomData<_Self>)->&_Self::Element{
            extern_fn_panic_handling!{
                let _self=unsafe{ transmute_reference::<_,_Self>({this}) };
                Trait::get(_self)
            }
        }

        extern "C" fn get_mut(this:&mut PhantomData<_Self>)->&mut _Self::Element{
            extern_fn_panic_handling!{
                let _self=unsafe{ transmute_mut_reference::<_,_Self>(this) };
                Trait::get_mut(_self)
            }
        }

        extern "C" fn into_value(this:_ErasedPtr)->_Self::Element {
            extern_fn_panic_handling!{
                let _self=unsafe{ transmute_ignore_size::<_ErasedPtr,_OrigPtr>(this) };
                Trait::into_value(_self.into_inner_())
            }
        }
    }


    
}

