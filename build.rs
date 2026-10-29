// build.rs

use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("/usr/local/include/liblimeade/liblimeade.h")
        .clang_arg("-I/usr/include/liblimeade")
        .clang_arg("-I/usr/local/include/liblimeade")
        .allowlist_function("limeade_.*")
        .allowlist_type("limeade_.*")
        .generate()
        .expect("Failed to generate bindings for liblimeade");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out.join("bindings.rs"))
        .expect("Failed to write bindings for liblimeade");
    
    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=limeade");
    println!("cargo:rerun-if-changed=/usr/include/liblimeade/liblimeade.h");
    println!("cargo:rerun-if-changed=/usr/local/include/liblimeade/liblimeade.h");
}


