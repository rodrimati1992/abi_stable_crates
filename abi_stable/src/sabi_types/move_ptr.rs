use std::{
    ops::{Deref,DerefMut},
    fmt::{self,Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr,
};

use crate::traits::IntoInner;

/**
A move pointer,which allows moving the value from the reference,
consuming it in the process.

if MovePtr::into_inner isn't called,
this drops the referenced value when its dropped

# Safety

This is unsafe to construct since the user must ensure that the value 
being referenced is not read again,even when being dropped.

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
    #[inline]
    pub unsafe fn new(ptr:&'a mut T)->Self{
        Self{
            ptr,
            _marker:PhantomData,
        }
    }

    /// Moves the value out of the reference
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