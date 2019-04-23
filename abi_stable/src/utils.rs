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
