use std::env;
use std::path::{Path, PathBuf};
use image::GenericImageView;

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
        [_, command, path] if command == "flipv" => flip_vertical(path),
        [_, command, degrees, path] if command == "rotate" => rotate(degrees, path),
        [_, command, src, dst] if command == "convert" => convert(src, dst),
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

fn flip_vertical(path: &str) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let flipped = img.flipv();
    let output = output_path(path, "flipv");
    flipped
        .save(&output)
        .map_err(|e| format!("failed to save image '{}': {e}", output.display()))?;

    println!("Saved flipped image to {}", output.display());
    Ok(())
}

fn rotate(degrees: &str, path: &str) -> Result<(), String> {
    let deg: u16 = degrees
        .parse()
        .map_err(|_| format!("invalid rotation '{degrees}': use 90, 180, or 270"))?;

    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let rotated = match deg {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => return Err(format!("invalid rotation '{degrees}': use 90, 180, or 270")),
    };

    let output = output_path(path, &format!("rotate{deg}"));
    rotated
        .save(&output)
        .map_err(|e| format!("failed to save image '{}': {e}", output.display()))?;

    println!("Saved rotated image to {}", output.display());
    Ok(())
}

fn convert(src: &str, dst: &str) -> Result<(), String> {
    let mut img = image::open(src).map_err(|e| format!("failed to open image '{src}': {e}"))?;
    let dst_path = Path::new(dst);
    let ext = dst_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "ico" => {},
        _ => return Err(format!("unsupported format '{ext}': use jpg, png, or ico")),
    };

    // Resize for ICO format (max 256x256 pixels)
    if ext == "ico" {
        let (width, height) = img.dimensions();
        if width > 256 || height > 256 {
            let max_dim = width.max(height) as f32;
            let ratio = 256.0 / max_dim;
            let new_width = (width as f32 * ratio) as u32;
            let new_height = (height as f32 * ratio) as u32;
            img = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
        }
    }

    img.save(dst)
        .map_err(|e| format!("failed to save image '{dst}': {e}"))?;

    println!("Converted image to {}", dst);
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
    [
        "Usage:",
        "  simple-edit fliph <path-to-image>",
        "  simple-edit flipv <path-to-image>",
        "  simple-edit rotate <degrees> <path-to-image>",
        "  simple-edit convert <path-to-image> <new-path>",
    ]
    .join("\n")
}
