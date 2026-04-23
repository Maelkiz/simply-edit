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
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        OutputMode::Explicit(output_path) => {
            save_image(img, output_path.as_str())?;
            Ok(output_path)
        }
        OutputMode::Replace => {
            let temp_path = replacement_temp_path(source_path, default_suffix);
            save_image(img, temp_path.as_path())?;
            fs::rename(&temp_path, source_path).map_err(|e| {
                format!(
                    "failed to replace image '{}' with '{}': {e}",
                    source_path,
                    temp_path.display()
                )
            })?;
            Ok(source_path.to_string())
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
        "jpg" | "jpeg" | "png" | "ico" => {}
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

pub(crate) fn is_replace_flag(value: &str) -> bool {
    matches!(value, "-r" | "--replace")
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

    #[test]
    fn test_is_replace_flag_short() {
        assert!(is_replace_flag("-r"));
    }

    #[test]
    fn test_is_replace_flag_long() {
        assert!(is_replace_flag("--replace"));
    }

    #[test]
    fn test_is_replace_flag_rejects_other_short() {
        assert!(!is_replace_flag("-f"));
        assert!(!is_replace_flag("-x"));
    }

    #[test]
    fn test_is_replace_flag_rejects_other_long() {
        assert!(!is_replace_flag("--foo"));
        assert!(!is_replace_flag("--other"));
    }

    #[test]
    fn test_is_replace_flag_rejects_close_matches() {
        assert!(!is_replace_flag("replace"));
        assert!(!is_replace_flag("-replace"));
        assert!(!is_replace_flag("--r"));
    }

    #[test]
    fn test_is_replace_flag_case_sensitive() {
        assert!(!is_replace_flag("-R"));
        assert!(!is_replace_flag("--Replace"));
        assert!(!is_replace_flag("--REPLACE"));
    }
}
