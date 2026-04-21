use std::env;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, command, path] if command == "fliph" => flip_horizontal(path),
        _ => Err(usage()),
    }
}

fn flip_horizontal(path: &str) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let flipped = img.fliph();
    let output = output_path(path, "fliph");
    flipped
        .save(&output)
        .map_err(|e| format!("failed to save image '{}': {e}", output.display()))?;

    println!("Saved flipped image to {}", output.display());
    Ok(())
}

fn output_path(input: &str, suffix: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    parent.join(format!("{stem}_{suffix}.{ext}"))
}

fn usage() -> String {
    "Usage: simple-edit fliph <path-to-image>".to_string()
}
