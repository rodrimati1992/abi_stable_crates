use rustc_version::{Version, Channel};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../readme.md");

    let rver = rustc_version::version().unwrap();

    if Version::new(1, 36, 0) <= rver {
        println!("cargo:rustc-cfg=feature=\"rust_1_36\"");
    }
    if Version::new(1, 38, 0) <= rver {
        println!("cargo:rustc-cfg=feature=\"rust_1_38\"");
    }
    if Version::new(1, 39, 0) <= rver {
        println!("cargo:rustc-cfg=feature=\"rust_1_39\"");
    }
    let channel=rustc_version::version_meta().unwrap().channel;
    if let Channel::Nightly=channel {
        println!("cargo:rustc-cfg=feature=\"nightly_rust\"");
    }

    // skeptic::generate_doc_tests(&["../readme.md"]);

}
