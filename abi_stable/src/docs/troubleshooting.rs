/*!

Here are some problems and their solutions

# Opaque compiletime errors

As of writing this section,having `extern fn` in a type definition causes
compile-time errors for `#[derive(StableAbi)]` to look like this

```text
error: unknown lifetime
```
ẁhere it doesn't point at what the cause of the error is.

To fix this,replace `extern fn` with `extern "C" fn`
and the error message will look like this:

```text
error: unknown lifetime
   --> abi_stable/src/lib.rs:313:6
    |
313 |   a:&'b (),
    |      ^^
```



*/
