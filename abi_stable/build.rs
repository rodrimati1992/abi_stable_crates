use rustc_version::{version, Version};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let rver = version().unwrap();

    if rver < Version::new(1, 33, 0) {
        panic!("\n\n`abi_stable` requires a Rust version greater than or equal to 1.33\n\n");
    }if Version::new(1, 34, 0) <= rver {
        println!("cargo:rustc-cfg=rust_1_34");
    }
}
