use rustc_version::Version;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let rver = rustc_version::version().unwrap();

    if Version::new(1, 39, 0) <= rver {
        println!("cargo:rustc-cfg=feature=\"rust_1_39\"");
    }
}
