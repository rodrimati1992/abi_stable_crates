/*!

Contains `VirtualWrapper<_>`'s vtable,and related types/traits.

*/
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use super::c_functions::*;
use super::*;

use crate::{
    ErasedObject,
};

use core_extensions::{ResultLike, StringExt};

/// Returns the vtable used by VirtualWrapper to do dynamic dispatch.
pub trait GetVtable<This,Ptr>: ImplType {
    const GET_VTABLE: *const VTable<This, Ptr>;

    /// Retrieves the VTable of the type.
    fn get_vtable<'a>() -> &'a VTable<This, Ptr>
    where
        This: 'a,
    {
        // I am just getting a vtable
        unsafe { &*Self::GET_VTABLE }
    }

    /// Gets an erased version of the VTable<This>.
    fn erased_vtable() -> &'static VTable<ErasedObject,ErasedObject> {
        // I am just getting a vtable,which doesn't actually contain an instance of This.
        // This is why it is safe to transmute it to a reference of static lifetime.
        unsafe {
            let x = &*Self::GET_VTABLE;
            mem::transmute::<
                &VTable<This, Ptr>, 
                &'static VTable<ErasedObject,ErasedObject>
            >(x)
        }
    }
}

/// The set of impls this vtable stores.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ImplFlag(ImplFlagRepr);

impl ImplFlag {
    fn iter(&self) -> impl DoubleEndedIterator<Item = WhichImpl> + Clone {
        let unimp = self.0;
        (0..=MAX_BIT_INDEX)
            .filter(move |i| (unimp & (1u64 << i)) != 0)
            .map(WhichImpl)
    }
}

impl Debug for ImplFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

type ImplFlagRepr = u64;

/// Returns the impls that this type represents,each impl is 1 bit in the flags.
pub trait GetImplFlags {
    const FLAGS: ImplFlag;
}

