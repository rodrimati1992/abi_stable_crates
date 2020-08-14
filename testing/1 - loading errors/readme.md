These crates are for testing different kinds of errors that happen while loading a dynamic library.

To run the binary:
```sh
cd impl_1
cargo build
cd ../non_abi_stable_lib
cargo build
cd ../user_1
env "RETURN=ok" cargo run; env "RETURN=error" cargo run; env "RETURN=panic" cargo run


```