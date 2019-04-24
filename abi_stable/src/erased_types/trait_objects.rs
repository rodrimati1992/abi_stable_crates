/*!
Ffi-safe trait objects for individual traits.
*/

use std::ops::{Deref, DerefMut};

use core_extensions::prelude::*;

use super::{c_functions::*, *};

use crate::ErasedObject;

macro_rules! declare_trait_object {
    (
        trait_object=$trait_object_vis:vis $trait_object:ident;

        value=$value:ident;
        erased_pointer=$erased_ptr:ident;
        original_pointer=$orig_ptr:ident;

        vtable{
            $vis:vis
            struct $struct_name:ident
            where[ $($initializer_where:tt)* ]
            {
                $( $field_vis:vis $field_name:ident : $field_ty:ty =  $field_init:expr ),*
                $(,)?
            }
        }
    ) => (
        #[derive(StableAbi)]
        #[repr(C)]
        #[derive(Copy,Clone)]
        #[sabi(inside_abi_stable_crate)]
        $vis struct $struct_name {
            $( $field_vis  $field_name:$field_ty, )*
        }

        #[allow(dead_code)]
        impl $struct_name{
            pub fn new<$value,$orig_ptr>()->$struct_name
            where $($initializer_where)*
            {
                Self{
                    $( $field_name : $field_init ,)*
                }
            }
        }

        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(inside_abi_stable_crate)]
        pub struct $trait_object<$erased_ptr> {
            this: $erased_ptr,
            vtable: HasherVtable,
        }

        impl $trait_object<()> {
            pub fn new<$value,$erased_ptr,$orig_ptr>(
                this: $orig_ptr
            ) -> $trait_object<$erased_ptr>
            where
                $orig_ptr:DerefMut<Target=$value>+
                    $crate::pointer_trait::StableDeref+
                    $crate::pointer_trait::ErasedStableDeref<
                        (),
                        TransmutedPtr=$erased_ptr
                    >,
                $erased_ptr:$crate::pointer_trait::StableDeref<
                    Target=$crate::marker_type::ZeroSized<()>
                >,
                $($initializer_where)*
            {
                $trait_object {
                    this: this.erased(<()>::T),
                    vtable: HasherVtable::new::<$value,$orig_ptr>(),
                }
            }
        }

        impl<$erased_ptr> $trait_object<$erased_ptr>{
            pub fn as_ref(&self)->$trait_object<&ErasedObject>
            where $erased_ptr:Deref<Target=ErasedObject>,
            {
                $trait_object{
                    this:&*self.this,
                    vtable:self.vtable,
                }
            }
            pub fn as_mut(&mut self)->$trait_object<&mut ErasedObject>
            where $erased_ptr:DerefMut<Target=ErasedObject>,
            {
                $trait_object{
                    this:&mut *self.this,
                    vtable:self.vtable,
                }
            }

        }

    )
}


//////////////

declare_trait_object! {
    trait_object=pub HasherTraitObject;

    value  =T;
    erased_pointer=ErasedPtr;
    original_pointer=OrigPtr;

    vtable{
        pub(crate) struct HasherVtable
        where [ T:Hasher, ]
        {
            pub(crate) hash_slice: extern "C" fn(&mut ErasedObject, RSlice<'_, u8>) = 
                hash_slice_Hasher::<T>,

            pub(crate) finish: extern "C" fn(&ErasedObject) -> u64 = 
                finish_Hasher::<T>,
        }
    }

}

impl<P> Hasher for HasherTraitObject<P>
where
    P: DerefMut<Target = ErasedObject>,
{
    fn finish(&self) -> u64 {
        (self.vtable.finish)((&*self.this).into())
    }
    fn write(&mut self, bytes: &[u8]) {
        (self.vtable.hash_slice)((&mut *self.this).into(), bytes.into())
    }
}

//////////////
