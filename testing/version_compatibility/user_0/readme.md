This crate tests that newer semver compatible versions of abi_stabl don't change 
the layout of publically types incompatibly.

Note that types from abi_stable::type_layout cannot be tested for layout compatibility 
since they are the very types used to do the checking.

To run this,simply `cargo run`/`cargo run --release` it.