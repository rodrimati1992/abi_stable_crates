#![allow(non_snake_case)]

use std::{
    fmt,
    io::{self, BufRead, Read, Write as IoWrite},
    mem, ptr,
};

use super::*;

use crate::{
    marker_type::ErasedObject,
    pointer_trait::{GetPointerKind, PK_MutReference, PK_Reference, PK_SmartPointer},
    sabi_types::{RMut, RRef},
    std_types::{RIoError, RSeekFrom},
};

use core_extensions::utils::transmute_ignore_size;

pub(crate) unsafe fn adapt_std_fmt<T>(
    value: RRef<'_, T>,
    function: unsafe extern "C" fn(RRef<'_, T>, FormattingMode, &mut RString) -> RResult<(), ()>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut buf = RString::new();
    let mode = if formatter.alternate() {
        FormattingMode::Alternate
    } else {
        FormattingMode::Default_
    };

    unsafe { function(value, mode, &mut buf) }
        .into_rust()
        .map_err(|_| fmt::Error)?;

    fmt::Display::fmt(&*buf, formatter)
}

pub(crate) unsafe extern "C" fn drop_pointer_impl<OrigP, ErasedPtr>(this: RMut<'_, ErasedPtr>) {
    extern_fn_panic_handling! {no_early_return; unsafe {
        let this = this.transmute_into_mut::<OrigP>();
        ptr::drop_in_place(this);
    }}
}

pub(crate) unsafe extern "C" fn clone_pointer_impl<OrigP, ErasedPtr>(
    this: RRef<'_, ErasedPtr>,
) -> ErasedPtr
where
    OrigP: Clone,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<OrigP>() };
        let clone = this.clone();
        unsafe { transmute_ignore_size(clone) }
    }
}

////////////////////////////////////////////////////

/*
I'm implementing DefaultImpl for all pointer kinds,
only requiring `std::default::Default` for `PK_SmartPointer`
because it is the only one for which `DynTrait::default` can be called.
*/

pub trait DefaultImpl<PtrKind> {
    fn default_impl() -> Self;
}

impl<This> DefaultImpl<PK_SmartPointer> for This
where
    Self: Default,
{
    fn default_impl() -> Self {
        Default::default()
    }
}

impl<This> DefaultImpl<PK_Reference> for This {
    fn default_impl() -> Self {
        unreachable!("This should not be called in DynTrait::default")
    }
}

impl<This> DefaultImpl<PK_MutReference> for This {
    fn default_impl() -> Self {
        unreachable!("This should not be called in DynTrait::default")
    }
}

pub(crate) unsafe extern "C" fn default_pointer_impl<OrigP, ErasedPtr>() -> ErasedPtr
where
    OrigP: GetPointerKind,
    OrigP: DefaultImpl<<OrigP as GetPointerKind>::Kind>,
{
    extern_fn_panic_handling! {no_early_return; unsafe {
        transmute_ignore_size( OrigP::default_impl() )
    }}
}

/////////////

pub(crate) unsafe extern "C" fn display_impl<T>(
    this: RRef<'_, ErasedObject>,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Display,
{
    extern_fn_panic_handling! {no_early_return;
        use std::fmt::Write;
        let this = unsafe { this.transmute_into_ref::<T>() };

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

pub(crate) unsafe extern "C" fn debug_impl<T>(
    this: RRef<'_, ErasedObject>,
    mode: FormattingMode,
    buf: &mut RString,
) -> RResult<(), ()>
where
    T: Debug,
{
    extern_fn_panic_handling! {no_early_return;
        use std::fmt::Write;

        let this = unsafe { this.transmute_into_ref::<T>() };

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

pub(crate) unsafe extern "C" fn serialize_impl<'s, T, I>(
    this: RRef<'s, ErasedObject>,
) -> RResult<<I as SerializeProxyType<'s>>::Proxy, RBoxError>
where
    T: for<'borr> SerializeType<'borr, Interface = I>,
    I: for<'borr> SerializeProxyType<'borr>,
{
    extern_fn_panic_handling! {no_early_return; unsafe {
        let ret: RResult<<I as SerializeProxyType<'_>>::Proxy, RBoxError> =
            this
            .transmute_into_ref::<T>()
            .serialize_impl()
            .into_c();

        core_extensions::utils::transmute_ignore_size(ret)
    }}
}

pub(crate) unsafe extern "C" fn partial_eq_impl<T>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> bool
where
    T: PartialEq,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<T>() };
        let other = unsafe { other.transmute_into_ref::<T>() };
        this == other
    }
}

pub(crate) unsafe extern "C" fn cmp_ord<T>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> RCmpOrdering
where
    T: Ord,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<T>() };
        let other = unsafe { other.transmute_into_ref::<T>() };
        this.cmp(other).into_c()
    }
}

