// Copyright (c) EVA-OS. All rights reserved.
// Licensed under the MIT License.

fn main() {
    // Generate C header using cbindgen
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("EVA_NPU_H")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("eva_npu.h");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
