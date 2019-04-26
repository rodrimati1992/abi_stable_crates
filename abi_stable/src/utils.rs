/*!
Utility functions.
*/

/// Creates an empty slice.
pub const fn empty_slice<'a, T>() -> &'a [T]
where
    T: 'a,
{
    GetEmptySlice::<'a, T>::EMPTY
}

struct GetEmptySlice<'a, T>(&'a T);

impl<'a, T> GetEmptySlice<'a, T>
where
    T: 'a,
{
    const EMPTY: &'a [T] = &[];
}

/// Prints an error message for attempting to panic across the 
/// ffi boundary and aborts the process.
#[inline(never)]
#[cold]
pub fn ffi_panic_message(file: &'static str, line: u32) -> ! {
    eprintln!("\nfile:{}\nline:{}", file, line);
    eprintln!("Attempted to panic across the ffi boundary.");
    eprintln!("Aborting to handle the panic...\n");
    ::std::process::abort();
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
pub unsafe fn transmute_mut_reference<T,U>(ref_:&mut T)->&mut U{
    &mut *(ref_ as *mut _ as *mut U)
}

//////////////////////////////////////


use std::fmt::{self,Debug,Display};

use core_extensions::{
    strings::LeftPadder,
    prelude::*,
};

use crate::std_types::RString;


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