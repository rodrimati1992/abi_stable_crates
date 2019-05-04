#![allow(non_snake_case)]

use std::{
    fmt,
    io::{self,Write as IoWrite,Read,BufRead,Seek},
    ptr,
    mem,
};

use super::*;

use crate::{
    marker_type::ErasedObject,
    utils::{transmute_reference,transmute_mut_reference},
    std_types::{
        RIoError,
        RSeekFrom,
    },
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
    T: SerializeImplType,
{
    extern_fn_panic_handling! {unsafe{
        let this=transmute_reference::<ErasedObject,T>(this);
        this.serialize_impl()
            .map(|x| mem::transmute::<RCow<'_,RStr<'_>>,RCow<'a, RStr<'a>>>(x) )
            .into()
    }}
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
    mut state: trait_objects::HasherObject<'_>,
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

//////////////////////////////////////////////////////////////////////////////////////
////                        fmt
//////////////////////////////////////////////////////////////////////////////////////


pub(super) extern fn write_str_fmt_write<T>(
    this:&mut ErasedObject, 
    data:RStr<'_>
) -> RResult<(), ()>
where T:fmt::Write,
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,T>(this) };
        match fmt::Write::write_str(this,data.as_str()) {
            Ok(())=>ROk(()),
            Err(_)=>RErr(()),
        }
    }
}


//////////////////////////////////////////////////////////////////////////////////////
////                         io
//////////////////////////////////////////////////////////////////////////////////////



#[inline]
fn convert_io_result<T,U>(res:io::Result<T>)->RResult<U,RIoError>
where
    T:Into<U>
{
    match res {
        Ok(v)=>ROk(v.into()),
        Err(e)=>RErr(RIoError::from(e)),
    }
}


///////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Copy,Clone)]
pub struct IoWriteFns{
    pub(super) write:
        extern "C" fn (
            &mut ErasedObject,
            buf: RSlice<'_,u8>
        ) -> RResult<usize,RIoError>,

    pub(super) write_all:
        extern "C" fn (
            &mut ErasedObject,
            buf: RSlice<'_,u8>
        ) -> RResult<(),RIoError>,

    pub(super) flush:extern "C" fn (&mut ErasedObject) -> RResult<(),RIoError>,
}


pub(super) struct MakeIoWriteFns<W>(W);

impl<W> MakeIoWriteFns<W>
where
    W:IoWrite
{
    pub(super) const NEW:IoWriteFns=IoWriteFns{
        write:io_Write_write::<W>,
        write_all:io_Write_write_all::<W>,
        flush:io_Write_flush::<W>,
    };
}


pub(super) extern "C" fn io_Write_write<W>(
    this:&mut ErasedObject, 
    buf: RSlice<'_,u8>
) -> RResult<usize,RIoError>
where W:IoWrite
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,W>(this) };

        convert_io_result(this.write(buf.into()))
    }
}

pub(super) extern "C" fn io_Write_write_all<W>(
    this:&mut ErasedObject, 
    buf: RSlice<'_,u8>
) -> RResult<(),RIoError>
where W:IoWrite
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,W>(this) };

        convert_io_result(this.write_all(buf.into()))
    }
}

pub(super) extern "C" fn io_Write_flush<W>( this:&mut ErasedObject ) -> RResult<(),RIoError>
where W:IoWrite
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,W>(this) };

        convert_io_result(this.flush())
    }
}


///////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Copy,Clone)]
pub struct IoReadFns{
    pub(super) read:
        extern "C" fn(&mut ErasedObject,RSliceMut<'_,u8>) -> RResult<usize,RIoError>,

    pub(super) read_exact:
        extern "C" fn(&mut ErasedObject,RSliceMut<'_,u8>) -> RResult<(),RIoError>,
}


pub(super) struct MakeIoReadFns<W>(W);

impl<W> MakeIoReadFns<W>
where
    W:io::Read
{
    pub(super) const NEW:IoReadFns=IoReadFns{
        read:io_Read_read::<W>,
        read_exact:io_Read_read_exact::<W>,
    };
}


pub(super) extern "C" fn io_Read_read<R>(
    this:&mut ErasedObject, 
    buf: RSliceMut<'_,u8>
) -> RResult<usize,RIoError>
where R:Read
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,R>(this) };

        convert_io_result(this.read(buf.into()))
    }
}


pub(super) extern "C" fn io_Read_read_exact<R>(
    this:&mut ErasedObject, 
    buf: RSliceMut<'_,u8>
) -> RResult<(),RIoError>
where R:Read
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,R>(this) };

        convert_io_result(this.read_exact(buf.into()))
    }
}

///////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Copy,Clone)]
pub struct IoBufReadFns{
    pub(super) fill_buf:
        extern "C" fn(&mut ErasedObject) -> RResult<RSlice<'_,u8>,RIoError>,

    pub(super) consume:extern "C" fn(&mut ErasedObject,usize)
}


pub(super) struct MakeIoBufReadFns<W>(W);

impl<W> MakeIoBufReadFns<W>
where
    W:io::BufRead
{
    pub(super) const NEW:IoBufReadFns=IoBufReadFns{
        fill_buf:io_BufRead_fill_buf::<W>,
        consume:io_BufRead_consume::<W>,
    };
}

pub(super) extern "C" fn io_BufRead_fill_buf<'a,R>(
    this:&mut ErasedObject,
) -> RResult<RSlice<'_,u8>,RIoError>
where R:BufRead
{
    extern_fn_panic_handling! {unsafe{
        let this=transmute_mut_reference::<ErasedObject,R>(this);

        mem::transmute::<
            RResult<RSlice<'_,u8>,RIoError>,
            RResult<RSlice<'_,u8>,RIoError>
        >(convert_io_result(this.fill_buf()))
    }}
}


pub(super) extern "C" fn io_BufRead_consume<R>(
    this:&mut ErasedObject, 
    ammount: usize
)where 
    R:BufRead
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,R>(this) };

        this.consume(ammount)
    }
}


///////////////////////////


pub(super) extern "C" fn io_Seek_seek<S>(
    this:&mut ErasedObject,
    seek_from:RSeekFrom,
) -> RResult<u64,RIoError>
where 
    S:io::Seek
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,S>(this) };

        convert_io_result(this.seek(seek_from.into()))
    }
}
