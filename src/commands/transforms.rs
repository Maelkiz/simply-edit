use crate::{OutputMode, SaveMode, io::save_transformed_image};
use cliclack::{input, select};
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
        OutputMode::Generated => {
            let p = save_transformed_image(img, source, SaveMode::Generated(suffix.to_string()), suffix)?;
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
        None => prompt_flip_axis()?,
    };

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
        let img = image::open(path)
            .map_err(|e| format!("flip: failed to open image '{path}': {e}"))?;
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

fn prompt_flip_axis() -> Result<FlipAxis, String> {
    if !stdin().is_terminal() {
        return prompt_flip_axis_non_tty();
    }

    select("Choose flip axis:")
        .item(FlipAxis::Vertical, "X axis (vertical, top to bottom)", "")
        .item(FlipAxis::Horizontal, "Y axis (horizontal, left to right)", "")
        .interact()
        .map_err(|e| format!("failed to read flip axis: {e}"))
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

fn prompt_rotate_degrees() -> Result<u16, String> {
    if !stdin().is_terminal() {
        return prompt_rotate_degrees_non_tty();
    }

    select("Choose rotation:")
        .item(90u16, "90 degrees", "")
        .item(180u16, "180 degrees", "")
        .item(270u16, "270 degrees", "")
        .interact()
        .map_err(|e| format!("failed to read rotation: {e}"))
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
        (None, None) => Some(prompt_resize_dimensions()?),
        _ => None,
    };

    let spinner = if matches!(output, OutputMode::Preview) {
        None
    } else {
        start_spinner("Processing resize...")
    };

    let result: Result<(), String> = (|| {
        let img = image::open(path)
            .map_err(|e| format!("resize: failed to open image '{path}': {e}"))?;
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
        let img = image::open(path)
            .map_err(|e| format!("scale: failed to open image '{path}': {e}"))?;
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
        .validate(|s: &String| {
            match s.parse::<f32>() {
                Err(_) => Err("Please enter a positive number"),
                Ok(v) if v <= 0.0 || !v.is_finite() => Err("Scale factor must be greater than 0"),
                Ok(_) => Ok(()),
            }
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
    let v: f32 = buf
        .trim()
        .parse()
        .map_err(|_| format!("invalid scale factor '{}': use a positive number", buf.trim()))?;
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
    let (given_label, stretch_label) = match (width, height) {
        (Some(_), None) => ("width", "Stretch horizontally"),
        (None, Some(_)) => ("height", "Stretch vertically"),
        _ => unreachable!(),
    };

    let mode = prompt_resize_mode(&format!("Only {given_label} provided:"), stretch_label)?;

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

fn prompt_resize_mode(title: &str, stretch_label: &str) -> Result<ResizeMode, String> {
    if !stdin().is_terminal() {
        return prompt_resize_mode_non_tty();
    }

    let choice = select(title)
        .item("preserve", "Preserve aspect ratio", "")
        .item("stretch", stretch_label, "")
        .interact()
        .map_err(|e| format!("failed to read resize mode: {e}"))?;

    match choice {
        "preserve" => Ok(ResizeMode::Preserve),
        "stretch" => Ok(ResizeMode::Stretch),
        _ => unreachable!(),
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

    let width_str: String = input("Enter new width in pixels:")
        .validate(|s: &String| match s.parse::<u32>() {
            Err(_) => Err("Please enter a positive integer"),
            Ok(0) => Err("Width must be greater than 0"),
            Ok(_) => Ok(()),
        })
        .interact()
        .map_err(|e| format!("failed to read width: {e}"))?;
    let width: u32 = width_str.parse().unwrap();

    let height_str: String = input("Enter new height in pixels:")
        .validate(|s: &String| match s.parse::<u32>() {
            Err(_) => Err("Please enter a positive integer"),
            Ok(0) => Err("Height must be greater than 0"),
            Ok(_) => Ok(()),
        })
        .interact()
        .map_err(|e| format!("failed to read height: {e}"))?;
    let height: u32 = height_str.parse().unwrap();

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
    if let Some(output_path) = dispatch_save(inverted, path, output, "invert")? {
        println!("Saved inverted image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_grayscale(path: &str, output: OutputMode) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("grayscale: failed to open image '{path}': {e}"))?;
    let grayscale = img.grayscale();
    if let Some(output_path) = dispatch_save(grayscale, path, output, "grayscale")? {
        println!("Saved grayscale image to {output_path}");
    }
    Ok(())
}

pub(crate) fn run_binarize(path: &str, output: OutputMode, threshold: u8) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("binarize: failed to open image '{path}': {e}"))?;
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
    let img = image::open(path)
        .map_err(|e| format!("pad: failed to open image '{path}': {e}"))?;
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
    use crate::preview::LivePreview;
    use crossterm::event::KeyCode;

    // Open up front so a missing/unreadable file fails immediately on all code paths.
    let img = image::open(path)
        .map_err(|e| format!("binarize: failed to open image '{path}': {e}"))?;

    if !stdin().is_terminal() {
        let threshold = prompt_binarize_threshold_stdin()?;
        return save_binarized(img, path, output, threshold);
    }
    if !crate::commands::view::detect_kitty_support() {
        let threshold = prompt_binarize_threshold_cliclack()?;
        return save_binarized(img, path, output, threshold);
    }

    let mut preview = LivePreview::new(&img)?;
    // None = no value typed yet (show original); Some(t) = apply binarize with t.
    let mut threshold: Option<u8> = None;
    let mut typed = String::new();

    preview.render(&preview.scaled)?;
    crate::preview::print_prompt("Enter threshold (0\u{2013}255)", &typed)?;

    loop {
        if LivePreview::needs_resize() {
            preview.handle_resize(&img);
            let binarized = threshold.map(|t| binarize_rgba(&preview.scaled, t));
            preview.render(binarized.as_ref().unwrap_or(&preview.scaled))?;
            crate::preview::print_prompt("Enter threshold (0\u{2013}255)", &typed)?;
        }

        let key = LivePreview::read_key()?;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                preview.finish()?;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                preview.finish()?;
                return Ok(());
            }
            KeyCode::Enter if !typed.is_empty() => break,
            KeyCode::Enter => continue,
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if typed.len() >= 3 {
                    continue;
                }
                let candidate = format!("{typed}{c}");
                // Only accept digits that keep the value within 0–255.
                if candidate.parse::<u16>().map(|t| t <= 255).unwrap_or(false) {
                    typed = candidate;
                    threshold = Some(typed.parse::<u8>().unwrap());
                } else {
                    continue;
                }
            }
            KeyCode::Backspace => {
                if typed.pop().is_none() {
                    continue;
                }
                threshold = if typed.is_empty() {
                    None
                } else {
                    Some(typed.parse::<u8>().unwrap_or(128))
                };
            }
            _ => continue,
        }

        let binarized = threshold.map(|t| binarize_rgba(&preview.scaled, t));
        preview.render(binarized.as_ref().unwrap_or(&preview.scaled))?;
        crate::preview::print_prompt("Enter threshold (0\u{2013}255)", &typed)?;
    }

    preview.finish()?;
    save_binarized(img, path, output, threshold.unwrap_or(128))
}

pub(crate) fn interactive_rotate(path: &str, output: OutputMode) -> Result<(), String> {
    use crate::preview::LivePreview;
    use crossterm::event::{KeyCode, KeyModifiers};

    const DEGREES: [u16; 3] = [90, 180, 270];
    const LABELS: [&str; 3] = ["90 degrees", "180 degrees", "270 degrees"];

    if !stdin().is_terminal() {
        return run_rotate(path, output, None);
    }
    if !crate::commands::view::detect_kitty_support() {
        return run_rotate(path, output, None);
    }

    let img = image::open(path)
        .map_err(|e| format!("rotate: failed to open image '{path}': {e}"))?;

    let mut preview = LivePreview::new(&img)?;
    let mut cursor = 0usize;

    let frame = make_rotate_frame(&preview, DEGREES[cursor]);
    preview.render(&frame)?;
    crate::preview::print_select_prompt("Choose rotation:", &LABELS, cursor)?;

    loop {
        if LivePreview::needs_resize() {
            preview.handle_resize(&img);
            let frame = make_rotate_frame(&preview, DEGREES[cursor]);
            preview.render(&frame)?;
            crate::preview::print_select_prompt("Choose rotation:", &LABELS, cursor)?;
        }

        let key = LivePreview::read_key()?;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                preview.finish()?;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                preview.finish()?;
                return Ok(());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if cursor == 0 {
                    continue;
                }
                cursor -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if cursor == DEGREES.len() - 1 {
                    continue;
                }
                cursor += 1;
            }
            KeyCode::Enter => break,
            _ => continue,
        }

        let frame = make_rotate_frame(&preview, DEGREES[cursor]);
        preview.render(&frame)?;
        crate::preview::print_select_prompt("Choose rotation:", &LABELS, cursor)?;
    }

    preview.finish()?;
    save_rotated(img, path, output, DEGREES[cursor])
}

/// Rotate `preview.content` (unpadded) by `degrees`, then re-fit to the terminal budget.
/// Works on the small pre-scaled image so it's fast; re-fitting handles the changed aspect ratio.
fn make_rotate_frame(
    preview: &crate::preview::LivePreview,
    degrees: u16,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let dyn_img = image::DynamicImage::ImageRgba8(preview.content.clone());
    let rotated = match degrees {
        90 => dyn_img.rotate90(),
        180 => dyn_img.rotate180(),
        270 => dyn_img.rotate270(),
        _ => dyn_img,
    };
    preview.fit_for_render(&rotated)
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
        _ => return Err(format!("rotate: invalid rotation '{degrees}': use 90, 180, or 270")),
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
    buf.trim()
        .parse()
        .map_err(|_| format!("binarize: invalid threshold '{}': expected 0-255", buf.trim()))
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

fn prompt_binarize_threshold_cliclack() -> Result<u8, String> {
    let s: String = input("Enter threshold (0-255):")
        .validate(|s: &String| match s.parse::<u16>() {
            Err(_) => Err("Please enter a number between 0 and 255"),
            Ok(n) if n > 255 => Err("Threshold must be between 0 and 255"),
            Ok(_) => Ok(()),
        })
        .interact()
        .map_err(|e| format!("binarize: failed to read threshold: {e}"))?;
    Ok(s.parse::<u8>().unwrap())
}

/// Binarize a raw RGBA buffer directly without going through DynamicImage.
/// Uses the same Rec. 601 luma coefficients (77/150/29 >> 8) as the image crate's to_luma8.
fn binarize_rgba(
    src: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    threshold: u8,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let mut out = src.clone();
    for pixel in out.pixels_mut() {
        let luma =
            ((pixel[0] as u32 * 77 + pixel[1] as u32 * 150 + pixel[2] as u32 * 29) >> 8) as u8;
        let bw = if luma > threshold { 255u8 } else { 0u8 };
        pixel[0] = bw;
        pixel[1] = bw;
        pixel[2] = bw;
    }
    out
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
        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(4, 3, image::Rgba([255, 0, 0, 255])),
        );
        let result = pad_image(img, 2, 3, 4, 5, image::Rgba([0, 0, 0, 0])).unwrap();
        assert_eq!(result.width(), 4 + 5 + 3);
        assert_eq!(result.height(), 3 + 2 + 4);
    }

    #[test]
    fn test_pad_zero_padding_same_size() {
        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(5, 5, image::Rgba([100, 100, 100, 255])),
        );
        let result = pad_image(img, 0, 0, 0, 0, image::Rgba([0, 0, 0, 0])).unwrap();
        assert_eq!(result.width(), 5);
        assert_eq!(result.height(), 5);
    }

    #[test]
    fn test_pad_fill_color_in_padding_region() {
        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 0, 0, 255])),
        );
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
        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(1, 1, image::Rgba([200, 100, 50, 255])),
        );
        let result = pad_image(img, 3, 0, 0, 5, image::Rgba([0, 0, 0, 0])).unwrap();
        // Original image placed at (left=5, top=3)
        assert_eq!(result.to_rgba8().get_pixel(5, 3).0, [200, 100, 50, 255]);
    }
}
