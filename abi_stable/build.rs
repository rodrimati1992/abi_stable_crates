use rustc_version::{version, Version};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../readme.md");

    let rver = version().unwrap();

    if Version::new(1, 36, 0) <= rver {
        println!("cargo:rustc-cfg=rust_1_36");
    }
    if Version::new(1, 38, 0) <= rver {
        println!("cargo:rustc-cfg=rust_1_38");
    }

    // skeptic::generate_doc_tests(&["../readme.md"]);

}
