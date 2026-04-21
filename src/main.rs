use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use image::GenericImageView;
use palette::Srgba;

enum OutputMode<'a> {
    Generated(&'a str),
    Explicit(&'a str),
    Replace,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, command, flag, path] if command == "fliph" && is_replace_flag(flag) => {
            flip_horizontal(path, OutputMode::Replace)
        }
        [_, command, path] if command == "fliph" => flip_horizontal(path, OutputMode::Generated("fliph")),
        [_, command, path, output] if command == "fliph" => flip_horizontal(path, OutputMode::Explicit(output.as_str())),
        [_, command, flag, path] if command == "flipv" && is_replace_flag(flag) => {
            flip_vertical(path, OutputMode::Replace)
        }
        [_, command, path] if command == "flipv" => flip_vertical(path, OutputMode::Generated("flipv")),
        [_, command, path, output] if command == "flipv" => flip_vertical(path, OutputMode::Explicit(output.as_str())),
        [_, command, degrees, flag, path] if command == "rotate" && is_replace_flag(flag) => {
            rotate(degrees, path, OutputMode::Replace)
        }
        [_, command, degrees, path] if command == "rotate" => rotate(degrees, path, OutputMode::Generated("rotate")),
        [_, command, degrees, path, output] if command == "rotate" => rotate(degrees, path, OutputMode::Explicit(output.as_str())),
        [_, command, flag, path] if command == "invert" && is_replace_flag(flag) => {
            invert(path, OutputMode::Replace)
        }
        [_, command, path] if command == "invert" => invert(path, OutputMode::Generated("invert")),
        [_, command, path, output] if command == "invert" => invert(path, OutputMode::Explicit(output.as_str())),
        [_, command, flag, path] if command == "grayscale" && is_replace_flag(flag) => {
            grayscale(path, OutputMode::Replace)
        }
        [_, command, path] if command == "grayscale" => grayscale(path, OutputMode::Generated("grayscale")),
        [_, command, path, output] if command == "grayscale" => grayscale(path, OutputMode::Explicit(output.as_str())),
        [_, command, src, dst] if command == "convert" => convert(src, dst),
        _ => Err(usage()),
    }
}

fn flip_horizontal(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let flipped = img.fliph();
    let output_path = save_transformed_image(flipped, path, output, "fliph")?;
    println!("Saved flipped image to {}", output_path);
    Ok(())
}

fn flip_vertical(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let flipped = img.flipv();
    let output_path = save_transformed_image(flipped, path, output, "flipv")?;
    println!("Saved flipped image to {}", output_path);
    Ok(())
}

fn rotate(degrees: &str, path: &str, output: OutputMode<'_>) -> Result<(), String> {
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

    let output_path = save_transformed_image(rotated, path, output, &format!("rotate{deg}"))?;
    println!("Saved rotated image to {}", output_path);
    Ok(())
}

fn convert(src: &str, dst: &str) -> Result<(), String> {
    let img = image::open(src).map_err(|e| format!("failed to open image '{src}': {e}"))?;
    save_image(img, dst)?;
    println!("Converted image to {}", dst);
    Ok(())
}

fn invert(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let inverted = invert_colors(img);
    let output_path = save_transformed_image(inverted, path, output, "invert")?;
    println!("Saved inverted image to {}", output_path);
    Ok(())
}

fn invert_colors(img: image::DynamicImage) -> image::DynamicImage {
    let mut rgba_image = img.to_rgba8();

    for pixel in rgba_image.pixels_mut() {
        let color = Srgba::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
            pixel[3] as f32 / 255.0,
        );

        let inverted = Srgba::new(1.0 - color.red, 1.0 - color.green, 1.0 - color.blue, color.alpha);

        *pixel = image::Rgba([
            (inverted.red * 255.0).round() as u8,
            (inverted.green * 255.0).round() as u8,
            (inverted.blue * 255.0).round() as u8,
            (inverted.alpha * 255.0).round() as u8,
        ]);
    }

    image::DynamicImage::ImageRgba8(rgba_image)
}

fn grayscale(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let grayscale = img.grayscale();
    let output_path = save_transformed_image(grayscale, path, output, "grayscale")?;
    println!("Saved grayscale image to {}", output_path);
    Ok(())
}

fn save_transformed_image(
    img: image::DynamicImage,
    source_path: &str,
    output: OutputMode<'_>,
    default_suffix: &str,
) -> Result<String, String> {
    match output {
        OutputMode::Generated(suffix) => {
            let output_path = output_path_with_suffix(source_path, suffix);
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        OutputMode::Explicit(output_path) => {
            save_image(img, output_path)?;
            Ok(output_path.to_string())
        }
        OutputMode::Replace => {
            let temp_path = replacement_temp_path(source_path, default_suffix);
            save_image(img, temp_path.as_path())?;
            fs::rename(&temp_path, source_path).map_err(|e| {
                format!("failed to replace image '{}' with '{}': {e}", source_path, temp_path.display())
            })?;
            Ok(source_path.to_string())
        }
    }
}

fn save_image<P: AsRef<Path>>(mut img: image::DynamicImage, output_path: P) -> Result<(), String> {
    let output_path = output_path.as_ref();
    let dst_path = output_path;
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

    img.save(output_path)
        .map_err(|e| format!("failed to save image '{}': {e}", output_path.display()))?;

    Ok(())
}

fn replacement_temp_path(input: &str, suffix: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    parent.join(format!("{stem}_{suffix}.simple-edit-tmp.{ext}"))
}

fn output_path_with_suffix(input: &str, suffix: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    parent.join(format!("{stem}_{suffix}.{ext}"))
}

fn is_replace_flag(value: &str) -> bool {
    matches!(value, "-r" | "--replace")
}

fn usage() -> String {
    [
        "Usage:",
        "  simple-edit fliph [-r|--replace] <path-to-image> [output-path]",
        "  simple-edit flipv [-r|--replace] <path-to-image> [output-path]",
        "  simple-edit rotate <degrees> [-r|--replace] <path-to-image> [output-path]",
        "  simple-edit invert [-r|--replace] <path-to-image> [output-path]",
        "  simple-edit grayscale [-r|--replace] <path-to-image> [output-path]",
        "  simple-edit convert <path-to-image> <new-path>",
    ]
    .join("\n")
}