macro_rules! declare_meta_vtable {
    (declare_impl_index;$value:expr;$which_impl0:ident,$which_impl1:ident $(,$rest:ident)* )=>{
        #[allow(non_upper_case_globals)]
        impl trait_selector::$which_impl0{
            pub const WHICH_BIT:u16=$value;
        }
        #[allow(non_upper_case_globals)]
        impl trait_selector::$which_impl1{
            pub const WHICH_BIT:u16=$value + 1;
        }

        declare_meta_vtable!{
            declare_impl_index;
            ($value + 2);
            $($rest),*
        }
    };
    (declare_impl_index;$value:expr;$which_impl:ident)=>{
        #[allow(non_upper_case_globals)]
        impl trait_selector::$which_impl{
            pub const WHICH_BIT:u16=$value;
        }
        pub const MAX_BIT_INDEX:u16=$value;
    };
    (declare_impl_index;$value:expr)=>{
        pub const MAX_BIT_INDEX:u16=$value;
    };
    ////////////////////////////////////////////////////////////////////////
    (
        value=$value:ident;
        pointer=$pointer:ident;

        $([
            $field:ident : $field_ty:ty ;
            impl[$($impl_params:tt)*] VtableFieldValue<$selector:ident>
            where [ $($where_clause:tt)* ]
            { $field_value:expr }
        ])*
    ) => (

        /// This is the vtable for VirtualWrapper<_>,
        ///
        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(inside_abi_stable_crate)]
        //#[sabi(debug_print)]
        pub struct VTable<$value,$pointer>{
            /// Flags for quickly checking whether two VTables have the same impls.
            pub impl_flags:ImplFlag,
            pub type_info:&'static TypeInfo,
            $(
                pub $field:Option<$field_ty>,
            )*
            _marker:PhantomData<extern fn(&$value,&$pointer)>,
        }


        impl<T,P> VTable<T,P>{
            $(
                pub fn $field<E>(&self)->$field_ty
                where
                    E:InterfaceType<$selector=True>,
                {
                    self.assert_is_subset_of::<E>();
                    // Safety:
                    // This is safe to call since we've checked that the
                    // vtable contains an implementation for that trait in the assert method above.
                    unsafe{
                        self.$field.unwrap_unchecked()
                    }
                }
            )*
        }


        /// A non-exhaustive enum that represents one implementations of
        /// the traits mentioned in InterfaceType.
        #[derive(Debug,Copy,Clone,Eq,PartialEq)]
        #[repr(C)]
        pub struct WhichImpl(u16);

        #[allow(non_upper_case_globals)]
        impl WhichImpl{
            $(pub const $selector:Self=WhichImpl(trait_selector::$selector::WHICH_BIT); )*
        }

        impl<A,B> VTable<A,B>{
            #[inline(never)]
            #[cold]
            fn abort_unimplemented(&self,unimplemented_impls:u64){
                eprintln!("error:{}",
                    TooFewImplsError::new(self.type_info,ImplFlag(unimplemented_impls))
                );
                ::std::process::abort();
            }

            pub fn assert_is_subset_of<E>(&self)
            where
                E:GetImplFlags
            {
                let unimplemented_impls=self.get_unimplemented::<E>();
                if unimplemented_impls!=0 {
                    self.abort_unimplemented(unimplemented_impls);
                }
            }

            #[inline]
            /// Gets the impls from E that self does not implement.
            pub fn get_unimplemented<E>(&self)->ImplFlagRepr
            where
                E:GetImplFlags
            {
                let required=E::FLAGS.0;
                let provided=self.impl_flags.0;
                required&(!provided)& LOW_MASK
            }

            /// Checks that `self` implements a subset of the impls that `other` does.
            pub fn check_is_subset_of<E>(&self)->Result<(),TooFewImplsError>
            where
                E:GetImplFlags
            {
                let unimplemented_impls=self.get_unimplemented::<E>();

                if unimplemented_impls==0 {
                    Ok(())
                }else{
                    Err( TooFewImplsError::new(self.type_info,ImplFlag(unimplemented_impls)) )
                }
            }
        }

        /// Returns the type of a vtable field.
        pub type VTableFieldType<Selector,$value,$pointer>=
            <Selector as VTableFieldType_<$value,$pointer>>::Field;

        /// Returns the type of a vtable field.
        pub trait VTableFieldType_<$value,$pointer>{
            type Field;
        }

        /// Returns the value of a vtable field in the current binary
        /// (this can be a different value in a dynamically_linked_library/executable).
        pub trait VTableFieldValue<Ty,IsImpld,$value,$pointer>{
            const FIELD:Option<Ty>;
        }


        $(
            impl<$value,$pointer> VTableFieldType_<$value,$pointer> for trait_selector::$selector {
                type Field=$field_ty;
            }

            impl<$value,$pointer>
                VTableFieldValue<$field_ty,False,$value,$pointer>
            for trait_selector::$selector
            {
                const FIELD:Option<$field_ty>=None;
            }

            impl<$value,$pointer,$($impl_params)*>
                VTableFieldValue<$field_ty,True,$value,$pointer>
            for trait_selector::$selector
            where $($where_clause)*
            {
                const FIELD:Option<$field_ty>=Some($field_value);
            }
        )*

        /// Contains marker types representing traits of the same name.
        pub mod trait_selector{
            $(
                /// Marker type representing the trait of the same name.
                pub struct $selector;
            )*
        }


        declare_meta_vtable!{
            declare_impl_index;
            1;
            $($selector),*
        }

        /// The largest value for the flags representing which impls are required/provided.
        const MAX_BIT_VALUE:ImplFlagRepr= 1 << MAX_BIT_INDEX ;

        // For keeping the bits that are relevant for checking that
        // all the traits required in the InterfaceType are implemented.
        const LOW_MASK:ImplFlagRepr=(MAX_BIT_VALUE-1)|MAX_BIT_VALUE;

        $(
            impl GetImplFlags for trait_selector::$selector{
                const FLAGS:ImplFlag=ImplFlag(1 << Self::WHICH_BIT);
            }
        )*

        impl<E> GetImplFlags for E
        where
            E:InterfaceType,
            $(
                E::$selector:Boolean,
            )*
        {
            const FLAGS:ImplFlag=ImplFlag(
                0 $(
                    | [0,trait_selector::$selector::FLAGS.0]
                      [ <E::$selector as Boolean>::VALUE as usize ]
                )*
            );
        }


        impl<This,$value,$pointer,E> GetVtable<$value,$pointer> for This
        where
            This:ImplType<Interface=E>,
            E:InterfaceType+GetImplFlags,
            $(
                trait_selector::$selector:VTableFieldValue<
                    $field_ty,
                    E::$selector,
                    $value,
                    $pointer
                >,
            )*
        {
            const GET_VTABLE:*const VTable<$value,$pointer>={
                &VTable{
                    impl_flags:E::FLAGS,
                    type_info:This::INFO,
                    $(
                        $field:
                            <trait_selector::$selector as
                                VTableFieldValue<
                                    VTableFieldType<trait_selector::$selector,$value,$pointer>,
                                    E::$selector,
                                    $value,
                                    $pointer
                                >
                            >::FIELD,
                    )*
                    _marker:PhantomData,
                }
            };
        }

        impl<$value,$pointer> Debug for VTable<$value,$pointer> {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable")
                    .field("type_info",&self.type_info)
                    $(
                        .field(
                            stringify!($field),
                            &format_args!("{:x}",self.$field.map_or(0,|x|x as usize))
                        )
                    )*
                    .finish()
            }
        }

    )
}

