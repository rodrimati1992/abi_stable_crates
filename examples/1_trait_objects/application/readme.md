This is an example application crate,a command line application with 
text operations subcommands.

This crate uses example_0_interface as a regular Rust dependency,
and loads example_0_impl at runtime as a dynamic library.

# Running 

To run this application:

1.    compile all the plugins with `cargo build`/`cargo build --release`,

2.    run this crate with one of these:

    - `cargo run`,

    - `cargo run --release`,

    - `cargo build; ../../../target/debug/example_1_application[.exe]`

    - `cargo build --release; ../../../target/release/example_1_application[.exe]`