pub(crate) unsafe extern "C" fn partial_cmp_ord<T>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> ROption<RCmpOrdering>
where
    T: PartialOrd,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<T>() };
        let other = unsafe { other.transmute_into_ref::<T>() };

        this.partial_cmp(other).map(IntoReprC::into_c).into_c()
    }
}

//////////////////
// Hash

pub(crate) unsafe extern "C" fn hash_Hash<T>(
    this: RRef<'_, ErasedObject>,
    mut state: trait_objects::HasherObject<'_>,
) where
    T: Hash,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<T>() };

        this.hash(&mut state);
    }
}

//////////////////
// Hasher

pub(crate) unsafe extern "C" fn write_Hasher<T>(this: RMut<'_, ErasedObject>, slic_: RSlice<'_, u8>)
where
    T: Hasher,
{
    extern_fn_panic_handling! {
        let this = unsafe { this.transmute_into_mut::<T>() };
        this.write(slic_.into());
    }
}

macro_rules! fn_write {
    ( $(($ty:ty, $delegated_fn:ident, $new_fn:ident)),* ) => {
        $(
            pub(crate) unsafe extern "C" fn $new_fn<T>(
                this: RMut<'_, ErasedObject>,
                val: $ty,
            ) where
                T: Hasher,
            {
                extern_fn_panic_handling! {
                    let this = unsafe { this.transmute_into_mut::<T>() };
                    this.$delegated_fn(val);
                }
            }
        )*
    }
}

fn_write!(
    // No c-compatible layout for u128 yet
    // (i128, write_i128, write_i128_Hasher),
    (i16, write_i16, write_i16_Hasher),
    (i32, write_i32, write_i32_Hasher),
    (i64, write_i64, write_i64_Hasher),
    (i8, write_i8, write_i8_Hasher),
    (isize, write_isize, write_isize_Hasher),
    // No c-compatible layout for u128 yet
    // (u128, write_u128, write_u128_Hasher),
    (u16, write_u16, write_u16_Hasher),
    (u32, write_u32, write_u32_Hasher),
    (u64, write_u64, write_u64_Hasher),
    (u8, write_u8, write_u8_Hasher),
    (usize, write_usize, write_usize_Hasher)
);

pub(crate) unsafe extern "C" fn finish_Hasher<T>(this: RRef<'_, ErasedObject>) -> u64
where
    T: Hasher,
{
    extern_fn_panic_handling! {
        let this = unsafe { this.transmute_into_ref::<T>() };

        this.finish()
    }
}

//////////////////////////////////////////////////////////////////////////////////////
////                        fmt
//////////////////////////////////////////////////////////////////////////////////////

pub(super) unsafe extern "C" fn write_str_fmt_write<T>(
    this: RMut<'_, ErasedObject>,
    data: RStr<'_>,
) -> RResult<(), ()>
where
    T: fmt::Write,
{
    extern_fn_panic_handling! {
        let this = unsafe { this.transmute_into_mut::<T>() };
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
fn convert_io_result<T, U>(res: io::Result<T>) -> RResult<U, RIoError>
where
    T: Into<U>,
{
    match res {
        Ok(v) => ROk(v.into()),
        Err(e) => RErr(RIoError::from(e)),
    }
}

///////////////////////////

#[repr(C)]
#[derive(StableAbi, Copy, Clone)]
pub struct IoWriteFns {
    pub(super) write: unsafe extern "C" fn(
        RMut<'_, ErasedObject>,
        buf: RSlice<'_, u8>,
    ) -> RResult<usize, RIoError>,

    pub(super) write_all:
        unsafe extern "C" fn(RMut<'_, ErasedObject>, buf: RSlice<'_, u8>) -> RResult<(), RIoError>,

    pub(super) flush: unsafe extern "C" fn(RMut<'_, ErasedObject>) -> RResult<(), RIoError>,
}

pub(super) struct MakeIoWriteFns<W>(W);

impl<W> MakeIoWriteFns<W>
where
    W: IoWrite,
{
    pub(super) const NEW: IoWriteFns = IoWriteFns {
        write: io_Write_write::<W>,
        write_all: io_Write_write_all::<W>,
        flush: io_Write_flush::<W>,
    };
}

pub(super) unsafe extern "C" fn io_Write_write<W>(
    this: RMut<'_, ErasedObject>,
    buf: RSlice<'_, u8>,
) -> RResult<usize, RIoError>
where
    W: IoWrite,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<W>() };

        convert_io_result(this.write(buf.into()))
    }
}

pub(super) unsafe extern "C" fn io_Write_write_all<W>(
    this: RMut<'_, ErasedObject>,
    buf: RSlice<'_, u8>,
) -> RResult<(), RIoError>
where
    W: IoWrite,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<W>() };

        convert_io_result(this.write_all(buf.into()))
    }
}

