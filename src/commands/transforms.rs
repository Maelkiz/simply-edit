use crate::{OutputMode, SaveMode, io::save_transformed_image};
use std::io::{IsTerminal, stdin};

use super::start_spinner;

fn dispatch_save(
    img: image::DynamicImage,
    source: &str,
    output: OutputMode,
    suffix: &str,
) -> Result<Option<String>, String> {
    match output {
        OutputMode::Preview => {
            crate::commands::view::display_image(img)?;
            Ok(None)
        }
        OutputMode::Generated(dir) => {
            let p = save_transformed_image(
                img,
                source,
                SaveMode::Generated(suffix.to_string(), dir),
                suffix,
            )?;
            Ok(Some(p))
        }
        OutputMode::Explicit(p) => {
            let p = save_transformed_image(img, source, SaveMode::Explicit(p), suffix)?;
            Ok(Some(p))
        }
        OutputMode::Replace(t) => {
            let p = save_transformed_image(img, source, SaveMode::Replace(t), suffix)?;
            Ok(Some(p))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlipAxis {
    Horizontal,
    Vertical,
}

pub(crate) fn run_flip(
    path: &str,
    output: OutputMode,
    axis: Option<FlipAxis>,
) -> Result<(), String> {
    let axis = match axis {
        Some(a) => a,
        None if !stdin().is_terminal() => prompt_flip_axis_non_tty()?,
        None => {
            let img = open_for_flip(path)?;
            return match crate::tui::run_screen(
                path,
                &img,
                crate::tui::screens::FlipScreen::default(),
            )? {
                Some(screen) => save_flipped(Some(img), path, output, screen.direction()),
                None => {
                    crate::tui::print_cancelled();
                    Ok(())
                }
            };
        }
    };
    save_flipped(None, path, output, axis)
}

fn open_for_flip(path: &str) -> Result<image::DynamicImage, String> {
    image::open(path).map_err(|e| format!("flip: failed to open image '{path}': {e}"))
}

/// Flip and save, opening `path` under the spinner unless the image is already loaded.
fn save_flipped(
    img: Option<image::DynamicImage>,
    path: &str,
    output: OutputMode,
    axis: FlipAxis,
) -> Result<(), String> {
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing flip...")
    };

    let (axis_label, suffix) = match axis {
        FlipAxis::Horizontal => ("horizontally", "fliph"),
        FlipAxis::Vertical => ("vertically", "flipv"),
    };

    let result: Result<Option<String>, String> = (|| {
        let img = match img {
            Some(img) => img,
            None => open_for_flip(path)?,
        };
        let flipped = match axis {
            FlipAxis::Horizontal => img.fliph(),
            FlipAxis::Vertical => img.flipv(),
        };
        dispatch_save(flipped, path, output, suffix)
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some(output_path) = result? {
        println!("Saved {axis_label} flipped image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_flip_both(path: &str, output: OutputMode) -> Result<(), String> {
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing flip...")
    };

    let result: Result<Option<String>, String> = (|| {
        let img =
            image::open(path).map_err(|e| format!("flip: failed to open image '{path}': {e}"))?;
        let flipped = img.flipv().fliph();
        dispatch_save(flipped, path, output, "flipxy")
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some(output_path) = result? {
        println!("Saved both-axis flipped image to {output_path}");
    }
    Ok(())
}

fn prompt_flip_axis_non_tty() -> Result<FlipAxis, String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read flip axis from stdin: {e}"))?;

    match input.trim() {
        "1" => Ok(FlipAxis::Vertical),
        "2" => Ok(FlipAxis::Horizontal),
        other => Err(format!(
            "invalid flip axis '{other}': use 1 (X axis, vertical) or 2 (Y axis, horizontal)"
        )),
    }
}

pub(crate) fn run_rotate(
    path: &str,
    output: OutputMode,
    degrees: Option<u16>,
) -> Result<(), String> {
    let deg = match degrees {
        Some(deg) => deg,
        None => prompt_rotate_degrees_non_tty()?,
    };
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing rotation...")
    };

    let result: Result<Option<String>, String> = (|| {
        let img =
            image::open(path).map_err(|e| format!("rotate: failed to open image '{path}': {e}"))?;
        let rotated = match deg {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => {
                return Err(format!(
                    "rotate: invalid rotation '{deg}': use 90, 180, or 270"
                ));
            }
        };
        let suffix = format!("rotate{deg}");
        dispatch_save(rotated, path, output, &suffix)
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some(output_path) = result? {
        println!("Saved rotated image to {output_path}");
    }
    Ok(())
}

fn prompt_rotate_degrees_non_tty() -> Result<u16, String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read rotation from stdin: {e}"))?;

    match input.trim() {
        "1" => Ok(90),
        "2" => Ok(180),
        "3" => Ok(270),
        other => Err(format!(
            "invalid rotation '{other}': use 1 (90deg), 2 (180deg), or 3 (270deg)"
        )),
    }
}

pub(crate) fn run_resize(
    path: &str,
    output: OutputMode,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<(), String> {
    let known_dims = match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ if stdin().is_terminal() => return interactive_resize(path, output, width, height),
        (None, None) => Some(prompt_resize_dimensions_non_tty()?),
        _ => None,
    };

    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing resize...")
    };

    let result: Result<(), String> = (|| {
        let img =
            image::open(path).map_err(|e| format!("resize: failed to open image '{path}': {e}"))?;
        let (w, h) = match known_dims {
            Some(dims) => dims,
            None => resolve_partial_resize(width, height, img.width(), img.height())?,
        };
        save_resize(img, path, output, w, h)
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    result
}

/// Ask for the size in the TUI, prefilled with whichever side the command line gave.
fn interactive_resize(
    path: &str,
    output: OutputMode,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<(), String> {
    let img =
        image::open(path).map_err(|e| format!("resize: failed to open image '{path}': {e}"))?;
    let screen = crate::tui::screens::ResizeScreen::new((img.width(), img.height()), width, height);
    let Some(screen) = crate::tui::run_screen(path, &img, screen)? else {
        crate::tui::print_cancelled();
        return Ok(());
    };
    let (w, h) = screen
        .dims()
        .expect("the screen only confirms a valid size");
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing resize...")
    };
    let result = save_resize(img, path, output, w, h);
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }
    result
}

fn save_resize(
    img: image::DynamicImage,
    path: &str,
    output: OutputMode,
    w: u32,
    h: u32,
) -> Result<(), String> {
    let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    let suffix = format!("resize{w}x{h}");
    if let Some(output_path) = dispatch_save(resized, path, output, &suffix)? {
        println!("Saved resized image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_scale(
    path: &str,
    output: OutputMode,
    x_factor: f32,
    y_factor: f32,
) -> Result<(), String> {
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing scale...")
    };

    let result: Result<(), String> = (|| {
        let img =
            image::open(path).map_err(|e| format!("scale: failed to open image '{path}': {e}"))?;
        let w = ((img.width() as f64 * x_factor as f64).round() as u32).max(1);
        let h = ((img.height() as f64 * y_factor as f64).round() as u32).max(1);
        save_resize(img, path, output, w, h)
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    result
}

pub(crate) fn prompt_scale_factor_cliclack() -> Result<f32, String> {
    let s: String = cliclack::input("Enter scale factor (e.g. 0.5 to halve, 2 to double):")
        .validate(|s: &String| match s.parse::<f32>() {
            Err(_) => Err("Please enter a positive number"),
            Ok(v) if v <= 0.0 || !v.is_finite() => Err("Scale factor must be greater than 0"),
            Ok(_) => Ok(()),
        })
        .interact()
        .map_err(|e| format!("failed to read scale factor: {e}"))?;
    Ok(s.parse().unwrap())
}

pub(crate) fn prompt_scale_factor_stdin() -> Result<f32, String> {
    let mut buf = String::new();
    stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("scale: failed to read factor: {e}"))?;
    let v: f32 = buf.trim().parse().map_err(|_| {
        format!(
            "invalid scale factor '{}': use a positive number",
            buf.trim()
        )
    })?;
    if v <= 0.0 || !v.is_finite() {
        return Err("scale factor must be greater than 0".to_string());
    }
    Ok(v)
}

fn resolve_partial_resize(
    width: Option<u32>,
    height: Option<u32>,
    orig_w: u32,
    orig_h: u32,
) -> Result<(u32, u32), String> {
    let mode = prompt_resize_mode_non_tty()?;

    match (mode, width, height) {
        (ResizeMode::Preserve, Some(w), None) => Ok((w, scaled_other_side(w, orig_w, orig_h))),
        (ResizeMode::Preserve, None, Some(h)) => Ok((scaled_other_side(h, orig_h, orig_w), h)),
        (ResizeMode::Stretch, Some(w), None) => Ok((w, orig_h)),
        (ResizeMode::Stretch, None, Some(h)) => Ok((orig_w, h)),
        _ => unreachable!(),
    }
}

/// The other side's length when one side goes from `orig_given` to `given` and the aspect
/// ratio is kept. Never returns 0.
pub(crate) fn scaled_other_side(given: u32, orig_given: u32, orig_other: u32) -> u32 {
    ((orig_other as f64 * given as f64 / orig_given as f64).round() as u32).max(1)
}

enum ResizeMode {
    Preserve,
    Stretch,
}

fn prompt_resize_mode_non_tty() -> Result<ResizeMode, String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read resize mode from stdin: {e}"))?;

    match input.trim() {
        "1" => Ok(ResizeMode::Preserve),
        "2" => Ok(ResizeMode::Stretch),
        other => Err(format!(
            "invalid resize mode '{other}': use 1 (preserve) or 2 (stretch)"
        )),
    }
}

fn prompt_resize_dimensions_non_tty() -> Result<(u32, u32), String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read width from stdin: {e}"))?;
    let width: u32 = input
        .trim()
        .parse()
        .map_err(|_| format!("invalid width '{}': use a positive integer", input.trim()))?;
    if width == 0 {
        return Err("width must be greater than 0".to_string());
    }

    input.clear();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read height from stdin: {e}"))?;
    let height: u32 = input
        .trim()
        .parse()
        .map_err(|_| format!("invalid height '{}': use a positive integer", input.trim()))?;
    if height == 0 {
        return Err("height must be greater than 0".to_string());
    }

    Ok((width, height))
}

pub(crate) fn run_invert(path: &str, output: OutputMode) -> Result<(), String> {
    let img =
        image::open(path).map_err(|e| format!("invert: failed to open image '{path}': {e}"))?;
    let inverted = invert_colors(img);
    if let Some(output_path) = dispatch_save(inverted, path, output, "invert")? {
        println!("Saved inverted image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_grayscale(path: &str, output: OutputMode) -> Result<(), String> {
    let img =
        image::open(path).map_err(|e| format!("grayscale: failed to open image '{path}': {e}"))?;
    let grayscale = img.grayscale();
    if let Some(output_path) = dispatch_save(grayscale, path, output, "grayscale")? {
        println!("Saved grayscale image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_binarize(path: &str, output: OutputMode, threshold: u8) -> Result<(), String> {
    let img =
        image::open(path).map_err(|e| format!("binarize: failed to open image '{path}': {e}"))?;
    let binarized = binarize_image(img, threshold);
    if let Some(output_path) = dispatch_save(binarized, path, output, "binarize")? {
        println!("Saved binarized image to {output_path}");
    }
    Ok(())
}

pub(crate) fn binarize_image(img: image::DynamicImage, threshold: u8) -> image::DynamicImage {
    let mut rgba = img.into_rgba8();
    for pixel in rgba.pixels_mut() {
        let luma =
            ((pixel[0] as u32 * 77 + pixel[1] as u32 * 150 + pixel[2] as u32 * 29) >> 8) as u8;
        let bw = if luma > threshold { 255 } else { 0 };
        pixel[0] = bw;
        pixel[1] = bw;
        pixel[2] = bw;
    }
    image::DynamicImage::ImageRgba8(rgba)
}

pub(crate) fn run_pad(
    path: &str,
    output: OutputMode,
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
    color: image::Rgba<u8>,
) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("pad: failed to open image '{path}': {e}"))?;
    let padded = pad_image(img, top, right, bottom, left, color)?;
    if let Some(output_path) = dispatch_save(padded, path, output, "pad")? {
        println!("Saved padded image to {output_path}");
    }
    Ok(())
}

pub(crate) fn pad_image(
    img: image::DynamicImage,
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
    color: image::Rgba<u8>,
) -> Result<image::DynamicImage, String> {
    let (orig_w, orig_h) = (img.width(), img.height());
    let new_w = orig_w
        .checked_add(left)
        .and_then(|v| v.checked_add(right))
        .ok_or_else(|| "pad: dimensions overflow u32".to_string())?;
    let new_h = orig_h
        .checked_add(top)
        .and_then(|v| v.checked_add(bottom))
        .ok_or_else(|| "pad: dimensions overflow u32".to_string())?;
    let mut canvas = image::ImageBuffer::from_pixel(new_w, new_h, color);
    let img_rgba = img.into_rgba8();
    image::imageops::overlay(&mut canvas, &img_rgba, left as i64, top as i64);
    Ok(image::DynamicImage::ImageRgba8(canvas))
}

pub(crate) fn invert_colors(img: image::DynamicImage) -> image::DynamicImage {
    let mut rgba_image = img.into_rgba8();

    for pixel in rgba_image.pixels_mut() {
        pixel[0] = 255 - pixel[0];
        pixel[1] = 255 - pixel[1];
        pixel[2] = 255 - pixel[2];
    }

    image::DynamicImage::ImageRgba8(rgba_image)
}

pub(crate) fn interactive_binarize(path: &str, output: OutputMode) -> Result<(), String> {
    // Open up front so a missing/unreadable file fails immediately on all code paths.
    let img =
        image::open(path).map_err(|e| format!("binarize: failed to open image '{path}': {e}"))?;

    if !stdin().is_terminal() {
        let threshold = prompt_binarize_threshold_stdin()?;
        return save_binarized(img, path, output, threshold);
    }

    match crate::tui::run_screen(path, &img, crate::tui::screens::BinarizeScreen::default())? {
        Some(screen) => save_binarized(img, path, output, screen.threshold()),
        None => {
            crate::tui::print_cancelled();
            Ok(())
        }
    }
}

pub(crate) fn interactive_rotate(path: &str, output: OutputMode) -> Result<(), String> {
    if !stdin().is_terminal() {
        return run_rotate(path, output, None);
    }

    let img =
        image::open(path).map_err(|e| format!("rotate: failed to open image '{path}': {e}"))?;
    match crate::tui::run_screen(path, &img, crate::tui::screens::RotateScreen::default())? {
        Some(screen) => save_rotated(img, path, output, screen.degrees()),
        None => {
            crate::tui::print_cancelled();
            Ok(())
        }
    }
}

fn save_rotated(
    img: image::DynamicImage,
    path: &str,
    output: OutputMode,
    degrees: u16,
) -> Result<(), String> {
    let rotated = match degrees {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => {
            return Err(format!(
                "rotate: invalid rotation '{degrees}': use 90, 180, or 270"
            ));
        }
    };
    let suffix = format!("rotate{degrees}");
    if let Some(output_path) = dispatch_save(rotated, path, output, &suffix)? {
        println!("Saved rotated image to {output_path}");
    }
    Ok(())
}

fn prompt_binarize_threshold_stdin() -> Result<u8, String> {
    let mut buf = String::new();
    stdin()
        .read_line(&mut buf)
        .map_err(|e| format!("binarize: failed to read threshold: {e}"))?;
    buf.trim().parse().map_err(|_| {
        format!(
            "binarize: invalid threshold '{}': expected 0-255",
            buf.trim()
        )
    })
}

fn save_binarized(
    img: image::DynamicImage,
    path: &str,
    output: OutputMode,
    threshold: u8,
) -> Result<(), String> {
    let binarized = binarize_image(img, threshold);
    if let Some(output_path) = dispatch_save(binarized, path, output, "binarize")? {
        println!("Saved binarized image to {output_path}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_inversion_black_becomes_white() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let inverted = invert_colors(dynamic_img);
        let rgba_img = inverted.to_rgba8();
        let pixel = rgba_img.get_pixel(0, 0);

        assert_eq!(pixel[0], 255);
        assert_eq!(pixel[1], 255);
        assert_eq!(pixel[2], 255);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn test_color_inversion_white_becomes_black() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let inverted = invert_colors(dynamic_img);
        let rgba_img = inverted.to_rgba8();
        let pixel = rgba_img.get_pixel(0, 0);

        assert_eq!(pixel[0], 0);
        assert_eq!(pixel[1], 0);
        assert_eq!(pixel[2], 0);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn test_color_inversion_preserves_alpha() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 0, 0, 128]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let inverted = invert_colors(dynamic_img);
        let rgba_img = inverted.to_rgba8();
        let pixel = rgba_img.get_pixel(0, 0);

        assert_eq!(pixel[3], 128);
    }

    #[test]
    fn test_color_inversion_gray_stays_roughly_gray() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([128, 128, 128, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let inverted = invert_colors(dynamic_img);
        let rgba_img = inverted.to_rgba8();
        let pixel = rgba_img.get_pixel(0, 0);

        assert!(pixel[0] >= 126 && pixel[0] <= 128);
        assert!(pixel[1] >= 126 && pixel[1] <= 128);
        assert!(pixel[2] >= 126 && pixel[2] <= 128);
    }

    #[test]
    fn test_binarize_white_stays_white() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 128);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [255, 255, 255, 255]);
    }

    #[test]
    fn test_binarize_black_stays_black() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 128);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [0, 0, 0, 255]);
    }

    #[test]
    fn test_binarize_dark_gray_becomes_black() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([100, 100, 100, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 128);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [0, 0, 0, 255]);
    }

    #[test]
    fn test_binarize_light_gray_becomes_white() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([200, 200, 200, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 128);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [255, 255, 255, 255]);
    }

    #[test]
    fn test_binarize_preserves_alpha() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([200, 200, 200, 128]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 128);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel[3], 128);
    }

    #[test]
    fn test_binarize_threshold_zero() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([1, 1, 1, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 0);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [255, 255, 255, 255]);
    }

    #[test]
    fn test_binarize_threshold_255_everything_black() {
        let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        let result = binarize_image(dynamic_img, 255);
        let pixel = result.to_rgba8().get_pixel(0, 0).0;

        assert_eq!(pixel, [0, 0, 0, 255]);
    }

    #[test]
    fn test_pad_dimensions_uniform() {
        let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            4,
            3,
            image::Rgba([255, 0, 0, 255]),
        ));
        let result = pad_image(img, 2, 3, 4, 5, image::Rgba([0, 0, 0, 0])).unwrap();
        assert_eq!(result.width(), 4 + 5 + 3);
        assert_eq!(result.height(), 3 + 2 + 4);
    }

    #[test]
    fn test_pad_zero_padding_same_size() {
        let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            5,
            5,
            image::Rgba([100, 100, 100, 255]),
        ));
        let result = pad_image(img, 0, 0, 0, 0, image::Rgba([0, 0, 0, 0])).unwrap();
        assert_eq!(result.width(), 5);
        assert_eq!(result.height(), 5);
    }

    #[test]
    fn test_pad_fill_color_in_padding_region() {
        let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            1,
            1,
            image::Rgba([255, 0, 0, 255]),
        ));
        let fill = image::Rgba([0, 255, 0, 255]);
        let result = pad_image(img, 2, 2, 2, 2, fill).unwrap();
        // Top-left corner is padding
        assert_eq!(result.to_rgba8().get_pixel(0, 0).0, [0, 255, 0, 255]);
        // Bottom-right corner is padding
        let w = result.width() - 1;
        let h = result.height() - 1;
        assert_eq!(result.to_rgba8().get_pixel(w, h).0, [0, 255, 0, 255]);
    }

    #[test]
    fn test_pad_original_preserved_at_offset() {
        let img = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            1,
            1,
            image::Rgba([200, 100, 50, 255]),
        ));
        let result = pad_image(img, 3, 0, 0, 5, image::Rgba([0, 0, 0, 0])).unwrap();
        // Original image placed at (left=5, top=3)
        assert_eq!(result.to_rgba8().get_pixel(5, 3).0, [200, 100, 50, 255]);
    }
}
