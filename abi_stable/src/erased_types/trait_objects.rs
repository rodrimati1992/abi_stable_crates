use std::ops::{Deref, DerefMut};

use core_extensions::prelude::*;

use super::{c_functions::*, *};

use crate::ErasedObject;

#[macro_export]
macro_rules! declare_trait_object {
    (
        trait_object=$trait_object_vis:vis $trait_object:ident;

        vtable{
            $vis:vis
            struct $struct_name:ident [$($ty_params:ident),* $(,)*]
            where[ $($initializer_where:tt)* ]
            {
                $( $field_vis:vis $field_name:ident : $field_ty:ty =  $field_init:expr ),*
                $(,)?
            }
        }
    ) => (
        #[derive(StableAbi)]
        #[repr(C)]
        #[sabi(inside_abi_stable_crate)]
        #[sabi(kind(unsafe_Prefix))]
        $vis struct $struct_name < $($ty_params= $crate::ErasedObject,)* > {
            $( $field_vis  $field_name:$field_ty, )*
        }

        #[allow(dead_code)]
        impl< $($ty_params,)*> $struct_name< $($ty_params,)*>
        where $($initializer_where)*
        {
            $vis const NEW:Self=Self{
                $( $field_name : $field_init ,)*
            };

            $vis const ERASED:*const $struct_name={
                let x=&Self{
                    $( $field_name : $field_init ,)*
                };
                x as *const Self as *const $struct_name
            };

            $vis fn erased_vtable()->&'static $struct_name{
                unsafe{ &*Self::ERASED }
            }
        }

        #[repr(C)]
        #[derive(StableAbi)]
        #[sabi(inside_abi_stable_crate)]
        pub struct $trait_object<P> {
            this: P,
            vtable: &'static HasherVtable<ErasedObject>,
        }

        impl $trait_object<()> {
            pub fn new<P,T>(this: P) -> $trait_object<P::TransmutedPtr>
            where
                P:DerefMut<Target=T>+
                    $crate::pointer_trait::StableDeref+
                    $crate::pointer_trait::ErasedStableDeref<()>,
                $($initializer_where)*
            {
                $trait_object {
                    this: this.erased(<()>::T),
                    vtable: HasherVtable::<T>::erased_vtable(),
                }
            }
        }

        impl<P> $trait_object<P>{
            pub fn as_ref(&self)->$trait_object<&ErasedObject>
            where P:Deref<Target=ErasedObject>,
            {
                $trait_object{
                    this:&*self.this,
                    vtable:self.vtable,
                }
            }
            pub fn as_mut(&mut self)->$trait_object<&mut ErasedObject>
            where P:DerefMut<Target=ErasedObject>,
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

// declare_trait_vtable!{
//     pub(crate) struct FmtWriteVTable[][T]
//     where [ T:io::Write ]
//     {
//         pub(crate) write_str:extern fn(&mut T, RStr<'_>) -> RResult<(), ()> = () ,
//         pub(crate) write_char:extern fn(&mut T,char) -> RResult<(), ()> = () ,
//     }
// }

// //////////////

// pub(crate) struct IoWriteVTable<T>{
//     pub(crate) write:extern fn(&mut T, RSlice<'_,u8>) -> RResult<usize,RIoError>,
//     pub(crate) flush:extern fn(&mut T) -> RResult<(),RIoError>,

//     pub(crate) write_all:extern fn(&mut T, buf: RSlice<'_,u8>) -> RResult<(),RIoError> ,
// }

// //////////////

// pub(crate) struct IoReadVTable<T>{
//     pub(crate) read:extern fn(&mut T, buf: RSliceMut<'_,u8>) -> RResult<usize,RIoError>,
//     pub(crate) read_exact:extern fn(&mut T, buf: RSliceMut<'_,u8>) -> RResult<(),RIoError>,
// }

// //////////////

// pub(crate) struct IoBufReadVTable<T>{
//     pub(crate) fill_buf:extern fn(&mut T) -> RResult<RSlice<'_,u8>,RIoError>,
//     pub(crate) consume:extern fn(&mut T, amt: usize),
// }

//////////////

declare_trait_object! {
    trait_object=pub HasherTraitObject;

    vtable{
        pub(crate) struct HasherVtable[T]
        where [ T:Hasher, ]
        {
            pub(crate) hash_slice: extern "C" fn(&mut T, RSlice<'_, u8>) = hash_slice_Hasher,
            pub(crate) finish: extern "C" fn(&T) -> u64 = finish_Hasher,
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
