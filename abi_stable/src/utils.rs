/*!
Utility functions.
*/

use std::{
    cmp::Ord,
    fmt::{self,Debug,Display},
    mem::{self,ManuallyDrop},
};


use core_extensions::{
    strings::LeftPadder,
    prelude::*,
};

use crate::std_types::{RString,RStr};


//////////////////////////////////////


/// Information about a panic,used in `ffi_panic_message`. 
#[derive(Debug,Copy,Clone)]
pub struct PanicInfo{
    pub file:&'static str,
    pub line:u32,
}


/// Prints an error message for attempting to panic across the 
/// ffi boundary and aborts the process.
#[inline(never)]
#[cold]
pub fn ffi_panic_message(info:&'static PanicInfo) -> ! {
    eprintln!("\nfile:{}\nline:{}", info.file, info.line);
    eprintln!("Attempted to panic across the ffi boundary.");
    eprintln!("Aborting to handle the panic...\n");
    std::process::exit(1);
}


//////////////////////////////////


#[doc(hidden)]
pub struct AbortBomb{
    pub fuse:&'static PanicInfo,
}

impl Drop for AbortBomb{
    fn drop(&mut self){
        ffi_panic_message(self.fuse);
    }
}


//////////////////////////////////

/// Leaks `value` into the heap,and returns a reference to it.
#[inline]
pub fn leak_value<'a,T>(value:T)->&'a T
where T:'a // T:'a is for the docs
{
    let x=Box::new(value);
    Box::leak(x)
}


/// Transmute a reference to another reference,
/// changing the referent's type.
/// 
/// # Safety
///
/// This has the same safety concerns that `std::mem::transmute` has,including that
/// `T` has to have an alignment and be compatible with `U`.
#[inline]
pub unsafe fn transmute_reference<T,U>(ref_:&T)->&U{
    &*(ref_ as *const _ as *const U)
}


/// Transmute a mutable reference to another mutable reference,
/// changing the referent's type.
/// 
/// # Safety
///
/// This has the same safety concerns that `std::mem::transmute` has,including that
/// `T` has to have an alignment and be compatible with `U`.
#[inline]
pub unsafe fn transmute_mut_reference<'a,T,U>(ref_:&'a mut T)->&'a mut U{
    &mut *(ref_ as *mut _ as *mut U)
}

//////////////////////////////////////


#[allow(dead_code)]
pub(crate) fn min_by<T,F,K>(l:T,r:T,mut f:F)->T
where 
    F:FnMut(&T)->K,
    K:Ord,
{
    if f(&l) < f(&r) {
        l
    }else{
        r
    }
}


#[allow(dead_code)]
pub(crate) fn max_by<T,F,K>(l:T,r:T,mut f:F)->T
where 
    F:FnMut(&T)->K,
    K:Ord,
{
    if f(&l) > f(&r) {
        l
    }else{
        r
    }
}

pub(crate) fn min_max_by<T,F,K>(l:T,r:T,mut f:F)->(T,T)
where 
    F:FnMut(&T)->K,
    K:Ord,
{
    if f(&l) < f(&r) {
        (l,r)
    }else{
        (r,l)
    }
}



//////////////////////////////////////




pub(crate) trait FmtPadding{
    fn display_pad<'a,T>(&'a mut self,padding:usize,v:&T)->Result<LeftPadder<'a> ,fmt::Error>
    where T:Display;

    fn debug_pad<'a,T>(&'a mut self,padding:usize,v:&T)->Result<LeftPadder<'a> ,fmt::Error>
    where T:Debug;
}


macro_rules! impl_fmt_padding {
    ($ty:ty) => (
        impl FmtPadding for $ty{
            fn display_pad<'a,T>(
                &'a mut self,
                padding:usize,
                v:&T
            )->Result<LeftPadder<'a> ,fmt::Error>
            where T:Display
            {
                use std::fmt::Write;
                let this=self.into_type_mut();

                this.clear();

                writeln!(this,"{}",v)?;

                Ok(this.left_padder(padding))
            }

            fn debug_pad<'a,T>(
                &'a mut self,
                padding:usize,
                v:&T
            )->Result<LeftPadder<'a> ,fmt::Error>
            where T:Debug
            {
                use std::fmt::Write;
                let this=self.into_type_mut();

                this.clear();

                writeln!(this,"{:#?}",v)?;

                Ok(this.left_padder(padding))
            }           
        }

    )
 }





impl_fmt_padding!{ String }
impl_fmt_padding!{ RString }




//////////////////////////////////////////////////////////////////////

/// Takes the contents out of a `ManuallyDrop<T>`.
///
/// # Safety
///
/// After this function is called `slot` will become uninitialized and 
/// must not be read again.
pub unsafe fn take_manuallydrop<T>(slot: &mut ManuallyDrop<T>) -> T {
    #[cfg(feature = "rust_1_42")]
    {
        ManuallyDrop::take(slot)
    }
    #[cfg(not(feature = "rust_1_42"))]
    {
        ManuallyDrop::into_inner(std::ptr::read(slot))
    }
}


#[doc(hidden)]
#[inline(always)]
pub fn assert_fnonce<F,R>(_:&F)
where
    F:FnOnce()->R
{}


/// This function allows calculating the distance (in `T`s) from `from` to `to`.
/// 
/// This returns `None` if `from` has a higher address than `to`,
/// or if `T` is a zero sized type.
/// 
/// # Example
/// 
/// ```
/// use abi_stable::utils;
/// 
/// let arr=["hello","world","foo","bar","baz"];
/// 
/// assert_eq!(utils::distance_from(&arr[0],&arr[0]),Some(0));
/// assert_eq!(utils::distance_from(&arr[0],&arr[4]),Some(4));
///
/// assert_eq!(utils::distance_from(&arr[4],&arr[0]),None);
/// 
/// ```
pub fn distance_from<T>(from:*const T,to:*const T)->Option<usize>{
    (to as usize)
        .checked_sub(from as usize)?
        .checked_div(mem::size_of::<T>())
}


//////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub extern "C" fn get_type_name<T>()->RStr<'static>{
    RStr::from(std::any::type_name::<T>())
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn distance_from_(){
        let int_array=[0,1,2,3,4];
        let unit_array=[(),(),(),(),()];

        for (ix,x) in int_array.iter().enumerate() {
            for (iy,y) in int_array.iter().enumerate() {
                if ix <= iy {
                    assert_eq!(distance_from(x,y), Some(iy-ix));
                }else{
                    assert_eq!(distance_from(x,y), None);
                }
            }
        }

        for x in &unit_array {
            for y in &unit_array {
                assert_eq!(distance_from(x,y), None);
            }
        }

    }
}