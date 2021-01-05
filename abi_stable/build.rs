use rustc_version::{Version, Channel};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../readme.md");

    let rver = rustc_version::version().unwrap();

    if Version::new(1, 42, 0) <= rver {
        println!("cargo:rustc-cfg=rust_1_42");
    }

    let channel=rustc_version::version_meta().unwrap().channel;
    if channel == Channel::Nightly && std::env::var("CARGO_FEATURE_test_const_params").is_ok() {
        println!("cargo:warning=ENABLED CONST GENERICS");

        println!("cargo:rustc-cfg=feature=\"nightly_const_params\"");
        println!("cargo:rustc-cfg=feature=\"const_params\"");
    }


    
}
