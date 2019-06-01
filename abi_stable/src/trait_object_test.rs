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

pub use self::RSomething_items::{
    __TraitObject as RSomething_TO,
    __Methods as RSomething_Methods,
    __TraitMarker as RSomething_Marker,
    __Trait as RSomething,
};

mod RSomething_items{
    use super::*;

    use abi_stable::sabi_trait::reexports::{*,__sabi_re};

    /// A marker type that represents RSomething in generic contexts.
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct __TraitMarker;

    pub type __TraitObject<'lt,P,T,Element>=
        __sabi_re::RObject<'lt,P,__TraitMarker,VTable<(),P,T,Element>>;

    mod __inside_{
        use super::__TraitMarker;
        use abi_stable::{InterfaceType,type_level::bools::*};

        abi_stable::impl_InterfaceType!{
            impl InterfaceType for __TraitMarker{
                type Send=False;
                type Sync=True;
                type Clone=True;
            }
        }
    }

    ///
    /// # Generated trait object
    ///
    /// The generated trait object only implements this trait if it implements
    /// the {insert which Deref trait here} trait.
    ///
    /// To only require Deref in methods taking `&self` you can use the 
    /// `{trait_name}_Methods` trait.
    pub trait __Trait<T>{
        type Element:Debug;

        fn get(&self)->&Self::Element;
        fn get_mut(&mut self)->&mut Self::Element;
        fn into_value(self)->Self::Element
        where Self:Sized;
    }

    /// A duplicate of the trait definition,
    /// to be able to call methods with the most permissive pointer traits.
    pub trait __Methods<__ErasedPtr,T>{
        type Element_:Debug;

        fn get_(&self)->&Self::Element_
        where
            __ErasedPtr: __DerefTrait<Target=()>;
        
        fn get_mut_(&mut self)->&mut Self::Element_
        where
            __ErasedPtr: __DerefMutTrait<Target=()>;

        fn into_value_(self)->Self::Element_
        where
            __ErasedPtr: __sabi_re::OwnedPointer<Target=()>;
    }

    impl<'lt,__ErasedPtr,T,Element>  
        __Methods<__ErasedPtr,T> 
    for __TraitObject<'lt,__ErasedPtr,T,Element>
    where
        Element:Debug,
    {
        type Element_=Element;

        fn get_(&self)->&Self::Element_
        where
            __ErasedPtr: __DerefTrait<Target=()>
        {
            self.sabi_vtable().get()(self.sabi_erased_ref())
        }
        fn get_mut_(&mut self)->&mut Self::Element_
        where
            __ErasedPtr: __DerefMutTrait<Target=()>
        {
            self.sabi_vtable().get_mut()(self.sabi_erased_mut())
        }
        fn into_value_(self)->Self::Element_
        where
            __ErasedPtr: __sabi_re::OwnedPointer<Target=()>
        {
            let __method=self.sabi_vtable().into_value();
            self.sabi_with_value(move|_self|__method(_self))
        }
    }

