use std::fs;
use std::path::{Path, PathBuf};

use image::GenericImageView;

use crate::SaveMode;

pub(crate) fn save_transformed_image(
    img: image::DynamicImage,
    source_path: &str,
    output: SaveMode,
    default_suffix: &str,
) -> Result<String, String> {
    match output {
        SaveMode::Generated(suffix, dir) => {
            let output_path = place_in(output_path_with_suffix(source_path, &suffix), dir);
            let output_path = enumerate_if_exists(&output_path);
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        SaveMode::Explicit(output_path) => {
            // Overwrite the specified path — no enumeration.
            let output_path = PathBuf::from(&output_path);
            save_image(img, output_path.as_path())?;
            Ok(output_path.to_string_lossy().to_string())
        }
        SaveMode::Replace(target) => {
            let target = target.as_deref().unwrap_or(source_path);
            let temp_path = replacement_temp_path(target, default_suffix);
            save_image(img, temp_path.as_path())?;
            fs::rename(&temp_path, target).map_err(|e| {
                if fs::remove_file(&temp_path).is_err() {
                    format!(
                        "failed to replace '{}': {e}; could not remove temporary file '{}'",
                        target,
                        temp_path.display()
                    )
                } else {
                    format!("failed to replace '{}': {e}", target)
                }
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

/// Returns an extension that can carry an alpha channel.
///
/// JPEG (and anything unrecognised) cannot store transparency, so callers that
/// produce RGBA output fall back to PNG rather than silently dropping alpha.
pub(crate) fn alpha_safe_ext(src_ext: &str) -> &'static str {
    match src_ext.to_lowercase().as_str() {
        "webp" => "webp",
        _ => "png",
    }
}

/// Like [`output_path_with_suffix`], but forces the output extension instead of
/// inheriting the source one.
pub(crate) fn output_path_with_suffix_ext(input: &str, suffix: &str, ext: &str) -> PathBuf {
    let path = Path::new(input);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    parent.join(format!("{stem}_{suffix}.{ext}"))
}

/// Interprets `path` as a target directory the way `cp` and `mv` do: an existing
/// directory, or any path ending in a separator (created if missing).
pub(crate) fn as_output_dir(path: &str) -> Result<Option<PathBuf>, String> {
    let dir = Path::new(path);
    if dir.is_dir() {
        return Ok(Some(dir.to_path_buf()));
    }
    if !path.ends_with('/') && !path.ends_with(std::path::MAIN_SEPARATOR) {
        return Ok(None);
    }
    // `file/` never "exists" on Unix, so check the path without its separator.
    if Path::new(path.trim_end_matches(['/', std::path::MAIN_SEPARATOR])).exists() {
        return Err(format!("output path '{path}' is not a directory"));
    }
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create output directory '{path}': {e}"))?;
    Ok(Some(dir.to_path_buf()))
}

/// Moves a generated output path into `dir`, keeping its file name.
pub(crate) fn place_in(path: PathBuf, dir: Option<PathBuf>) -> PathBuf {
    match (dir, path.file_name()) {
        (Some(dir), Some(name)) => dir.join(name),
        _ => path,
    }
}

/// Output path for a format conversion: `dst` as given, or `<stem>.<ext>` next
/// to the source or inside `dst` when it names a directory.
pub(crate) fn default_output_path(
    src: &str,
    dst: Option<&str>,
    ext: &str,
) -> Result<String, String> {
    let dir = match dst {
        Some(d) => match as_output_dir(d)? {
            Some(dir) => Some(dir),
            None => return Ok(d.to_string()),
        },
        None => None,
    };
    let path = place_in(Path::new(src).with_extension(ext), dir);
    Ok(path.to_string_lossy().to_string())
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
    use crate::testutil::temp_dir;

    #[test]
    fn test_as_output_dir_existing_dir() {
        let dir = temp_dir("io-outdir-existing");
        let got = as_output_dir(dir.to_str().unwrap()).unwrap();
        assert_eq!(got, Some(dir.clone()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_as_output_dir_file_path_is_not_dir() {
        assert_eq!(as_output_dir("does/not/exist.png").unwrap(), None);
    }

    #[test]
    fn test_as_output_dir_trailing_slash_creates_dir() {
        let dir = temp_dir("io-outdir-create");
        let target = dir.join("new").join("nested");
        let arg = format!("{}/", target.display());
        let got = as_output_dir(&arg).unwrap();
        assert!(target.is_dir());
        assert_eq!(got, Some(PathBuf::from(&arg)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_as_output_dir_trailing_slash_on_file_errors() {
        let dir = temp_dir("io-outdir-file");
        let file = dir.join("taken");
        fs::write(&file, b"x").unwrap();
        let err = as_output_dir(&format!("{}/", file.display())).unwrap_err();
        assert!(err.contains("not a directory"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_place_in_moves_file_name_into_dir() {
        let got = place_in(
            PathBuf::from("src/img_invert.png"),
            Some(PathBuf::from("out")),
        );
        assert_eq!(got, PathBuf::from("out/img_invert.png"));
        let got = place_in(PathBuf::from("src/img_invert.png"), None);
        assert_eq!(got, PathBuf::from("src/img_invert.png"));
    }

    #[test]
    fn test_default_output_path_variants() {
        let dir = temp_dir("io-default-out");
        let d = dir.to_str().unwrap();
        assert_eq!(
            default_output_path("a/img.png", None, "svg").unwrap(),
            "a/img.svg"
        );
        assert_eq!(
            default_output_path("a/img.png", Some("x.svg"), "svg").unwrap(),
            "x.svg"
        );
        assert_eq!(
            default_output_path("a/img.png", Some(d), "svg").unwrap(),
            dir.join("img.svg").to_string_lossy()
        );
        let _ = fs::remove_dir_all(&dir);
    }

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
    fn test_alpha_safe_ext_preserves_alpha_capable_formats() {
        assert_eq!(alpha_safe_ext("png"), "png");
        assert_eq!(alpha_safe_ext("webp"), "webp");
    }

    #[test]
    fn test_alpha_safe_ext_falls_back_to_png() {
        assert_eq!(alpha_safe_ext("jpg"), "png");
        assert_eq!(alpha_safe_ext("jpeg"), "png");
        assert_eq!(alpha_safe_ext("ico"), "png");
        assert_eq!(alpha_safe_ext("bmp"), "png");
        assert_eq!(alpha_safe_ext(""), "png");
    }

    #[test]
    fn test_alpha_safe_ext_is_case_insensitive() {
        assert_eq!(alpha_safe_ext("PNG"), "png");
        assert_eq!(alpha_safe_ext("WebP"), "webp");
        assert_eq!(alpha_safe_ext("JPG"), "png");
    }

    #[test]
    fn test_output_path_with_suffix_ext_forces_extension() {
        let path = output_path_with_suffix_ext("a/b.jpg", "cutout", "png");
        assert_eq!(path.to_string_lossy(), "a/b_cutout.png");
    }

    #[test]
    fn test_output_path_with_suffix_ext_same_extension() {
        let path = output_path_with_suffix_ext("image.png", "cutout", "png");
        assert_eq!(path.to_string_lossy(), "image_cutout.png");
    }

    #[test]
    fn test_output_path_with_suffix_ext_multiple_dots() {
        let path = output_path_with_suffix_ext("my.image.file.jpg", "cutout", "png");
        assert_eq!(path.to_string_lossy(), "my.image.file_cutout.png");
    }

    #[test]
    fn test_output_path_with_suffix_ext_no_extension() {
        let path = output_path_with_suffix_ext("imagefile", "cutout", "png");
        assert_eq!(path.to_string_lossy(), "imagefile_cutout.png");
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
