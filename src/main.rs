use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use image::GenericImageView;
use palette::Srgba;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use vtracer::Config;

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
        [_, command, rest @ ..] if command == "convert" => convert(rest),
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

fn convert(args: &[String]) -> Result<(), String> {
    let ConvertArgs { options, src, dst } = parse_convert_args(args)?;

    if is_svg_path(&dst) {
        if !options.is_empty() {
            return Err("scale and width/height flags are only supported for SVG to image conversion".to_string());
        }

        return convert_image_to_svg(&src, &dst);
    }

    if is_svg_path(&src) {
        return convert_svg_to_image(&src, &dst, options);
    }

    if !options.is_empty() {
        return Err("scale and width/height flags are only supported for SVG to image conversion".to_string());
    }

    let img = image::open(&src).map_err(|e| format!("failed to open image '{src}': {e}"))?;
    save_image(img, &dst)?;
    println!("Converted image to {}", dst);
    Ok(())
}

#[derive(Default)]
struct ConvertOptions {
    scale: Option<f32>,
    width: Option<u32>,
    height: Option<u32>,
}

impl ConvertOptions {
    fn is_empty(&self) -> bool {
        self.scale.is_none() && self.width.is_none() && self.height.is_none()
    }
}

struct ConvertArgs {
    options: ConvertOptions,
    src: String,
    dst: String,
}

fn parse_convert_args(args: &[String]) -> Result<ConvertArgs, String> {
    let mut options = ConvertOptions::default();
    let mut positionals = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-s" | "--scale" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value for --scale".to_string())?;
                options.scale = Some(parse_positive_f32(value, "--scale")?);
                index += 2;
            }
            "-w" | "--width" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value for --width".to_string())?;
                options.width = Some(parse_positive_u32(value, "--width")?);
                index += 2;
            }
            "-h" | "--height" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value for --height".to_string())?;
                options.height = Some(parse_positive_u32(value, "--height")?);
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unrecognized convert flag '{value}'"));
            }
            value => {
                positionals.push(value.to_string());
                index += 1;
            }
        }
    }

    match positionals.as_slice() {
        [src, dst] => Ok(ConvertArgs { options, src: src.clone(), dst: dst.clone() }),
        _ => Err(usage()),
    }
}

fn parse_positive_f32(value: &str, flag: &str) -> Result<f32, String> {
    let parsed: f32 = value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}: use a positive number"))?;

    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("invalid value '{value}' for {flag}: use a positive number"));
    }

    Ok(parsed)
}

fn parse_positive_u32(value: &str, flag: &str) -> Result<u32, String> {
    let parsed: u32 = value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}: use a positive integer"))?;

    if parsed == 0 {
        return Err(format!("invalid value '{value}' for {flag}: use a positive integer"));
    }

    Ok(parsed)
}

fn convert_image_to_svg(src: &str, dst: &str) -> Result<(), String> {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);
    vtracer::convert_image_to_svg(src_path, dst_path, Config::default()).map_err(|e| {
        format!(
            "failed to vectorize image '{}' to '{}': {e}",
            src_path.display(),
            dst_path.display()
        )
    })?;
    println!("Converted image to {}", dst);
    Ok(())
}

fn convert_svg_to_image(src: &str, dst: &str, options: ConvertOptions) -> Result<(), String> {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);
    let svg_data = fs::read(src_path)
        .map_err(|e| format!("failed to read SVG '{}': {e}", src_path.display()))?;

    let mut usvg_options = Options::default();
    usvg_options.resources_dir = src_path.parent().map(Path::to_path_buf);

    let tree = Tree::from_data(&svg_data, &usvg_options)
        .map_err(|e| format!("failed to parse SVG '{}': {e}", src_path.display()))?;

    let (render_width, render_height, scale_x, scale_y) = compute_render_dimensions(tree.size(), &options)?;
    let mut pixmap = Pixmap::new(render_width, render_height).ok_or_else(|| {
        format!(
            "failed to create pixmap for '{}' with size {}x{}",
            src_path.display(),
            render_width,
            render_height
        )
    })?;

    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(&tree, Transform::from_scale(scale_x, scale_y), &mut pixmap_mut);

    save_rendered_pixmap(pixmap, dst_path)?;
    println!("Converted image to {}", dst);
    Ok(())
}

