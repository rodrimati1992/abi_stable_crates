use rustc_version::{Version, Channel};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../readme.md");

    let rver = rustc_version::version().unwrap();

    if Version::new(1, 42, 0) <= rver {
        println!("cargo:rustc-cfg=rust_1_42");
    }

    let channel=rustc_version::version_meta().unwrap().channel;
    if let Channel::Nightly=channel {
        println!("cargo:rustc-cfg=feature=\"nightly_rust\"");
    }
}
