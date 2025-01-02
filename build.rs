use std::{env, fs};
use std::path::{Path};
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=resources");
    let resource_dir = Path::new("resources");

    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let out_path = Path::new(&manifest_dir_string).join("target").join(build_type);
    iterate_copy(&resource_dir, &out_path);
}

fn iterate_copy(in_path: &Path, out_path: &Path) {
    for entry in WalkDir::new(in_path).into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {

        let final_out_path = out_path.join(entry.path());

        // Get the parent in order to create subdirectories in output path
        let final_out_dir = final_out_path.parent().unwrap();
        fs::create_dir_all(final_out_dir).unwrap();
        fs::copy(entry.into_path(), final_out_path).unwrap();
    }
}
