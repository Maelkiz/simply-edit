use std::fs;
use std::path::{Path, PathBuf};

use image::GenericImageView;

use crate::OutputMode;

pub(crate) fn save_transformed_image(
    img: image::DynamicImage,
    source_path: &str,
    output: OutputMode<'_>,
    default_suffix: &str,
) -> Result<String, String> {
    match output {
        OutputMode::Generated(suffix) => {
            let output_path = output_path_with_suffix(source_path, suffix);
            let output_path = enumerate_if_exists(&output_path);
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        OutputMode::Explicit(output_path) => {
            let output_path = enumerate_if_exists(Path::new(&output_path));
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        OutputMode::Replace(target) => {
            let target = target.as_deref().unwrap_or(source_path);
            let temp_path = replacement_temp_path(target, default_suffix);
            save_image(img, temp_path.as_path())?;
            fs::rename(&temp_path, target).map_err(|e| {
                format!(
                    "failed to replace image '{}' with '{}': {e}",
                    target,
                    temp_path.display()
                )
            })?;
            Ok(target.to_string())
        }
    }
}

pub(crate) fn save_image<P: AsRef<Path>>(
    mut img: image::DynamicImage,
    output_path: P,
) -> Result<(), String> {
    let output_path = output_path.as_ref();
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "ico" | "webp" => {}
        _ => {
            return Err(format!(
                "unsupported format '{ext}': use jpg, png, ico, or webp"
            ));
        }
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
        img = image::DynamicImage::ImageRgba8(img.to_rgba8());
    }

    img.save(output_path)
        .map_err(|e| format!("failed to save image '{}': {e}", output_path.display()))
}

pub(crate) fn replacement_temp_path(input: &str, suffix: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    parent.join(format!("{stem}_{suffix}.simple-edit-tmp.{ext}"))
}

pub(crate) fn output_path_with_suffix(input: &str, suffix: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
    parent.join(format!("{stem}_{suffix}.{ext}"))
}

pub(crate) fn enumerate_if_exists(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = path.extension().and_then(|e| e.to_str());
    let mut counter = 1u32;
    loop {
        let candidate = match ext {
            Some(ext) => parent.join(format!("{stem}{counter}.{ext}")),
            None => parent.join(format!("{stem}{counter}")),
        };
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_path_with_suffix_simple() {
        let path = output_path_with_suffix("image.jpg", "fliph");
        assert_eq!(path.to_string_lossy(), "image_fliph.jpg");
    }

    #[test]
    fn test_output_path_with_suffix_nested() {
        let path = output_path_with_suffix("path/to/image.png", "invert");
        assert_eq!(path.to_string_lossy(), "path/to/image_invert.png");
    }

    #[test]
    fn test_output_path_with_suffix_multiple_dots() {
        let path = output_path_with_suffix("my.image.file.jpg", "rotate90");
        assert_eq!(path.to_string_lossy(), "my.image.file_rotate90.jpg");
    }

    #[test]
    fn test_output_path_with_suffix_no_extension() {
        let path = output_path_with_suffix("imagefile", "grayscale");
        assert_eq!(path.to_string_lossy(), "imagefile_grayscale.png");
    }

    #[test]
    fn test_replacement_temp_path_creates_tmp_suffix() {
        let path = replacement_temp_path("image.jpg", "fliph");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("simple-edit-tmp"));
        assert!(path_str.ends_with(".jpg"));
    }

    #[test]
    fn test_replacement_temp_path_nested() {
        let path = replacement_temp_path("dir/image.png", "rotate90");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("simple-edit-tmp"));
        assert!(path_str.contains("dir/"));
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "simply-edit-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn test_enumerate_if_exists_returns_original_when_no_conflict() {
        let dir = temp_dir("enum-no-conflict");
        let path = dir.join("image.png");
        assert_eq!(enumerate_if_exists(&path), path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enumerate_if_exists_returns_stem1_when_original_exists() {
        let dir = temp_dir("enum-one");
        let path = dir.join("image.png");
        fs::write(&path, b"").expect("failed to write");
        assert_eq!(enumerate_if_exists(&path), dir.join("image1.png"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enumerate_if_exists_skips_existing_numbered_files() {
        let dir = temp_dir("enum-skip");
        let path = dir.join("image.png");
        fs::write(&path, b"").expect("failed to write");
        fs::write(dir.join("image1.png"), b"").expect("failed to write");
        assert_eq!(enumerate_if_exists(&path), dir.join("image2.png"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_enumerate_if_exists_handles_no_extension() {
        let dir = temp_dir("enum-noext");
        let path = dir.join("imagefile");
        fs::write(&path, b"").expect("failed to write");
        assert_eq!(enumerate_if_exists(&path), dir.join("imagefile1"));
        let _ = fs::remove_dir_all(&dir);
    }
}
