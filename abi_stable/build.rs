use rustc_version::Channel;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../readme.md");

    let channel=rustc_version::version_meta().unwrap().channel;
   
}
