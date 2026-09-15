use std::collections::VecDeque;
use std::path::Path;

use image::{DynamicImage, RgbaImage};

use crate::OutputMode;
use crate::io;

/// Suffix used for generated output paths.
pub(crate) const SUFFIX: &str = "cutout";

/// Default neighbour-distance tolerance for [`fast_cutout`].
pub(crate) const DEFAULT_TOLERANCE: f32 = 12.0;

pub(crate) struct CutoutArgs {
    pub src: String,
    pub tolerance: f32,
    pub output: OutputMode,
}

/// Removes a flat background by flood-filling inward from the image border.
///
/// Each candidate pixel is compared against the *neighbour* it was reached
/// from, not against a global background colour: a global model is not
/// discriminative on gradient backgrounds, and comparing locally lets the fill
/// follow a smooth gradient while still stopping at a subject edge.
///
/// Because the fill is connectivity-bound, a background-coloured region fully
/// enclosed by the subject is never reached and stays opaque.
pub(crate) fn fast_cutout(img: &DynamicImage, tolerance: f32) -> RgbaImage {
    let mut out = img.to_rgba8();
    let (width, height) = (out.width(), out.height());
    if width == 0 || height == 0 {
        return out;
    }

    let idx = |x: u32, y: u32| (y * width + x) as usize;
    let rgb = |x: u32, y: u32| {
        let p = out.get_pixel(x, y).0;
        [p[0] as f32, p[1] as f32, p[2] as f32]
    };
    let distance = |a: [f32; 3], b: [f32; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };

    let mut is_background = vec![false; (width as usize) * (height as usize)];
    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();

    let seed = |x: u32, y: u32, bg: &mut Vec<bool>, q: &mut VecDeque<(u32, u32)>| {
        if !bg[idx(x, y)] {
            bg[idx(x, y)] = true;
            q.push_back((x, y));
        }
    };
    for x in 0..width {
        seed(x, 0, &mut is_background, &mut queue);
        seed(x, height - 1, &mut is_background, &mut queue);
    }
    for y in 0..height {
        seed(0, y, &mut is_background, &mut queue);
        seed(width - 1, y, &mut is_background, &mut queue);
    }

    while let Some((x, y)) = queue.pop_front() {
        let current = rgb(x, y);
        let mut neighbours = [(0u32, 0u32); 4];
        let mut count = 0;
        if x > 0 {
            neighbours[count] = (x - 1, y);
            count += 1;
        }
        if x + 1 < width {
            neighbours[count] = (x + 1, y);
            count += 1;
        }
        if y > 0 {
            neighbours[count] = (x, y - 1);
            count += 1;
        }
        if y + 1 < height {
            neighbours[count] = (x, y + 1);
            count += 1;
        }

        for &(nx, ny) in &neighbours[..count] {
            if is_background[idx(nx, ny)] {
                continue;
            }
            if distance(current, rgb(nx, ny)) <= tolerance {
                is_background[idx(nx, ny)] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            if is_background[idx(x, y)] {
                out.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }

    out
}

/// Batch output path for `input`, forcing an alpha-capable extension.
pub(crate) fn batch_output_path(
    input: &Path,
    options: &crate::batch::BatchOptions,
) -> std::path::PathBuf {
    let ext = io::alpha_safe_ext(input.extension().and_then(|e| e.to_str()).unwrap_or(""));
    crate::batch::resolve_output_path_with_suffix_ext(input, SUFFIX, ext, options)
}

pub(crate) fn run_cutout(args: CutoutArgs) -> Result<(), String> {
    let CutoutArgs {
        src,
        tolerance,
        output,
    } = args;

    if super::convert::is_svg_path(&src) {
        return Err(format!(
            "cutout: unsupported file format '{}': cutout works on raster images",
            Path::new(&src).display()
        ));
    }

    let img =
        image::open(&src).map_err(|e| format!("cutout: failed to open image '{src}': {e}"))?;

    let result = fast_cutout(&img, tolerance);

    if let Some(output_path) = dispatch_save(DynamicImage::ImageRgba8(result), &src, output)? {
        println!("Saved cutout to {output_path}");
    }
    Ok(())
}

fn extension_of(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// Rejects destinations that cannot store an alpha channel.
///
/// Without this, saving a cutout as JPEG would silently discard the
/// transparency the command exists to produce.
fn reject_opaque_destination(path: &str) -> Result<(), String> {
    match extension_of(path).as_str() {
        "jpg" | "jpeg" => Err(format!(
            "cutout: '{path}' cannot store transparency: use png or webp"
        )),
        _ => Ok(()),
    }
}

/// `--replace` rewrites the original, so it is only allowed when that file can
/// hold the alpha channel the cutout produces.
fn reject_opaque_replace_target(path: &str) -> Result<(), String> {
    match extension_of(path).as_str() {
        "png" | "webp" => Ok(()),
        _ => Err(format!(
            "cutout: cannot replace '{path}': only png and webp can store transparency"
        )),
    }
}

fn dispatch_save(
    img: DynamicImage,
    source: &str,
    output: OutputMode,
) -> Result<Option<String>, String> {
    match output {
        OutputMode::Preview => {
            super::view::display_image(img)?;
            Ok(None)
        }
        OutputMode::Generated => {
            let ext = io::alpha_safe_ext(&extension_of(source));
            let path = io::output_path_with_suffix_ext(source, SUFFIX, ext);
            let path = io::enumerate_if_exists(&path);
            io::save_image(img, path.as_path())?;
            Ok(Some(path.to_string_lossy().to_string()))
        }
        OutputMode::Explicit(dst) => {
            reject_opaque_destination(&dst)?;
            let path =
                io::save_transformed_image(img, source, crate::SaveMode::Explicit(dst), SUFFIX)?;
            Ok(Some(path))
        }
        OutputMode::Replace(target) => {
            reject_opaque_replace_target(target.as_deref().unwrap_or(source))?;
            let path =
                io::save_transformed_image(img, source, crate::SaveMode::Replace(target), SUFFIX)?;
            Ok(Some(path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// White canvas with an opaque red rectangle that encloses a white pocket.
    fn ringed_image() -> DynamicImage {
        let mut img = RgbaImage::from_pixel(20, 20, Rgba([255, 255, 255, 255]));
        for y in 5..15 {
            for x in 5..15 {
                img.put_pixel(x, y, Rgba([200, 30, 30, 255]));
            }
        }
        for y in 8..12 {
            for x in 8..12 {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_fast_cutout_clears_flat_background() {
        let out = fast_cutout(&ringed_image(), DEFAULT_TOLERANCE);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
        assert_eq!(out.get_pixel(19, 19).0[3], 0);
    }

    #[test]
    fn test_fast_cutout_preserves_subject() {
        let out = fast_cutout(&ringed_image(), DEFAULT_TOLERANCE);
        assert_eq!(out.get_pixel(6, 6).0, [200, 30, 30, 255]);
    }

    #[test]
    fn test_fast_cutout_preserves_enclosed_pocket() {
        // The pocket is the same colour as the background but unreachable from
        // the border, so connectivity must keep it opaque.
        let out = fast_cutout(&ringed_image(), DEFAULT_TOLERANCE);
        assert_eq!(out.get_pixel(9, 9).0, [255, 255, 255, 255]);
    }

    #[test]
    fn test_fast_cutout_zero_tolerance_still_clears_uniform_background() {
        let out = fast_cutout(&ringed_image(), 0.0);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
        assert_eq!(out.get_pixel(6, 6).0[3], 255);
    }

    #[test]
    fn test_fast_cutout_preserves_dimensions() {
        let out = fast_cutout(&ringed_image(), DEFAULT_TOLERANCE);
        assert_eq!((out.width(), out.height()), (20, 20));
    }

    #[test]
    fn test_reject_opaque_destination_rejects_jpeg() {
        assert!(reject_opaque_destination("a/b.jpg").is_err());
        assert!(reject_opaque_destination("a/b.JPEG").is_err());
    }

    #[test]
    fn test_reject_opaque_destination_allows_alpha_formats() {
        assert!(reject_opaque_destination("a/b.png").is_ok());
        assert!(reject_opaque_destination("a/b.webp").is_ok());
    }

    #[test]
    fn test_reject_opaque_replace_target_allows_only_png_and_webp() {
        assert!(reject_opaque_replace_target("a/b.png").is_ok());
        assert!(reject_opaque_replace_target("a/b.WEBP").is_ok());
        assert!(reject_opaque_replace_target("a/b.jpg").is_err());
        assert!(reject_opaque_replace_target("a/b.ico").is_err());
    }
}
