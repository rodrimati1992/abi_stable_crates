// keeping the build.rs just in case that I want to detect
// newer language versions for soundness fixes that require them.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let _channel = rustc_version::version_meta().unwrap().channel;
}
