extern crate rustc_version;
use rustc_version::{version_matches};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if version_matches("1.9.0") {
        println!("cargo:rustc-cfg=cannot_use_dotdotdot");
    }
}
