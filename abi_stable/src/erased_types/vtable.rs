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
    prefix_type::{PrefixTypeTrait,WithMetadata},
};

use core_extensions::{ResultLike, StringExt};

#[doc(hidden)]
/// Returns the vtable used by VirtualWrapper to do dynamic dispatch.
pub trait GetVtable<This,ErasedPtr,OrigPtr>: ImplType {
    
    const TMP_VTABLE:VTableVal<ErasedPtr>;

    const GET_VTABLE:*const WithMetadata<VTableVal<ErasedPtr>>=
        &WithMetadata::new(
            PrefixTypeTrait::METADATA,
            Self::TMP_VTABLE
        );


    /// Retrieves the VTable of the type.
    fn get_vtable<'a>() -> &'a VTable<ErasedPtr>
    where
        This: 'a,
    {
        // I am just getting a vtable
        unsafe { (*Self::GET_VTABLE).as_prefix() }
    }
}

/// The set of impls this vtable stores.
#[repr(C)]
#[derive(Copy,Clone)]
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
        erased_pointer=$erased_ptr:ident;
        original_pointer=$orig_ptr:ident;

        $([
            $( #[$field_attr:meta] )*
            $field:ident : $field_ty:ty ;
            priv $priv_field:ident;

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
        #[sabi(kind(Prefix(prefix_struct="VTable")))]
        #[sabi(missing_field(default))]
        pub struct VTableVal<$erased_ptr>{
            /// Flags for quickly checking whether two VTables have the same impls.
            pub impl_flags:ImplFlag,
            pub type_info:&'static TypeInfo,
            _marker:PhantomData<extern fn()->$erased_ptr>,
            pub drop_ptr:unsafe extern "C" fn(&mut $erased_ptr),
            $(
                $( #[$field_attr] )*
                $priv_field:Option<($field_ty)>,
            )*
        }


        impl<$erased_ptr> VTable<$erased_ptr>{
            $(
                pub fn $field<E>(&self)->($field_ty)
                where
                    E:InterfaceType<$selector=True>,
                {
                    self.assert_is_subset_of::<E>();
                    // Safety:
                    // This is safe to call since we've checked that the
                    // vtable contains an implementation for that trait in the assert method above.
                    unsafe{
                        self.$priv_field().unwrap_unchecked()
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

        impl<$erased_ptr> VTable<$erased_ptr>{
            #[inline(never)]
            #[cold]
            fn abort_unimplemented(&self,unimplemented_impls:u64){
                eprintln!("error:{}",
                    TooFewImplsError::new(self.type_info(),ImplFlag(unimplemented_impls))
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
                let provided=self.impl_flags().0;
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
                    Err( TooFewImplsError::new(self.type_info(),ImplFlag(unimplemented_impls)) )
                }
            }
        }

        /// Returns the type of a vtable field.
        pub type VTableFieldType<Selector,$value,$erased_ptr,$orig_ptr>=
            <Selector as VTableFieldType_<$value,$erased_ptr,$orig_ptr>>::Field;

        /// Returns the type of a vtable field.
        pub trait VTableFieldType_<$value,$erased_ptr,$orig_ptr>{
            type Field;
        }

        /// Returns the value of a vtable field in the current binary
        /// (this can be a different value in a dynamically_linked_library/executable).
        pub trait VTableFieldValue<Ty,IsImpld,$value,$erased_ptr,$orig_ptr>{
            const FIELD:Option<Ty>;
        }


        $(
            impl<$value,$erased_ptr,$orig_ptr> 
                VTableFieldType_<$value,$erased_ptr,$orig_ptr> 
            for trait_selector::$selector 
            {
                type Field=($field_ty);
            }

            impl<$value,$erased_ptr,$orig_ptr>
                VTableFieldValue<($field_ty),False,$value,$erased_ptr,$orig_ptr>
            for trait_selector::$selector
            {
                const FIELD:Option<($field_ty)>=None;
            }

            impl<$value,$erased_ptr,$orig_ptr,$($impl_params)*>
                VTableFieldValue<($field_ty),True,$value,$erased_ptr,$orig_ptr>
            for trait_selector::$selector
            where $($where_clause)*
            {
                const FIELD:Option<($field_ty)>=Some($field_value);
            }
        )*

        ///////////////////////////////////////////////////////////
        //      Uncomment in 0.3
        ///////////////////////////////////////////////////////////
        // pub trait SendIf<Cond>{}

        // impl<This> SendIf<False> for This
        // {}

        // impl<This> SendIf<True> for This
        // where This:Send
        // {}

        
        // pub trait SyncIf<Cond>{}

        // impl<This> SyncIf<False> for This
        // {}

        // impl<This> SyncIf<True> for This
        // where This:Sync
        // {}
        ///////////////////////////////////////////////////////////




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


        impl<This,$value,$erased_ptr,$orig_ptr,E> 
            GetVtable<$value,$erased_ptr,$orig_ptr> 
        for This
        where
            This:ImplType<Interface=E>,
            E:InterfaceType+GetImplFlags,
            $(
                trait_selector::$selector:VTableFieldValue<
                    ($field_ty),
                    E::$selector,
                    $value,
                    $erased_ptr,
                    $orig_ptr,
                >,
            )*
            // Uncomment in 0.3
            // $orig_ptr:SendIf<E::Send>,
            // $orig_ptr:SyncIf<E::Sync>,
            
            // $erased_ptr:SendIf<E::Send>,
            // $erased_ptr:SyncIf<E::Sync>,
        {
            const TMP_VTABLE:VTableVal<$erased_ptr>=VTableVal{
                impl_flags:E::FLAGS,
                type_info:This::INFO,
                drop_ptr:drop_pointer_impl::<$orig_ptr,$erased_ptr>,
                $(
                    $priv_field:
                        <trait_selector::$selector as
                            VTableFieldValue<
                                VTableFieldType<
                                    trait_selector::$selector,
                                    $value,
                                    $erased_ptr,
                                    $orig_ptr,
                                >,
                                E::$selector,
                                $value,
                                $erased_ptr,
                                $orig_ptr,
                            >
                        >::FIELD,
                )*
                _marker:PhantomData,
            };
        }

        impl<$erased_ptr> Debug for VTable<$erased_ptr> {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                f.debug_struct("VTable")
                    .field("type_info",&self.type_info())
                    $(
                        .field(
                            stringify!($field),
                            &format_args!("{:x}",self.$priv_field().map_or(0,|x|x as usize))
                        )
                    )*
                    .finish()
            }
        }

    )
}

declare_meta_vtable! {
    value  =T;
    erased_pointer=ErasedPtr;
    original_pointer=OrigP;

    [
        clone_ptr:    extern "C" fn(&ErasedPtr)->ErasedPtr;
        priv _clone_ptr;
        
        impl[] VtableFieldValue<Clone>
        where [OrigP:Clone]
        {
            clone_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        default_ptr: extern "C" fn()->ErasedPtr ;
        priv _default_ptr;
        impl[] VtableFieldValue<Default>
        where [OrigP:Default]
        {
            default_pointer_impl::<OrigP,ErasedPtr>
        }
    ]
    [
        display:    extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        priv _display;
        impl[] VtableFieldValue<Display>
        where [T:Display]
        {
            display_impl::<T>
        }
    ]
    [
        debug:      extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>;
        priv _debug;
        impl[] VtableFieldValue<Debug>
        where [T:Debug]
        {
            debug_impl::<T>
        }
    ]
    [
        serialize:  extern "C" fn(&ErasedObject)->RResult<RCow<'_,RStr<'_>>,RBoxError>;
        priv _serialize;
        impl[] VtableFieldValue<Serialize>
        where [
            T:ImplType+SerializeImplType,
            T::Interface:InterfaceType<Serialize=True>,
        ]{
            serialize_impl::<T>
        }
    ]
    [
        partial_eq: extern "C" fn(&ErasedObject,&ErasedObject)->bool;
        priv _partial_eq;
        impl[] VtableFieldValue<PartialEq>
        where [T:PartialEq,]
        {
            partial_eq_impl::<T>
        }
    ]
    [
        cmp:        extern "C" fn(&ErasedObject,&ErasedObject)->RCmpOrdering;
        priv _cmp;
        impl[] VtableFieldValue<Ord>
        where [T:Ord,]
        {
            cmp_ord::<T>
        }
    ]
    [
        partial_cmp:extern "C" fn(&ErasedObject,&ErasedObject)->ROption<RCmpOrdering>;
        priv _partial_cmp;
        impl[] VtableFieldValue<PartialOrd>
        where [T:PartialOrd,]
        {
            partial_cmp_ord::<T>
        }
    ]
    [
        #[sabi(last_prefix_field)]
        hash:extern "C" fn(&ErasedObject,trait_objects::HasherTraitObject<&mut ErasedObject>);
        priv _hash;
        impl[] VtableFieldValue<Hash>
        where [T:Hash]
        {
            hash_Hash::<T>
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