pub(super) unsafe extern "C" fn io_Write_flush<W>(
    this: RMut<'_, ErasedObject>,
) -> RResult<(), RIoError>
where
    W: IoWrite,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<W>() };

        convert_io_result(this.flush())
    }
}

///////////////////////////

#[repr(C)]
#[derive(StableAbi, Copy, Clone)]
pub struct IoReadFns {
    pub(super) read:
        unsafe extern "C" fn(RMut<'_, ErasedObject>, RSliceMut<'_, u8>) -> RResult<usize, RIoError>,

    pub(super) read_exact:
        unsafe extern "C" fn(RMut<'_, ErasedObject>, RSliceMut<'_, u8>) -> RResult<(), RIoError>,
}

pub(super) struct MakeIoReadFns<W>(W);

impl<W> MakeIoReadFns<W>
where
    W: io::Read,
{
    pub(super) const NEW: IoReadFns = IoReadFns {
        read: io_Read_read::<W>,
        read_exact: io_Read_read_exact::<W>,
    };
}

pub(super) unsafe extern "C" fn io_Read_read<R>(
    this: RMut<'_, ErasedObject>,
    buf: RSliceMut<'_, u8>,
) -> RResult<usize, RIoError>
where
    R: Read,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<R>() };

        convert_io_result(this.read(buf.into()))
    }
}

pub(super) unsafe extern "C" fn io_Read_read_exact<R>(
    this: RMut<'_, ErasedObject>,
    buf: RSliceMut<'_, u8>,
) -> RResult<(), RIoError>
where
    R: Read,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<R>() };

        convert_io_result(this.read_exact(buf.into()))
    }
}

///////////////////////////

#[repr(C)]
#[derive(StableAbi, Copy, Clone)]
pub struct IoBufReadFns {
    pub(super) fill_buf:
        unsafe extern "C" fn(RMut<'_, ErasedObject>) -> RResult<RSlice<'_, u8>, RIoError>,

    pub(super) consume: unsafe extern "C" fn(RMut<'_, ErasedObject>, usize),
}

pub(super) struct MakeIoBufReadFns<W>(W);

impl<W> MakeIoBufReadFns<W>
where
    W: io::BufRead,
{
    pub(super) const NEW: IoBufReadFns = IoBufReadFns {
        fill_buf: io_BufRead_fill_buf::<W>,
        consume: io_BufRead_consume::<W>,
    };
}

pub(super) unsafe extern "C" fn io_BufRead_fill_buf<R>(
    this: RMut<'_, ErasedObject>,
) -> RResult<RSlice<'_, u8>, RIoError>
where
    R: BufRead,
{
    extern_fn_panic_handling! {no_early_return; unsafe {
        let this = this.transmute_into_mut::<R>();

        // safety: the lifetime is guaranteed correct because the returned lifetime is
        // the same as the input lifetime,
        //
        // This is a workaround to avoid having to write a `R: BufRead + 'a` bound
        mem::transmute::<
            RResult<RSlice<'_,u8>,RIoError>,
            RResult<RSlice<'_,u8>,RIoError>
        >(convert_io_result(this.fill_buf()))
    }}
}

pub(super) unsafe extern "C" fn io_BufRead_consume<R>(this: RMut<'_, ErasedObject>, amount: usize)
where
    R: BufRead,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<R>() };

        this.consume(amount)
    }
}

///////////////////////////

pub(super) unsafe extern "C" fn io_Seek_seek<S>(
    this: RMut<'_, ErasedObject>,
    seek_from: RSeekFrom,
) -> RResult<u64, RIoError>
where
    S: io::Seek,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<S>() };

        convert_io_result(this.seek(seek_from.into()))
    }
}
