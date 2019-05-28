use std::fmt::Debug;

// #[sabi_trait]
// pub trait RSomething<T>{
//     type Element:Debug;

//     fn get(&self)->&Self::Element;
    
//     fn get_mut(&mut self)->&mut Self::Element;

//     #[sabi(last_prefix_field)]
//     fn into_value(self)->Self::Element
//     where Self:Sized;
// }

///////////////////////////////////////////////////////////////////////////
///////                Generated Code                               ///////
///////////////////////////////////////////////////////////////////////////

mod RSomething{
    use super::*;

    use abi_stable::sabi_trait::reexports::{*,__sabi_re};

    #[repr(C)]
    pub struct TraitObject<'lt,P,Element,T>{
        vtable:__sabi_re::StaticRef<VTable<(),P,Element,T>>,
        ptr: __sabi_re::ManuallyDrop<P>,
        _marker:__Phantom<&'lt ()>,
    }

    pub trait Trait<T>{
        type Element:Debug;

        fn get(&self)->&Self::Element;
        fn get_mut(&mut self)->&mut Self::Element;
        fn into_value(self)->Self::Element
        where Self:Sized;
    }

    pub fn new<'lt,P,_Self,T>(ptr:P)-> TraitObject<'lt,P::TransmutedPtr,_Self::Element,T>
    where 
        P:__DerefTrait<Target=_Self>+'lt,
        P:__sabi_re::TransmuteElement<()>,
        _Self:Trait<T>+'lt,
    {
        let vtable=MakeVTable::<_Self,P,P::TransmutedPtr,T>::VTABLE;
        let ptr=unsafe{
            __sabi_re::TransmuteElement::<()>::transmute_element(ptr,__Phantom)
        };
        let ptr=__sabi_re::ManuallyDrop::new(ptr);
        unsafe{
            TraitObject{
                vtable:__sabi_re::transmute_ignore_size(vtable),
                ptr,
                _marker:__Phantom,
            }
        }
    }

    impl<'lt,P,Element,T> TraitObject<'lt,P,Element,T>
    where
        Element:Debug,
    {
        fn _as_ref(&self)->&__Phantom<()>
        where
            P: __DerefTrait<Target=()>
        {
            unsafe{&*((&**self.ptr) as *const () as *const __Phantom<()>)}
        }
        fn _as_mut(&mut self)->&mut __Phantom<()>
        where
            P: __DerefMutTrait<Target=()>
        {
            unsafe{&mut *((&mut **self.ptr) as *mut () as *mut __Phantom<()>)}
        }

        #[inline]
        fn _sabi_with_value<F,R>(self,f:F)->R
        where 
            P: __sabi_re::OwnedPointer<Target=()>,
            F:FnOnce(__sabi_re::MovePtr<'_,()>)->R,
        {
            let mut __this= __sabi_re::ManuallyDrop::new(self);
            __sabi_re::OwnedPointer::with_moved_ptr(
                unsafe{ __sabi_re::ptr::read(&mut __this.ptr) },
                f,
            )
        }

        pub fn get_(&self)->&Element
        where
            P: __DerefTrait<Target=()>
        {
            self.vtable.get().get()(self._as_ref())
        }
        pub fn get_mut_(&mut self)->&mut Element
        where
            P: __DerefMutTrait<Target=()>
        {
            self.vtable.get().get_mut()(self._as_mut())
        }
        pub fn into_value_(self)->Element
        where
            P: __sabi_re::OwnedPointer<Target=()>
        {
            let __method=self.vtable.get().into_value();
            self._sabi_with_value(move|__this|__method(__this))
        }
    }

    impl<'lt,P,Element,T> Trait<T> for TraitObject<'lt,P,Element,T>
    where
        P: __DerefMutTrait<Target=()>,
        P: __sabi_re::OwnedPointer<Target=()>,
        Element:Debug,
    {
        type Element=Element;

        fn get(&self)->&Element{
            self.get_()
        }
        fn get_mut(&mut self)->&mut Element{
            self.get_mut_()
        }
        fn into_value(self)->Element{
            self.into_value_()
        }
    }


    impl<P,Element,T> Drop for TraitObject<'_,P,Element,T>{
        fn drop(&mut self){
            let __method=self.vtable.get()._sabi_drop();
            unsafe{
                __method(abi_stable::utils::take_manuallydrop(&mut self.ptr))
            }
        }
    }


    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix(prefix_struct="VTable")))]
    struct VTableVal<_Self,_ErasedPtr,Element,T>{
        _sabi_tys:__Phantom<extern "C" fn(_Self,_ErasedPtr,Element,T)>,

        _sabi_drop:extern "C" fn(this:_ErasedPtr),

        get:extern "C" fn(this:&__Phantom<_Self>)->&Element,
        
        get_mut:extern "C" fn(this:&mut __Phantom<_Self>)->&mut Element,

        #[sabi(last_prefix_field)]
        into_value:extern "C" fn(this:__sabi_re::MovePtr<'_,_Self>)->Element,
    }


    struct MakeVTable<_Self,_ErasedPtr,_OrigPtr,T>(_Self,_ErasedPtr,_OrigPtr,T);


    impl<_Self,_ErasedPtr,_OrigPtr,T> MakeVTable<_Self,_ErasedPtr,_OrigPtr,T>
    where 
        _Self:Trait<T>,
    {
        const VTABLE: *const __sabi_re::WithMetadata<VTableVal<_Self,_ErasedPtr,_Self::Element,T>>= 
            &__sabi_re::WithMetadata::new(
                __sabi_re::PrefixTypeTrait::METADATA,
                VTableVal{
                    _sabi_tys:__Phantom,
                    _sabi_drop:Self::_sabi_drop,
                    get:Self::get,
                    get_mut:Self::get_mut,
                    into_value:Self::into_value,
                }
            );

        #[inline]
        fn __unerase_ptr(this:_ErasedPtr)->_OrigPtr{
            unsafe{ __sabi_re::transmute_ignore_size(this) }
        }
        extern "C" fn _sabi_drop(this:_ErasedPtr){
            ::abi_stable::extern_fn_panic_handling!{
                Self::__unerase_ptr(this);
            }
        }
    
        extern "C" fn get(this:&__Phantom<_Self>)->&_Self::Element{
            ::abi_stable::extern_fn_panic_handling!{
                let _self=unsafe{ __sabi_re::transmute_reference::<_,_Self>(this) };
                Trait::get(_self)
            }
        }

        extern "C" fn get_mut(this:&mut __Phantom<_Self>)->&mut _Self::Element{
            ::abi_stable::extern_fn_panic_handling!{
                let _self=unsafe{ __sabi_re::transmute_mut_reference::<_,_Self>(this) };
                Trait::get_mut(_self)
            }
        }

        extern "C" fn into_value(this:__sabi_re::MovePtr<'_,_Self>)->_Self::Element {
            ::abi_stable::extern_fn_panic_handling!{
                Trait::into_value(this.into_inner())
            }
        }
    }    
}

