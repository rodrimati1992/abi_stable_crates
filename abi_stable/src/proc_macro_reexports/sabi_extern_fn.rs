/**

The `sabi_extern_fn` attribute macro allows defining `extern "C"` function that
abort on unwind instead of causing undefined behavior.

This macro is syntactic sugar to transform  this:
```ignore
<visibility> fn function_name( <params> ) -> <return type> {
    <code here>
}
```
into this:
```ignore
<visibility> extern "C" fn function_name( <params> ) -> <return type> {
    ::abi_stable::extern_fn_panic_handling!{
        <code here>
    }
}
```

What this attribute does is to give the function abort on unwind semantics
(only when the unwinds doesn't stop inside the function).
A user can still use [`std::panic::catch_unwind`] inside the function to 
catch panics and handle them appropriately.

### Basic examples

```rust
use abi_stable::{sabi_extern_fn, std_types::RArc, traits::IntoReprC};

#[sabi_extern_fn]
pub(crate) fn hello() -> RArc<u32> {
    RArc::new(100)
}

assert_eq!(hello(), RArc::new(100));


```

```rust
use abi_stable::{
    sabi_extern_fn,
    std_types::{RStr, RVec},
    traits::IntoReprC,
};

#[sabi_extern_fn]
fn collect_into_lines(text: &str) -> RVec<RStr<'_>> {
    text.lines()
        .filter(|x| !x.is_empty())
        .map(RStr::from)
        .collect()
}

assert_eq!(
    collect_into_lines("what\nis\nthat"),
    vec!["what".into_c(), "is".into(), "that".into()].into_c(),
);



```

# no_early_return

You can use `#[sabi_extern_fn(no_early_return)]` to potentially 
improve the runtime performance of the annotated function (this has not been tested).

This variant of the attribute removes an intermediate closure that is 
used to intercept early returns (`?`, `return`, etc),

If this version of the attribute is used on a function which does have an 
early return, it will (incorrectly) abort the process when it attempts to return early.

### Example

```rust
use abi_stable::{
    sabi_extern_fn,
    std_types::{RStr, RVec},
    traits::IntoReprC,
};

#[sabi_extern_fn(no_early_return)]
pub(crate) fn hello() -> RVec<RStr<'static>> {
    vec!["hello".into(), "world".into()].into()
}

assert_eq!(hello(), vec!["hello".into_c(), "world".into()].into_c(),);



```


*/
#[doc(inline)]
pub use abi_stable_derive::sabi_extern_fn;