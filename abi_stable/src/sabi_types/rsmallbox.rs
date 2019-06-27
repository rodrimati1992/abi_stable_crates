use crate::{
    pointer_trait::{
        CallReferentDrop,Deallocate,TransmuteElement,
        GetPointerKind,PK_SmartPointer,OwnedPointer,
    },
    sabi_types::MovePtr,
    std_types::RBox,
};

use std::{
    alloc::{self,Layout},
    fmt::{self,Display},
    marker::PhantomData,
    mem::{self,ManuallyDrop},
    ops::{Deref,DerefMut},
    ptr,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use serde::{Serialize,Deserialize,Serializer,Deserializer};


pub use crate::traits::{alignment,InlineStorage};


pub use self::private::RSmallBox;

mod private{
    use super::*;


/**

A box type which stores small values inline as an optimization.

# Inline storage

Inline is the storage space on the stack (as in inline with the RSmallBox struct)
where small values get stored,instead of storing them on the heap.

It has to have an alignment greater than or equal to the value being stored,
otherwise storing the value on the heap.

To ensure that the inline storage has enough alignemnt you can use one of hte 
`AlignTo*` types from the alignment submodule.

*/
    pub struct RSmallBox<T,Inline>{
        inline:ManuallyDrop<Inline>,
        ptr:*mut T,
        destroy: unsafe extern "C" fn(*mut T, CallReferentDrop,Deallocate),
        _marker:PhantomData<(T,Inline)>
    }

    impl<T,Inline> RSmallBox<T,Inline>{
        /// Constructs this RSmallBox from a value.
        #[inline]
        pub fn new(value:T)->RSmallBox<T,Inline>
        where
            Inline:InlineStorage
        {
            Self::new_inner(value)
        }

        #[inline]
        pub(super)fn new_inner(value:T)->RSmallBox<T,Inline>{
            let mut value=ManuallyDrop::new(value);

            unsafe{
                RSmallBox::from_move_ptr(MovePtr::new(&mut *value))
            }
        }


        /// Gets a raw pointer into the underlying data.
        #[inline]
        pub fn as_mut_ptr(this:&mut Self)->*mut T{
            if this.ptr.is_null() {
                (&mut this.inline as *mut ManuallyDrop<Inline> as *mut T)
            }else{
                this.ptr
            }
        }

        /// Gets a raw pointer into the underlying data.
        #[inline]
        pub fn as_ptr(this:&Self)->*const T{
            if this.ptr.is_null() {
                (&this.inline as *const ManuallyDrop<Inline> as *const T)
            }else{
                this.ptr
            }
        }

        /// Constructs this RSmallBox from a MovePtr.
        pub fn from_move_ptr(from_ptr:MovePtr<'_,T>)->Self{
            let destroy=destroy::<T>;
            let inline_size =mem::size_of::<Inline>();
            let value_size =mem::size_of::<T>();

            let inline_align =mem::align_of::<Inline>();
            let value_align =mem::align_of::<T>();

            unsafe{
                let mut inline:ManuallyDrop<Inline>=mem::uninitialized();
                let (storage_ptr,ptr)=if inline_size < value_size || inline_align < value_align {
                    let x=alloc::alloc(Layout::new::<T>());
                    (x,x as *mut T)
                }else{
                    ( (&mut inline as *mut ManuallyDrop<Inline> as *mut u8), ptr::null_mut() )
                };

                (from_ptr.into_raw() as *const T as *const u8)
                    .copy_to_nonoverlapping(storage_ptr,value_size);

                Self{
                    inline,
                    ptr,
                    destroy,
                    _marker:PhantomData,
                }
            }
        }

        /// Converts this RSmallBox into another one with a differnet inline size.
        #[inline]
        pub fn move_<Inline2>(this:Self)->RSmallBox<T,Inline2>
        where
            Inline2:InlineStorage
        {
            Self::with_move_ptr(
                ManuallyDrop::new(this),
                RSmallBox::from_move_ptr
            )
        }

        /// Queries whether the value is stored inline.
        pub fn is_inline(this:&Self)->bool{
            this.ptr.is_null()
        }


        /// Queries whether the value is stored on the heap.
        pub fn is_heap_allocated(this:&Self)->bool{
            !this.ptr.is_null()
        }

        /// Unwraps this pointer into its owned value.
        pub fn into_inner(this:Self)->T{
            Self::with_move_ptr(
                ManuallyDrop::new(this),
                |x|x.into_inner()
            )
        }

        pub(super) unsafe fn drop_in_place(this:&mut Self,drop_referent:CallReferentDrop){
            let (ptr,dealloc)=if this.ptr.is_null() {
                (&mut this.inline as *mut ManuallyDrop<Inline> as *mut T,Deallocate::No)
            }else{
                (this.ptr,Deallocate::Yes)
            };
            (this.destroy)(ptr,drop_referent,dealloc);
        }
    }


    /// Converts an RBox into an RSmallBox,currently this allocates.
    impl<T,Inline> From<RBox<T>> for RSmallBox<T,Inline>
    where
        Inline:InlineStorage
    {
        fn from(this:RBox<T>)->Self{
            RBox::with_move_ptr(
                ManuallyDrop::new(this),
                Self::from_move_ptr
            )
        }
    }


    /// Converts a RSmallBox into an RBox,currently this allocates.
    impl<T,Inline> Into<RBox<T>> for RSmallBox<T,Inline>
    where
        Inline:InlineStorage
    {
        fn into(self)->RBox<T>{
            Self::with_move_ptr(
                ManuallyDrop::new(self),
                |x|x.into_rbox()
            )
        }
    }

}


///////////////////////////////////////////////////////////////////////////////


unsafe impl<T,Inline> GetPointerKind for RSmallBox<T,Inline>{
    type Kind=PK_SmartPointer;
}

impl<T,Inline> Deref for RSmallBox<T,Inline>{
    type Target=T;

    fn deref(&self)->&T{
        unsafe{
            &*Self::as_ptr(self)
        }
    }
}

impl<T,Inline> DerefMut for RSmallBox<T,Inline>{
    fn deref_mut(&mut self)->&mut T{
        unsafe{
            &mut *Self::as_mut_ptr(self)
        }
    }
}


impl<T,Inline> Default for RSmallBox<T,Inline>
where
    T: Default,
    Inline:InlineStorage,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}


