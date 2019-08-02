/*!
Contains the `MovePtr<_>` type.
*/

use std::{
    alloc::{self,Layout},
    ops::{Deref,DerefMut},
    fmt::{self,Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr,
    
};

use crate::{
    traits::IntoInner,
    std_types::RBox,
};

/**
A move pointer,which allows moving the value from the reference,
consuming it in the process.

if MovePtr::into_inner isn't called,
this drops the referenced value when its dropped

# Safety

This is unsafe to construct since the user must ensure that the value 
being referenced is not read again,even when being dropped.

# Motivation

MovePtr was created as a way to pass self by value to ffi-safe trait object methods,
since one can't simply pass self by value(because the type is erased).

# Examples

### Using OwnedPointer::in_move_ptr

This is how one can use MovePtr without `unsafe`.

This simply moves the contents of an `RBox<T>` into a `Box<T>`.

```
use abi_stable::{
    pointer_trait::OwnedPointer,
    sabi_types::MovePtr,
    std_types::RBox,
};


fn move_rbox_to_box<T>(rbox:RBox<T>)->Box<T>{
    rbox.in_move_ptr(|move_ptr|{
        move_ptr.into_box()
    })
}

assert_eq!( move_rbox_to_box(RBox::new(99)), Box::new(99) );

assert_eq!( move_rbox_to_box(RBox::new(())), Box::new(()) );

assert_eq!(
    move_rbox_to_box(RBox::new(String::from("SHIT"))), 
    Box::new(String::from("SHIT")) 
);


```

### Using the (unsafe) `MovePtr::new`

This is (sort of) how `RBox<T>` implements moving the T it owns out of its allocation

This is basically what `OwnedPointer::{with_move_ptr,in_move_ptr}` do.

```
use abi_stable::{
    pointer_trait::OwnedPointer,
    sabi_types::MovePtr,
    std_types::RBox,
};

use std::mem::ManuallyDrop;

let rbox=RBox::new(0x100);

let second_rbox;
unsafe{ 
    let mut rbox=ManuallyDrop::new(rbox);
    let move_ptr=unsafe{ MovePtr::new( &mut **rbox ) };
    second_rbox=RBox::from_move_ptr(move_ptr);
    OwnedPointer::drop_allocation(&mut rbox); 
}

assert_eq!( second_rbox, RBox::new(0x100) );



```

*/
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound="T:'a")]
pub struct MovePtr<'a,T>{
    ptr:*mut T,
    _marker:PhantomData<&'a mut T>,
}


impl<'a,T> MovePtr<'a,T>{
    /// Constructs this more pointer from a mutable reference,
    /// moving the value out of the reference.
    ///
    /// # Safety 
    ///
    /// Callers must ensure that the value the reference points at is never read again.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::MovePtr;
    /// 
    /// use std::mem::ManuallyDrop;
    /// 
    /// let mut manual=ManuallyDrop::new(String::from("hello"));
    /// 
    /// let moveptr=unsafe{ MovePtr::new(&mut *manual) };
    ///
    /// drop(moveptr); // moveptr drops the String here.
    /// ```
    #[inline]
    pub unsafe fn new(ptr:&'a mut T)->Self{
        Self{
            ptr,
            _marker:PhantomData,
        }
    }

    /// Gets a raw pointer to the value being moved.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("NOPE"));
    /// let address_rbox=&*rbox as *const String as usize;
    ///
    /// rbox.in_move_ptr(|move_ptr|{
    ///     assert_eq!( address_rbox, move_ptr.as_ptr() as usize );
    /// });
    /// 
    /// ```
    #[inline]
    pub const fn as_ptr(&self)->*const T{
        self.ptr
    }