fn compute_render_dimensions(size: resvg::usvg::Size, options: &ConvertOptions) -> Result<(u32, u32, f32, f32), String> {
    let source_width = size.width();
    let source_height = size.height();

    if let Some(width) = options.width {
        let height = options.height.unwrap_or_else(|| round_scaled_dimension(source_height, width as f32 / source_width));
        let scale_x = width as f32 / source_width;
        let scale_y = height as f32 / source_height;
        return Ok((width, height, scale_x, scale_y));
    }

    if let Some(height) = options.height {
        let width = round_scaled_dimension(source_width, height as f32 / source_height);
        let scale_x = width as f32 / source_width;
        let scale_y = height as f32 / source_height;
        return Ok((width, height, scale_x, scale_y));
    }

    let scale = options.scale.unwrap_or(1.0);
    let scaled_size = size
        .scale_by(scale)
        .ok_or_else(|| format!("invalid SVG scale factor '{scale}'"))?;
    let int_size = scaled_size.to_int_size();
    Ok((int_size.width(), int_size.height(), scale, scale))
}

fn round_scaled_dimension(source: f32, scale: f32) -> u32 {
    (source * scale).round().max(1.0) as u32
}

fn save_rendered_pixmap(pixmap: Pixmap, output_path: &Path) -> Result<(), String> {
    let width = pixmap.width();
    let height = pixmap.height();
    let image = image::RgbaImage::from_raw(width, height, pixmap.take_demultiplied()).ok_or_else(|| {
        format!(
            "failed to build image buffer for '{}' with size {}x{}",
            output_path.display(),
            width,
            height
        )
    })?;

    image::DynamicImage::ImageRgba8(image)
        .save(output_path)
        .map_err(|e| format!("failed to save image '{}': {e}", output_path.display()))
}

