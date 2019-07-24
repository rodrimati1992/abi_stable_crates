These is an example crates,including a command line application,
which demonstrate the definition and usage of ffi-safe trait objects (using `#[sabi_trait]`).

This has `example_1_interface`(a regular Rust crate)
as the interface crate for the `plugin_*` dynamic library crates.

# Running 

To run this application:

1. compile all the plugins with `cargo build`/`cargo build --release`,

2. cd to `./application/`

3. run that crate with one of these:

    - `cargo run`,

    - `cargo run --release`,

    - `cargo build; ../../../target/debug/example_1_application[.exe]`

    - `cargo build --release; ../../../target/release/example_1_application[.exe]`

