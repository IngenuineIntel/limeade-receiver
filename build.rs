// build.rs

use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let local_include = manifest_dir.join("../liblimeade/include");
    let local_header = local_include.join("liblimeade/liblimeade.h");
    let (header, include_dir, local_lib) = if local_header.exists() {
        (
            local_header,
            local_include,
            Some(manifest_dir.join("../liblimeade")),
        )
    } else {
        let system_include = PathBuf::from("/usr/local/include");
        (
            system_include.join("liblimeade/liblimeade.h"),
            system_include,
            None,
        )
    };

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("limeade_.*")
        .allowlist_type("limeade_.*")
        .allowlist_var("LIMEADE_ERROR_REPRS")
        .generate()
        .expect("Failed to generate bindings for liblimeade");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out.join("bindings.rs"))
        .expect("Failed to write bindings for liblimeade");
    
    if let Some(local_lib) = local_lib {
        println!("cargo:rustc-link-search=native={}", local_lib.display());
    }
    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=limeade");
    println!("cargo:rerun-if-changed={}", header.display());
}