declare_meta_vtable! {
    value  =T;
    pointer=P;

    [
        clone_ptr:    extern fn(&P)->P;
        impl[] VtableFieldValue<Clone>
        where [P:Clone]
        {
            clone_impl
        }
    ]
    [
        default_ptr:    extern fn()->P;
        impl[] VtableFieldValue<Default>
        where [P:Default]
        {
            default_impl
        }
    ]
    [
        display:    extern fn(&T,FormattingMode,&mut RString)->RResult<(),()>;
        impl[] VtableFieldValue<Display>
        where [T:Display]
        {
            display_impl
        }
    ]
    [
        debug:      extern fn(&T,FormattingMode,&mut RString)->RResult<(),()>;
        impl[] VtableFieldValue<Debug>
        where [T:Debug]
        {
            debug_impl
        }
    ]
    [
        serialize:  extern fn(&T)->RResult<RCow<'_,str>,RBoxError>;
        impl[] VtableFieldValue<Serialize>
        where [
            T:ImplType+SerializeImplType,
            T::Interface:InterfaceType<Serialize=True>,
        ]{
            serialize_impl
        }
    ]
    [
        partial_eq: extern fn(&T,&T)->bool;
        impl[] VtableFieldValue<PartialEq>
        where [T:PartialEq,]
        {
            partial_eq_impl
        }
    ]
    [
        cmp:        extern fn(&T,&T)->RCmpOrdering;
        impl[] VtableFieldValue<Ord>
        where [T:Ord,]
        {
            cmp_ord
        }
    ]
    [
        partial_cmp:extern fn(&T,&T)->ROption<RCmpOrdering>;
        impl[] VtableFieldValue<PartialOrd>
        where [T:PartialOrd,]
        {
            partial_cmp_ord
        }
    ]
    [
        hash:extern "C" fn(&T,trait_objects::HasherTraitObject<&mut ErasedObject>);
        impl[] VtableFieldValue<Hash>
        where [T:Hash]
        {
            hash_Hash
        }
    ]
}

//////////////

/// Error for the case in which an ImplType does not
/// implement the traits declared in its InterfaceType.
#[derive(Debug)]
#[repr(C)]
pub struct TooFewImplsError {
    type_info: &'static TypeInfo,
    /// The required traits that the vtable does not implement
    unimplemented_impls: ImplFlag,
}

impl TooFewImplsError {
    fn new(type_info: &'static TypeInfo, unimplemented_impls: ImplFlag) -> Self {
        Self {
            type_info,
            unimplemented_impls,
        }
    }

    pub fn unimplemented_impls(&self) -> impl DoubleEndedIterator<Item = WhichImpl> + Clone {
        self.unimplemented_impls.iter()
    }
}

impl fmt::Display for TooFewImplsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.type_info;
        writeln!(f, "these traits are not implemented (and they should be):")?;
        for elem in self.unimplemented_impls.iter() {
            writeln!(f, "    {:?}", elem)?;
        }
        writeln!(f, "Type information:\n{}", info.to_string().left_pad(4))?;
        Ok(())
    }
}

//////////////