impl<T,Inline> Clone for RSmallBox<T,Inline>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        RSmallBox::new_inner((**self).clone())
    }
}


impl<T,Inline> Display for RSmallBox<T,Inline>
where 
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}



shared_impls! {
    mod=box_impls
    new_type=RSmallBox[][T,Inline],
    original_type=Box,
}


unsafe impl<T, O, Inline> TransmuteElement<O> for RSmallBox<T,Inline> {
    type TransmutedPtr = RSmallBox<O,Inline>;
}

unsafe impl<T: Send,Inline> Send for RSmallBox<T,Inline> {}
unsafe impl<T: Sync,Inline> Sync for RSmallBox<T,Inline> {}

///////////////////////////////////////////////////////////////////////////////


impl<T,Inline> Serialize for RSmallBox<T,Inline>
where
    T:Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).serialize(serializer)
    }
}


impl<'de,T,Inline> Deserialize<'de> for RSmallBox<T,Inline> 
where
    Inline:InlineStorage,
    T:Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self::new)
    }
}


//////////////////////////////////////////////////////////////////////////////


unsafe impl<T,Inline> OwnedPointer for RSmallBox<T,Inline>{
    type Target=T;

    #[inline]
    unsafe fn get_move_ptr(this:&mut ManuallyDrop<Self>)->MovePtr<'_,Self::Target>{
        MovePtr::new(&mut **this)
    }

    #[inline]
    unsafe fn drop_allocation(this:&mut ManuallyDrop<Self>){
        Self::drop_in_place(&mut **this,CallReferentDrop::No);
    }
}



impl<T,Inline> Drop for RSmallBox<T,Inline>{
    fn drop(&mut self){
        unsafe{
            Self::drop_in_place(self,CallReferentDrop::Yes);
        }
    }
}



unsafe extern "C" fn destroy<T>(ptr:*mut T,drop_referent:CallReferentDrop,dealloc:Deallocate){
    extern_fn_panic_handling! {no_early_return;
        if let CallReferentDrop::Yes=drop_referent{
            ptr::drop_in_place(ptr);
        }
        if let Deallocate::Yes=dealloc{
            Box::from_raw(ptr as *mut ManuallyDrop<T>);
        }
    }
}

//////////////////////////////////////////////////////////////////////////////




#[cfg(test)]
mod tests;