fn is_svg_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
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
        "  simple-edit convert [-s|--scale <factor>] [-w|--width <px>] [-h|--height <px>] <path-to-image> <new-path>",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Path generation tests
    mod path_generation {
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
    }

    // Flag parsing tests
    mod flag_parsing {
        use super::*;

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

    // Color inversion tests
    mod color_operations {
        use super::*;

        #[test]
        fn test_color_inversion_black_becomes_white() {
            // Create a simple 1x1 black pixel image
            let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([0, 0, 0, 255]));
            let dynamic_img = image::DynamicImage::ImageRgba8(img);

            let inverted = invert_colors(dynamic_img);
            let rgba_img = inverted.to_rgba8();
            let pixel = rgba_img.get_pixel(0, 0);

            // Black (0,0,0) should invert to white (255,255,255)
            assert_eq!(pixel[0], 255);
            assert_eq!(pixel[1], 255);
            assert_eq!(pixel[2], 255);
            assert_eq!(pixel[3], 255); // Alpha should remain unchanged
        }

        #[test]
        fn test_color_inversion_white_becomes_black() {
            // Create a simple 1x1 white pixel image
            let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
            let dynamic_img = image::DynamicImage::ImageRgba8(img);

            let inverted = invert_colors(dynamic_img);
            let rgba_img = inverted.to_rgba8();
            let pixel = rgba_img.get_pixel(0, 0);

            // White (255,255,255) should invert to black (0,0,0)
            assert_eq!(pixel[0], 0);
            assert_eq!(pixel[1], 0);
            assert_eq!(pixel[2], 0);
            assert_eq!(pixel[3], 255); // Alpha should remain unchanged
        }

        #[test]
        fn test_color_inversion_preserves_alpha() {
            // Create a 1x1 image with semi-transparent red
            let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 0, 0, 128]));
            let dynamic_img = image::DynamicImage::ImageRgba8(img);

            let inverted = invert_colors(dynamic_img);
            let rgba_img = inverted.to_rgba8();
            let pixel = rgba_img.get_pixel(0, 0);

            // Alpha should be preserved
            assert_eq!(pixel[3], 128);
        }

        #[test]
        fn test_color_inversion_gray_stays_roughly_gray() {
            // Create a 1x1 gray pixel (128,128,128)
            let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([128, 128, 128, 255]));
            let dynamic_img = image::DynamicImage::ImageRgba8(img);

            let inverted = invert_colors(dynamic_img);
            let rgba_img = inverted.to_rgba8();
            let pixel = rgba_img.get_pixel(0, 0);

            // Gray (128,128,128) should invert to roughly gray (127,127,127)
            // allowing for rounding
            assert!(pixel[0] >= 126 && pixel[0] <= 128);
            assert!(pixel[1] >= 126 && pixel[1] <= 128);
            assert!(pixel[2] >= 126 && pixel[2] <= 128);
        }
    }

    // Usage text validation
    mod usage {
        use super::*;

        #[test]
        fn test_usage_contains_all_commands() {
            let usage_text = usage();
            assert!(usage_text.contains("fliph"));
            assert!(usage_text.contains("flipv"));
            assert!(usage_text.contains("rotate"));
            assert!(usage_text.contains("invert"));
            assert!(usage_text.contains("grayscale"));
            assert!(usage_text.contains("convert"));
        }

        #[test]
        fn test_usage_contains_replace_flag_info() {
            let usage_text = usage();
            assert!(usage_text.contains("-r|--replace"));
        }

        #[test]
        fn test_usage_is_non_empty() {
            let usage_text = usage();
            assert!(!usage_text.is_empty());
        }
    }

    mod convert {
        use super::*;
        use std::fs;

        #[test]
        fn test_is_svg_path_accepts_svg_extension_case_insensitive() {
            assert!(is_svg_path("output.svg"));
            assert!(is_svg_path("output.SVG"));
        }

        #[test]
        fn test_is_svg_path_rejects_non_svg_extensions() {
            assert!(!is_svg_path("output.png"));
            assert!(!is_svg_path("output"));
        }

        #[test]
        fn test_convert_svg_to_image_creates_png() {
            let temp_root = std::env::temp_dir().join(format!(
                "simply-edit-svg-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system time before unix epoch")
                    .as_nanos()
            ));
            fs::create_dir_all(&temp_root).expect("failed to create temp dir");

            let input_path = temp_root.join("input.svg");
            let output_path = temp_root.join("output.png");
            fs::write(
                &input_path,
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1" viewBox="0 0 1 1">
  <rect width="1" height="1" fill="red"/>
</svg>"#,
            )
            .expect("failed to write svg");

            convert_svg_to_image(
                input_path.to_str().expect("invalid input path"),
                output_path.to_str().expect("invalid output path"),
                ConvertOptions::default(),
            )
            .expect("svg conversion failed");

            let converted = image::open(&output_path).expect("failed to open converted image");
            assert_eq!(converted.width(), 1);
            assert_eq!(converted.height(), 1);

            let _ = fs::remove_file(&input_path);
            let _ = fs::remove_file(&output_path);
            let _ = fs::remove_dir(&temp_root);
        }

        #[test]
        fn test_parse_convert_args_accepts_scale_and_resolution_flags() {
            let args = vec![
                "-s".to_string(),
                "2.5".to_string(),
                "-w".to_string(),
                "200".to_string(),
                "-h".to_string(),
                "100".to_string(),
                "input.svg".to_string(),
                "output.png".to_string(),
            ];

            let parsed = parse_convert_args(&args).expect("failed to parse convert args");
            assert_eq!(parsed.src, "input.svg");
            assert_eq!(parsed.dst, "output.png");
            assert_eq!(parsed.options.scale, Some(2.5));
            assert_eq!(parsed.options.width, Some(200));
            assert_eq!(parsed.options.height, Some(100));
        }

        #[test]
        fn test_compute_render_dimensions_uses_scale_when_no_resolution_is_set() {
            let size = resvg::usvg::Size::from_wh(10.0, 20.0).expect("valid size");
            let options = ConvertOptions { scale: Some(2.0), width: None, height: None };

            let (width, height, scale_x, scale_y) = compute_render_dimensions(size, &options).expect("dimension computation failed");
            assert_eq!((width, height), (20, 40));
            assert_eq!(scale_x, 2.0);
            assert_eq!(scale_y, 2.0);
        }

        #[test]
        fn test_compute_render_dimensions_uses_explicit_width_and_height() {
            let size = resvg::usvg::Size::from_wh(10.0, 20.0).expect("valid size");
            let options = ConvertOptions { scale: Some(5.0), width: Some(80), height: Some(60) };

            let (width, height, scale_x, scale_y) = compute_render_dimensions(size, &options).expect("dimension computation failed");
            assert_eq!((width, height), (80, 60));
            assert_eq!(scale_x, 8.0);
            assert_eq!(scale_y, 3.0);
        }
    }
}