    /// Gets a raw pointer to the value being moved.
    ///
    /// # Example
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("NOPE"));
    /// let address_rbox=&*rbox as *const String as usize;
    ///
    /// rbox.in_move_ptr(|move_ptr|{
    ///     assert_eq!( address_rbox, move_ptr.as_mut_ptr() as usize );
    /// });
    /// 
    /// ```
    #[inline]
    pub const fn as_mut_ptr(&self)->*const T{
        self.ptr
    }

    /// Converts this MovePtr into a raw pointer,
    /// which must be moved from before the pointed to value is deallocated.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("NOPE"));
    ///
    /// let string=rbox.in_move_ptr(|move_ptr|unsafe{
    ///     move_ptr.into_raw().read()
    /// });
    /// 
    /// assert_eq!(string,String::from("NOPE"));
    /// 
    /// ```
    pub const fn into_raw(self)->*mut T{
        let ptr=self.ptr;
        ManuallyDrop::new(self);
        ptr
    }

    /// Moves the value into a new `Box<T>`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("WHAT!!!"));
    ///
    /// let boxed=rbox.in_move_ptr(|move_ptr|unsafe{
    ///     move_ptr.into_box()
    /// });
    /// 
    /// assert_eq!(boxed,Box::new(String::from("WHAT!!!")));
    /// 
    /// ```
    pub fn into_box(self)->Box<T>{
        unsafe{
            let allocated=alloc::alloc(Layout::new::<T>()) as *mut T;

            self.into_raw().copy_to_nonoverlapping(allocated,1);

            Box::from_raw(allocated)
        }
    }

    /// Moves the value into a new `RBox<T>`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("WHAT!!!"));
    ///
    /// let boxed=rbox.in_move_ptr(|move_ptr|unsafe{
    ///     move_ptr.into_rbox()
    /// });
    /// 
    /// assert_eq!( boxed, RBox::new(String::from("WHAT!!!")) );
    /// 
    /// ```
    pub fn into_rbox(self)->RBox<T>{
        self.into_box().into()
    }

    /// Moves the value out of the reference
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     std_types::RBox,
    /// };
    /// 
    /// let rbox=RBox::new(String::from("(The Wi)zard(of)oz"));
    ///
    /// let string=rbox.in_move_ptr(|ptr| ptr.into_inner() );
    /// 
    /// assert_eq!( string, String::from("(The Wi)zard(of)oz") );
    /// 
    /// ```
    #[inline]
    pub fn into_inner(self)->T{
        let this=ManuallyDrop::new(self);
        unsafe{ 
            this.ptr.read()
        }
    }
}


shared_impls! {
    mod=move_ptr_impls
    new_type=MovePtr['a][T],
    original_type=AAAA,
}


impl<'a,T> Display for MovePtr<'a,T>
where
    T:Display,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}

impl<'a,T> Deref for MovePtr<'a,T>{
    type Target=T;

    fn deref(&self)->&T{
        unsafe{ &*self.ptr }
    }
}

impl<'a,T> DerefMut for MovePtr<'a,T>{
    fn deref_mut(&mut self)->&mut T{
        unsafe{ &mut *self.ptr }
    }
}

impl<'a,T> IntoInner for MovePtr<'a,T>{
    type Element=T;
    
    fn into_inner_(self)->T{
        self.into_inner()
    }
}

impl<'a,T> Drop for MovePtr<'a,T>{
    fn drop(&mut self){
        unsafe{
            ptr::drop_in_place(self.ptr);
        }
    }
}



//#[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod test{
    use super::*;

    use std::sync::Arc;

    #[test]
    fn with_manuallydrop(){
        let arc=Arc::new(10);
        unsafe{
            let mut cloned_arc=ManuallyDrop::new(arc.clone());
            
            let move_ptr=MovePtr::new(&mut *cloned_arc);
            assert_eq!(Arc::strong_count(&*move_ptr),2);
            
            let moved_arc=move_ptr.into_inner();
            assert_eq!(Arc::strong_count(&moved_arc),2);
        }
        assert_eq!(Arc::strong_count(&arc),1);
        unsafe{
            let mut cloned_arc=ManuallyDrop::new(arc.clone());
            
            let move_ptr=MovePtr::new(&mut *cloned_arc);
            assert_eq!(Arc::strong_count(&*move_ptr),2);
        }
        assert_eq!(Arc::strong_count(&arc),1);
    }
}