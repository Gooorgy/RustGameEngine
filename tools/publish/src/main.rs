use asset_pipeline::cook_pending;
use project::{AssetRegistry, Project};
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (project_path, output_dir) = parse_args(&args);

    let project = Project::load(&project_path)
        .unwrap_or_else(|e| panic!("failed to load '{}': {}", project_path.display(), e));

    let registry = AssetRegistry::scan(&project, None)
        .expect("failed to scan project content directory");

    cook_pending(&registry, &project);
    registry.save(&project)
        .unwrap_or_else(|e| eprintln!("warning: could not save asset registry: {}", e));

    let project_name = &project.name;
    let project_subdir = output_dir.join(project_name);

    let eproj_filename = project_path.file_name().expect("project path has no filename");
    copy_file(&project_path, &project_subdir.join(eproj_filename));
    copy_dir(&project.content_dir, &project_subdir.join("resources"));
    copy_dir(&project.cache_dir.join("cooked"), &project_subdir.join(".cache").join("cooked"));

    println!("Published to '{}'", output_dir.display());
    println!("  Binary: place your compiled executable alongside '{}'", project_subdir.display());
}

fn parse_args(args: &[String]) -> (PathBuf, PathBuf) {
    let mut project: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => { i += 1; project = Some(PathBuf::from(&args[i])); }
            "--output"  => { i += 1; output  = Some(PathBuf::from(&args[i])); }
            other => eprintln!("warning: unknown arg '{other}'"),
        }
        i += 1;
    }
    (
        project.expect("usage: publish --project <path> --output <dir>"),
        output.expect("usage: publish --project <path> --output <dir>"),
    )
}

fn copy_file(src: &Path, dst: &Path) {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::copy(src, dst).unwrap_or_else(|e| panic!("copy '{}' failed: {}", src.display(), e));
}

fn copy_dir(src: &Path, dst: &Path) {
    if !src.exists() {
        return;
    }
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let rel = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(rel);
        copy_file(entry.path(), &target);
    }
}
