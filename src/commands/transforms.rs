use crate::{OutputMode, SaveMode, io::save_transformed_image};
use inquire::{CustomType, validator::Validation};
use std::io::{IsTerminal, stdin};

use super::start_spinner;

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
        Some(axis) => axis,
        None => prompt_flip_axis()?,
    };
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing flip...")
    };

    let result: Result<Option<(String, &str)>, String> = (|| {
        let img = image::open(path)
            .map_err(|e| format!("flip: failed to open image '{path}': {e}"))?;
        let (flipped, suffix, axis_label) = match axis {
            FlipAxis::Horizontal => (img.fliph(), "fliph", "horizontally"),
            FlipAxis::Vertical => (img.flipv(), "flipv", "vertically"),
        };

        let save_mode = match output {
            OutputMode::Preview => {
                crate::commands::view::display_image(flipped)?;
                return Ok(None);
            }
            OutputMode::Generated => SaveMode::Generated(suffix),
            OutputMode::Explicit(p) => SaveMode::Explicit(p),
            OutputMode::Replace(t) => SaveMode::Replace(t),
        };

        let output_path = save_transformed_image(flipped, path, save_mode, suffix)?;
        Ok(Some((output_path, axis_label)))
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some((output_path, axis_label)) = result? {
        println!("Saved {axis_label} flipped image to {}", output_path);
    }
    Ok(())
}

fn prompt_flip_axis() -> Result<FlipAxis, String> {
    if !stdin().is_terminal() {
        return prompt_flip_axis_non_tty();
    }

    let mode = CustomType::<u8>::new("Choose flip direction:\n (1) Horizontal\n (2) Vertical\n")
        .with_error_message("Please enter 1 or 2")
        .with_validator(|value: &u8| {
            if matches!(*value, 1..=2) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Enter 1 or 2".into()))
            }
        })
        .prompt()
        .map_err(|e| format!("failed to read flip direction: {e}"))?;

    match mode {
        1 => Ok(FlipAxis::Horizontal),
        2 => Ok(FlipAxis::Vertical),
        _ => Err("invalid flip direction: use 1 (horizontal) or 2 (vertical)".to_string()),
    }
}

