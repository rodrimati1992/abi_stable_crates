#![allow(non_snake_case)]

use std::ptr;

use super::*;

use crate::{
    ErasedObject,
    utils::{transmute_reference,transmute_mut_reference},
};

use core_extensions::utils::transmute_ignore_size;

pub(crate) fn adapt_std_fmt<T>(
    value: &T,
    function: extern "C" fn(&T, FormattingMode, &mut RString) -> RResult<(), ()>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut buf = RString::new();
    let mode = if formatter.alternate() {
        FormattingMode::Alternate
    } else {
        FormattingMode::Default_
    };

    function(value.into(), mode, &mut buf)
        .into_rust()
        .map_err(|_| fmt::Error)?;

    fmt::Display::fmt(&*buf, formatter)
}



pub(crate) unsafe extern "C" fn drop_pointer_impl<OrigP,ErasedPtr>(this: &mut ErasedPtr){
    extern_fn_panic_handling! {unsafe{
        let this=transmute_mut_reference::<ErasedPtr,OrigP>(this);
        ptr::drop_in_place(this);
    }}
}


pub(crate) extern "C" fn clone_pointer_impl<OrigP,ErasedPtr>(this: &ErasedPtr) -> ErasedPtr
where
    OrigP: Clone,
{
    extern_fn_panic_handling! {unsafe{
        let this=transmute_reference::<ErasedPtr,OrigP>(this);
        let clone=this.clone();
        transmute_ignore_size(clone)
    }}
}

pub(crate) extern "C" fn default_pointer_impl<OrigP,ErasedPtr>() -> ErasedPtr
where
    OrigP:Default,
{
    extern_fn_panic_handling! {unsafe{
        transmute_ignore_size( OrigP::default() )
    }}
}

pub(crate) extern "C" fn display_impl<T>(
    this: &ErasedObject,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Display,
{
    extern_fn_panic_handling! {
        use std::fmt::Write;
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };

        let res = match mode {
            FormattingMode::Default_ => write!(buf, "{}", this),
            FormattingMode::Alternate => write!(buf, "{:#}", this),
        };
        match res {
            Ok(_) => ROk(()),
            Err(_) => RErr(()),
        }
    }
}

pub(crate) extern "C" fn debug_impl<T>(
    this: &ErasedObject,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Debug,
{
    extern_fn_panic_handling! {
        use std::fmt::Write;

        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };

        let res = match mode {
            FormattingMode::Default_ => write!(buf, "{:?}", this),
            FormattingMode::Alternate => write!(buf, "{:#?}", this),
        };
        match res {
            Ok(_) => ROk(()),
            Err(_) => RErr(()),
        }
    }
}

pub(crate) extern "C" fn serialize_impl<'a, T>(
    this: &'a ErasedObject
) -> RResult<RCow<'a, RStr<'a>>, RBoxError>
where
    T: ImplType + SerializeImplType,
    T::Interface: InterfaceType<Serialize = True>,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };
        this.serialize_impl().into()
    }
}

// #[inline(never)]
// fn to_string_debug<T>(this: T, mode: FormattingMode) -> RString
// where
//     T: Debug,
// {
//     match mode {
//         FormattingMode::Default_ => format!("{:?}", this),
//         FormattingMode::Alternate => format!("{:#?}", this),
//     }
//     .into_c()
// }

// #[inline(never)]
// fn to_string_display<T>(this: T, mode: FormattingMode) -> RString
// where
//     T: Display,
// {
//     match mode {
//         FormattingMode::Default_ => format!("{:}", this),
//         FormattingMode::Alternate => format!("{:#}", this),
//     }
//     .into_c()
// }

pub(crate) extern "C" fn partial_eq_impl<T>(this: &ErasedObject, other: &ErasedObject) -> bool
where
    T: PartialEq,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };
        let other=unsafe{ transmute_reference::<ErasedObject,T>(other) };
        this == other
    }
}

