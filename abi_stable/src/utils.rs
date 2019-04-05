use std::mem;

pub const fn coerce_static<'a, T: ?Sized>(r: &'static T) -> &'static T {
    r
}

/// To coerce array references to slices.
pub const fn as_slice<'a, T>(v: &'a [T]) -> &'a [T] {
    v
}

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

pub const fn assert_zero_sized<T>() -> usize {
    0 - mem::size_of::<T>()
}

#[inline(never)]
#[cold]
pub fn ffi_panic_message(file: &'static str, line: u32) -> ! {
    eprintln!("\nfile:{}\nline:{}", file, line);
    eprintln!("Attempted to panic across the ffi boundary.");
    eprintln!("Aborting to handle the panic...\n");
    ::std::process::abort();
}