fn prompt_flip_axis_non_tty() -> Result<FlipAxis, String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read flip direction from stdin: {e}"))?;

    match input.trim() {
        "1" => Ok(FlipAxis::Horizontal),
        "2" => Ok(FlipAxis::Vertical),
        other => Err(format!(
            "invalid flip direction '{other}': use 1 (horizontal) or 2 (vertical)"
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
        None => prompt_rotate_degrees()?,
    };
    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing rotation...")
    };

    let result: Result<Option<String>, String> = (|| {
        let img = image::open(path)
            .map_err(|e| format!("rotate: failed to open image '{path}': {e}"))?;
        let rotated = match deg {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => return Err(format!("rotate: invalid rotation '{deg}': use 90, 180, or 270")),
        };

        let rotate_suffix = format!("rotate{deg}");
        let save_mode = match output {
            OutputMode::Preview => {
                crate::commands::view::display_image(rotated)?;
                return Ok(None);
            }
            OutputMode::Generated => SaveMode::Generated(rotate_suffix.as_str()),
            OutputMode::Explicit(p) => SaveMode::Explicit(p),
            OutputMode::Replace(t) => SaveMode::Replace(t),
        };

        let output_path = save_transformed_image(rotated, path, save_mode, &rotate_suffix)?;
        Ok(Some(output_path))
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some(output_path) = result? {
        println!("Saved rotated image to {}", output_path);
    }
    Ok(())
}

fn prompt_rotate_degrees() -> Result<u16, String> {
    if !stdin().is_terminal() {
        return prompt_rotate_degrees_non_tty();
    }

    let deg = CustomType::<u16>::new(
        "Choose rotation:\n (1) 90 degrees\n (2) 180 degrees\n (3) 270 degrees\n",
    )
    .with_error_message("Please enter 1, 2, or 3")
    .with_validator(|value: &u16| {
        if matches!(*value, 1..=3) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Enter 1, 2, or 3".into()))
        }
    })
    .prompt()
    .map_err(|e| format!("failed to read rotation: {e}"))?;

    match deg {
        1 => Ok(90),
        2 => Ok(180),
        3 => Ok(270),
        _ => Err("invalid rotation selection: use 1, 2, or 3".to_string()),
    }
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
    scale: Option<f32>,
) -> Result<(), String> {
    let (w, h) = if let Some(s) = scale {
        let (orig_w, orig_h) = image::image_dimensions(path)
            .map_err(|e| format!("resize: failed to read image '{path}': {e}"))?;
        let w = (orig_w as f64 * s as f64).round() as u32;
        let h = (orig_h as f64 * s as f64).round() as u32;
        (w.max(1), h.max(1))
    } else {
        match (width, height) {
            (Some(w), Some(h)) => (w, h),
            (None, None) => prompt_resize_dimensions()?,
            (partial_w, partial_h) => {
                let (orig_w, orig_h) = image::image_dimensions(path)
                    .map_err(|e| format!("resize: failed to read image '{path}': {e}"))?;
                resolve_partial_resize(partial_w, partial_h, orig_w, orig_h)?
            }
        }
    };

    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing resize...")
    };

    let result: Result<Option<String>, String> = (|| {
        let img = image::open(path)
            .map_err(|e| format!("resize: failed to open image '{path}': {e}"))?;
        let resized = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);

        let resize_suffix = format!("resize{w}x{h}");
        let save_mode = match output {
            OutputMode::Preview => {
                crate::commands::view::display_image(resized)?;
                return Ok(None);
            }
            OutputMode::Generated => SaveMode::Generated(resize_suffix.as_str()),
            OutputMode::Explicit(p) => SaveMode::Explicit(p),
            OutputMode::Replace(t) => SaveMode::Replace(t),
        };

        let output_path = save_transformed_image(resized, path, save_mode, &resize_suffix)?;
        Ok(Some(output_path))
    })();

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if let Some(output_path) = result? {
        println!("Saved resized image to {}", output_path);
    }
    Ok(())
}

fn resolve_partial_resize(
    width: Option<u32>,
    height: Option<u32>,
    orig_w: u32,
    orig_h: u32,
) -> Result<(u32, u32), String> {
    let (given_label, stretch_label) = match (width, height) {
        (Some(_), None) => ("width", "Stretch horizontally"),
        (None, Some(_)) => ("height", "Stretch vertically"),
        _ => unreachable!(),
    };

    let prompt = format!(
        "Only {given_label} provided:\n (1) Preserve aspect ratio\n (2) {stretch_label}\n"
    );
    let mode = prompt_resize_mode(&prompt)?;

    match (mode, width, height) {
        (ResizeMode::Preserve, Some(w), None) => {
            let h = (orig_h as f64 * w as f64 / orig_w as f64).round() as u32;
            Ok((w, h.max(1)))
        }
        (ResizeMode::Preserve, None, Some(h)) => {
            let w = (orig_w as f64 * h as f64 / orig_h as f64).round() as u32;
            Ok((w.max(1), h))
        }
        (ResizeMode::Stretch, Some(w), None) => Ok((w, orig_h)),
        (ResizeMode::Stretch, None, Some(h)) => Ok((orig_w, h)),
        _ => unreachable!(),
    }
}

enum ResizeMode {
    Preserve,
    Stretch,
}

fn prompt_resize_mode(message: &str) -> Result<ResizeMode, String> {
    if !stdin().is_terminal() {
        return prompt_resize_mode_non_tty();
    }

    let mode = CustomType::<u8>::new(message)
        .with_error_message("Please enter 1 or 2")
        .with_validator(|value: &u8| {
            if matches!(*value, 1..=2) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Enter 1 or 2".into()))
            }
        })
        .prompt()
        .map_err(|e| format!("failed to read resize mode: {e}"))?;

    match mode {
        1 => Ok(ResizeMode::Preserve),
        2 => Ok(ResizeMode::Stretch),
        _ => Err("invalid resize mode: use 1 or 2".to_string()),
    }
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