pub(crate) extern "C" fn cmp_ord<T>(this: &ErasedObject, other: &ErasedObject) -> RCmpOrdering
where
    T: Ord,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };
        let other=unsafe{ transmute_reference::<ErasedObject,T>(other) };
        this.cmp(other).into_c()
    }
}

pub(crate) extern "C" fn partial_cmp_ord<T>(
    this: &ErasedObject, 
    other: &ErasedObject,
) -> ROption<RCmpOrdering>
where
    T: PartialOrd,
{
    extern_fn_panic_handling! {
        let this =unsafe{ transmute_reference::<ErasedObject,T>(this) };
        let other=unsafe{ transmute_reference::<ErasedObject,T>(other) };

        this.partial_cmp(other).map(IntoReprC::into_c).into_c()
    }
}

//////////////////
// Hash

pub(crate) extern "C" fn hash_Hash<T>(
    this: &ErasedObject,
    mut state: trait_objects::HasherTraitObject<&mut ErasedObject>,
) where
    T: Hash,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };

        this.hash(&mut state);
    }
}

//////////////////
// Hasher

pub(crate) extern "C" fn hash_slice_Hasher<T>(this: &mut ErasedObject, slic_: RSlice<'_, u8>)
where
    T: Hasher,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,T>(this) };
        this.write(slic_.into());
    }
}
pub(crate) extern "C" fn finish_Hasher<T>(this: &ErasedObject) -> u64
where
    T: Hasher,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,T>(this) };

        this.finish()
    }
}

// //////////////////////////////////////////////////////////////////////////////////////
// ////                        fmt/io
// //////////////////////////////////////////////////////////////////////////////////////

// pub extern fn write_str_fmt_write<T>(this:&mut T, data:RStr<'_>) -> RResult<(), ()>
// where T:fmt::Write,
// {
//     fmt::Write::write_str(this,data).map_err(drop)
// }

// pub extern fn write_char_fmt_writer<T>(this:&mut T,c:char) -> RResult<(), ()>
// where T:fmt::Write,
// {
//     fmt::Write::write_char(this,c).map_err(drop)
// }

// pub extern fn write_io_write<T>(this:&mut T, data:RSlice<'_,u8>) -> RResult<usize,RIoError>
// where T:io::Write,
// {
//     io::Write::write(this,data).map_err(IntoReprC::into_c).into_c()
// }

// pub extern fn flush_io_write<T>(this:&mut T) -> RResult<(),RIoError>
// where T:io::Write,
// {
//     io::Write::flush(this).map_err(IntoReprC::into_c).into_c()
// }

// pub extern fn write_all_io_write<T>(this:&mut T, buf: RSlice<'_,u8>) -> RResult<(),RIoError>
// where T:io::Write,
// {
//     io::Write::write_all(this,buf).map_err(IntoReprC::into_c).into_c()
// }

// pub extern fn read_io_read<T>(this:&mut T, buf: RSliceMut<'_,u8>) -> RResult<usize,RIoError>
// where T:io::Read,
// {
//     io::Read::read(this,buf).map_err(IntoReprC::into_c).into_c()

// }

// pub extern fn read_exact_io_read<T>(this:&mut T, buf: RSliceMut<'_,u8>) -> RResult<(),RIoError>
// where T:io::Read,
// {
//     io::Read::read_exact(this,buf).map_err(IntoReprC::into_c).into_c()
// }

// pub extern fn fill_buf_io_bufread<T>(this:&mut T) -> RResult<RSlice<'_,u8>,RIoError>
// where T:io::BufRead,
// {
//     match io::BufRead::fill_buf(this) {
//         Ok(x) =>ROk(x.into_c()),
//         Err(x)=>RErr(x.into_c()),
//     }
// }

// pub extern fn consume_io_bufread<T>(this:&mut T, amt: usize)
// where T:io::BufRead,
// {
//     io::BufRead::consume(this,amt)
// }
