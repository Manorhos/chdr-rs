#![allow(unused)]

use std::env;
use std::path::PathBuf;

fn main() {
    let libchdr_path = cmake::Config::new("libchdr")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("INSTALL_STATIC_LIBS", "ON")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", libchdr_path.display());
    println!("cargo:rustc-link-lib=chdr-static");
    println!("cargo:rustc-link-lib=lzma");
    println!("cargo:rustc-link-lib=zlib");

    #[cfg(feature = "bindgen")] {
        println!("cargo:rerun-if-changed=wrapper.h");

        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg("-Ilibchdr/include")
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings.write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}