fn prompt_resize_dimensions() -> Result<(u32, u32), String> {
    if !stdin().is_terminal() {
        return prompt_resize_dimensions_non_tty();
    }

    let width = CustomType::<u32>::new("Enter new width in pixels:")
        .with_error_message("Please enter a positive integer")
        .with_validator(|value: &u32| {
            if *value > 0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Width must be greater than 0".into()))
            }
        })
        .prompt()
        .map_err(|e| format!("failed to read width: {e}"))?;

    let height = CustomType::<u32>::new("Enter new height in pixels:")
        .with_error_message("Please enter a positive integer")
        .with_validator(|value: &u32| {
            if *value > 0 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Height must be greater than 0".into()))
            }
        })
        .prompt()
        .map_err(|e| format!("failed to read height: {e}"))?;

    Ok((width, height))
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
    let img = image::open(path)
        .map_err(|e| format!("invert: failed to open image '{path}': {e}"))?;
    let inverted = invert_colors(img);
    let save_mode = match output {
        OutputMode::Preview => return crate::commands::view::display_image(inverted),
        OutputMode::Generated => SaveMode::Generated("invert"),
        OutputMode::Explicit(p) => SaveMode::Explicit(p),
        OutputMode::Replace(t) => SaveMode::Replace(t),
    };
    let output_path = save_transformed_image(inverted, path, save_mode, "invert")?;
    println!("Saved inverted image to {}", output_path);
    Ok(())
}

pub(crate) fn run_grayscale(path: &str, output: OutputMode) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("grayscale: failed to open image '{path}': {e}"))?;
    let grayscale = img.grayscale();
    let save_mode = match output {
        OutputMode::Preview => return crate::commands::view::display_image(grayscale),
        OutputMode::Generated => SaveMode::Generated("grayscale"),
        OutputMode::Explicit(p) => SaveMode::Explicit(p),
        OutputMode::Replace(t) => SaveMode::Replace(t),
    };
    let output_path = save_transformed_image(grayscale, path, save_mode, "grayscale")?;
    println!("Saved grayscale image to {}", output_path);
    Ok(())
}

pub(crate) fn run_binarize(path: &str, output: OutputMode, threshold: u8) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("binarize: failed to open image '{path}': {e}"))?;
    let binarized = binarize_image(img, threshold);
    let save_mode = match output {
        OutputMode::Preview => return crate::commands::view::display_image(binarized),
        OutputMode::Generated => SaveMode::Generated("binarize"),
        OutputMode::Explicit(p) => SaveMode::Explicit(p),
        OutputMode::Replace(t) => SaveMode::Replace(t),
    };
    let output_path = save_transformed_image(binarized, path, save_mode, "binarize")?;
    println!("Saved binarized image to {}", output_path);
    Ok(())
}

pub(crate) fn binarize_image(img: image::DynamicImage, threshold: u8) -> image::DynamicImage {
    let gray = img.to_luma8();
    let mut rgba = img.to_rgba8();
    for (i, pixel) in rgba.pixels_mut().enumerate() {
        let luma = gray.as_raw()[i];
        let bw = if luma > threshold { 255 } else { 0 };
        pixel[0] = bw;
        pixel[1] = bw;
        pixel[2] = bw;
    }
    image::DynamicImage::ImageRgba8(rgba)
}

pub(crate) fn invert_colors(img: image::DynamicImage) -> image::DynamicImage {
    let mut rgba_image = img.to_rgba8();

    for pixel in rgba_image.pixels_mut() {
        pixel[0] = 255 - pixel[0];
        pixel[1] = 255 - pixel[1];
        pixel[2] = 255 - pixel[2];
    }

    image::DynamicImage::ImageRgba8(rgba_image)
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
}
