use asset_pipeline::codegen::generate_asset_constants;
use std::path::PathBuf;

fn main() {
    let content_dir = PathBuf::from("resources");

    // Rerun whenever any file in the resources directory changes.
    println!("cargo:rerun-if-changed=resources");

    let code = generate_asset_constants(&content_dir);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    std::fs::write(format!("{}/assets.rs", out_dir), code)
        .expect("failed to write assets.rs");
}