    impl<'lt,P,T,Element> __Trait<T> for __TraitObject<'lt,P,T,Element>
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


    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix(prefix_struct="VTable")))]
    pub struct VTableVal<_Self,_ErasedPtr,T,Element>{
        _sabi_tys: ::std::marker::PhantomData<extern "C" fn(_Self,_ErasedPtr,T,Element)>,

        _sabi_vtable:__sabi_re::StaticRef<__sabi_re::RObjectVtable<_ErasedPtr,__TraitMarker>>,

        get:extern "C" fn(_self:&__ErasedObject<_Self>)->&Element,
        
        get_mut:extern "C" fn(_self:&mut __ErasedObject<_Self>)->&mut Element,

        #[sabi(last_prefix_field)]
        into_value:extern "C" fn(_self:__sabi_re::MovePtr<'_,_Self>)->Element,
    }

    unsafe impl<IA,_Self,_ErasedPtr,_OrigPtr,T> 
        __sabi_re::GetVTable<IA,_Self,_ErasedPtr,_OrigPtr,T>
    for __TraitMarker
    where
        _Self:__Trait<T>,
        __TraitMarker:__sabi_re::GetRObjectVTable<IA,_Self,_ErasedPtr,_OrigPtr>,
    {
        type VTable=VTable<(),_ErasedPtr,T,_Self::Element>;

        fn get_vtable()->__sabi_re::StaticRef<Self::VTable>{
            unsafe{
                let vtable:*const VTable<_Self,_ErasedPtr,T,_Self::Element>=
                    (*MakeVTable::<IA,_Self,_ErasedPtr,_OrigPtr,T>::VTABLE).as_prefix_raw();
                __sabi_re::StaticRef::from_raw(vtable as *const Self::VTable)
            }
        }
    }


    struct MakeVTable<IA,_Self,_ErasedPtr,_OrigPtr,T>(IA,_Self,_ErasedPtr,_OrigPtr,T);


    impl<IA,_Self,_ErasedPtr,_OrigPtr,T> MakeVTable<IA,_Self,_ErasedPtr,_OrigPtr,T>
    where 
        _Self:__Trait<T>,
        __TraitMarker:__sabi_re::GetRObjectVTable<IA,_Self,_ErasedPtr,_OrigPtr>,
    {
        const VTABLE: *const __sabi_re::WithMetadata<
            VTableVal<_Self,_ErasedPtr,T,_Self::Element>
        >={
            let __vtable=VTableVal{
                _sabi_tys: ::std::marker::PhantomData,
                _sabi_vtable:__sabi_re::GetRObjectVTable::ROBJECT_VTABLE,
                get:Self::get,
                get_mut:Self::get_mut,
                into_value:Self::into_value,
            };
            &__sabi_re::WithMetadata::new(__sabi_re::PrefixTypeTrait::METADATA,__vtable)
        };

        extern "C" fn get(_self:&__ErasedObject<_Self>)->&_Self::Element{
            unsafe{
                __sabi_re::sabi_from_ref(
                    _self,
                    move|_self| __Trait::get(_self)
                )
            }
        }

        extern "C" fn get_mut(_self:&mut __ErasedObject<_Self>)->&mut _Self::Element{
            unsafe{
                __sabi_re::sabi_from_mut(
                    _self,
                    move|_self| __Trait::get_mut(_self)
                )
            }
        }

        extern "C" fn into_value(_self:__sabi_re::MovePtr<'_,_Self>)->_Self::Element {
            ::abi_stable::extern_fn_panic_handling!{no_early_return;
                __Trait::into_value(_self.into_inner())
            }
        }
    }    
}




#[cfg(test)]
mod tests{
    use super::*;

    use crate::std_types::RBox;

    impl RSomething<()> for u32{
        type Element=u32;

        fn get(&self)->&Self::Element{
            self
        }
        fn get_mut(&mut self)->&mut Self::Element{
            self
        }
        fn into_value(self)->Self::Element
        where Self:Sized
        {
            self
        }
    }

    fn assert_sync<T:Sync>(_:&T){}

    #[test]
    fn construct_trait_object(){
        let mut object=RSomething_TO::from_value_unerasable::<_,()>(100_u32);
        let mut erased=RSomething_TO::from_value::<_,()>(100_u32);
        
        assert_sync(&object);
        assert_sync(&erased);

        fn assertions_unerased(mut object:RSomething_TO<'_,RBox<()>,(),u32>){
            assert_eq!(object.sabi_as_unerased::<u32>().ok(),Some(&100));
            assert_eq!(object.sabi_as_unerased::<i8>().ok(),None::<&i8>);
            assert_eq!(object.sabi_as_unerased_mut::<u32>().ok(),Some(&mut 100));
            assert_eq!(object.sabi_as_unerased_mut::<i8>().ok(),None::<&mut i8>);
            object=object.sabi_into_unerased::<i8>().unwrap_err().into_inner();
            assert_eq!(object.sabi_into_unerased::<u32>().ok(),Some(RBox::new(100)));
        }

        fn assertions_erased(mut object:RSomething_TO<'_,RBox<()>,(),u32>){
            assert_eq!(object.sabi_as_unerased::<u32>().ok(),None);
            assert_eq!(object.sabi_as_unerased::<i8>().ok(),None);
            assert_eq!(object.sabi_as_unerased_mut::<u32>().ok(),None);
            assert_eq!(object.sabi_as_unerased_mut::<i8>().ok(),None);
            object=object.sabi_into_unerased::<u32>().unwrap_err().into_inner();
            object=object.sabi_into_unerased::<i8>().unwrap_err().into_inner();
        }

        assertions_unerased(object.clone());
        assertions_unerased(object);
        
        assertions_erased(erased.clone());
        assertions_erased(erased);
        
        
    }

    #[test]
    fn trait_object_methods(){
        let mut object=RSomething_TO::from_value::<_,()>(100);
        let mut cloned=object.clone();
        
        assert_eq!(object.get_(),&100);
        assert_eq!(object.get_mut_(),&mut 100);
        assert_eq!(object.into_value_(),100);

        assert_eq!(cloned.get(),&100);
        assert_eq!(cloned.get_mut(),&mut 100);
        assert_eq!(cloned.into_value(),100);
    }
}