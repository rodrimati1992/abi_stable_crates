This are example crates,including a command line application with 
text operations subcommands,
demonstrating prefix-types and `DynTrait`.

This has example_0_interface(a regular Rust crate)
as the interface crate for the example_0_impl dynamic library crate.

# Running 

To run the application (the crate in ./user/):

1. compile example_0_impl with `cargo build`/`cargo build --release`,

2. cd to `./user/`

3. run this crate with one of these:

    - `cargo run`,

    - `cargo run --release`,

    - `cargo build; ../../../target/debug/example_0_user[.exe]`

    - `cargo build --release; ../../../target/release/example_0_user[.exe]`

4. use one of the subcommands in the help message.

To get help running a subcommand you can run 
`path_to_executable/example_0_user subcommand -h`.
