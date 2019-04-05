#![allow(non_snake_case)]



use super::*;

use crate::opaque_type::ErasedObject;

pub(crate) fn adapt_std_fmt<T>(
    value: &T,
    function: extern "C" fn(CAbi<&T>, FormattingMode, &mut RString) -> RResult<(), ()>,
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

pub(crate) extern "C" fn clone_impl<T>(this: CAbi<&T>) -> T
where
    T: Clone,
{
    extern_fn_panic_handling!{
        T::clone(&this)
    }
}

pub(crate) extern "C" fn default_impl<T>() -> T
where
    T: Default,
{
    extern_fn_panic_handling!{
        T::default()
    }
}

pub(crate) extern "C" fn display_impl<T>(
    this: CAbi<&T>,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Display,
{
    extern_fn_panic_handling!{
        use std::fmt::Write;

        let res = match mode {
            FormattingMode::Default_ => write!(buf, "{}", *this),
            FormattingMode::Alternate => write!(buf, "{:#}", *this),
        };
        match res {
            Ok(_) => ROk(()),
            Err(_) => RErr(()),
        }
    }
}

pub(crate) extern "C" fn debug_impl<T>(
    this: CAbi<&T>,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Debug,
{
    extern_fn_panic_handling!{
        use std::fmt::Write;

        let res = match mode {
            FormattingMode::Default_ => write!(buf, "{:?}", *this),
            FormattingMode::Alternate => write!(buf, "{:#?}", *this),
        };
        match res {
            Ok(_) => ROk(()),
            Err(_) => RErr(()),
        }
    }
}

pub(crate) extern "C" fn serialize_impl<'a, T>(
    this: CAbi<&'a T>,
) -> RResult<RCow<'a, str>, RBoxError>
where
    T: ImplType + SerializeImplType,
    T::Interface: InterfaceType<Serialize = True>,
{
    extern_fn_panic_handling!{
        this.into_inner().serialize_impl().into()
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

pub(crate) extern "C" fn partial_eq_impl<T>(this: CAbi<&T>, other: CAbi<&T>) -> bool
where
    T: PartialEq,
{
    extern_fn_panic_handling!{
        *this == *other
    }
}

pub(crate) extern "C" fn cmp_ord<T>(this: CAbi<&T>, other: CAbi<&T>) -> RCmpOrdering
where
    T: Ord,
{
    extern_fn_panic_handling!{
        this.cmp(&*other).into_c()
    }
}

pub(crate) extern "C" fn partial_cmp_ord<T>(
    this: CAbi<&T>,
    other: CAbi<&T>,
) -> ROption<RCmpOrdering>
where
    T: PartialOrd,
{
    extern_fn_panic_handling!{
        this.partial_cmp(&*other).map(IntoReprC::into_c).into_c()
    }
}

//////////////////
// Hash

pub(crate) extern "C" fn hash_Hash<H>(
    this: CAbi<&H>,
    mut state: trait_objects::HasherTraitObject<&mut ErasedObject>,
) where
    H: Hash,
{
    extern_fn_panic_handling!{
        this.hash(&mut state);
    }
}

//////////////////
// Hasher

pub(crate) extern "C" fn hash_slice_Hasher<T>(this: CAbi<&mut T>, slic_: RSlice<'_, u8>)
where
    T: Hasher,
{
    extern_fn_panic_handling!{
        this.into_inner().write(slic_.into());
    }
}
pub(crate) extern "C" fn finish_Hasher<T>(this: CAbi<&T>) -> u64
where
    T: Hasher,
{
    extern_fn_panic_handling!{
        this.into_inner().finish()
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
