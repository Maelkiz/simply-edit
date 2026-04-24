use crate::{OutputMode, io::save_transformed_image};
use inquire::{CustomType, validator::Validation};
use palette::Srgba;
use std::io::{IsTerminal, stdin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlipAxis {
    Horizontal,
    Vertical,
}

pub(crate) fn run_flip(
    path: &str,
    output: OutputMode<'_>,
    axis: Option<FlipAxis>,
) -> Result<(), String> {
    let axis = match axis {
        Some(axis) => axis,
        None => prompt_flip_axis()?,
    };
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let (flipped, suffix, axis_label) = match axis {
        FlipAxis::Horizontal => (img.fliph(), "fliph", "horizontally"),
        FlipAxis::Vertical => (img.flipv(), "flipv", "vertically"),
    };

    let selected_output = match output {
        OutputMode::Generated(_) => OutputMode::Generated(suffix),
        OutputMode::Explicit(path) => OutputMode::Explicit(path),
        OutputMode::Replace => OutputMode::Replace,
    };

    let output_path = save_transformed_image(flipped, path, selected_output, suffix)?;
    println!("Saved {axis_label} flipped image to {}", output_path);
    Ok(())
}

fn prompt_flip_axis() -> Result<FlipAxis, String> {
    if !stdin().is_terminal() {
        return prompt_flip_axis_non_tty();
    }

    let mode = CustomType::<u8>::new(
        "Choose flip direction:\n (1) Horizontal\n (2) Vertical\n",
    )
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

pub(crate) fn run_rotate(degrees: &str, path: &str, output: OutputMode<'_>) -> Result<(), String> {
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

pub(crate) fn run_invert(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let inverted = invert_colors(img);
    let output_path = save_transformed_image(inverted, path, output, "invert")?;
    println!("Saved inverted image to {}", output_path);
    Ok(())
}

pub(crate) fn run_grayscale(path: &str, output: OutputMode<'_>) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("failed to open image '{path}': {e}"))?;
    let grayscale = img.grayscale();
    let output_path = save_transformed_image(grayscale, path, output, "grayscale")?;
    println!("Saved grayscale image to {}", output_path);
    Ok(())
}

pub(crate) fn invert_colors(img: image::DynamicImage) -> image::DynamicImage {
    let mut rgba_image = img.to_rgba8();

    for pixel in rgba_image.pixels_mut() {
        let color = Srgba::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
            pixel[3] as f32 / 255.0,
        );

        let inverted = Srgba::new(
            1.0 - color.red,
            1.0 - color.green,
            1.0 - color.blue,
            color.alpha,
        );

        *pixel = image::Rgba([
            (inverted.red * 255.0).round() as u8,
            (inverted.green * 255.0).round() as u8,
            (inverted.blue * 255.0).round() as u8,
            (inverted.alpha * 255.0).round() as u8,
        ]);
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
}
