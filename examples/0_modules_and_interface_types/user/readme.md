This is an example application crate,a command line application with 
text operations subcommands.

This crate uses example_0_interface as a regular Rust dependency,
and loads example_0_impl at runtime as a dynamic library.

# Running 

To run this application:

1.    compile example_0_impl with `cargo build`/`cargo build --release`,

2.    run this crate with one of these:

    - `cargo run`,

    - `cargo run --release`,

    - `cargo build; ../../../target/debug/example_0_user[.exe]`

    - `cargo build --release; ../../../target/release/example_0_user[.exe]`

3.    use one of the subcommands in the help message.

To get help running a subcommand you can run 
`path_to_executable/example_0_user subcommand -h`.
