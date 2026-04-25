use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use vtracer::Config;

use super::start_spinner;

fn fast_vectorize_config() -> Config {
    Config {
        color_precision: 4,
        layer_difference: 48,
        filter_speckle: 8,
        max_iterations: 4,
        ..Config::default()
    }
}

pub(crate) fn run_convert(args: &[String]) -> Result<(), String> {
    let mut positionals = Vec::new();

    for arg in args {
        if arg.starts_with('-') {
            return Err(format!("unrecognized convert flag '{arg}'"));
        }
        positionals.push(arg.as_str());
    }

    let (src, dst) = match positionals.as_slice() {
        [src, dst] => (*src, *dst),
        _ => return Err(crate::usage()),
    };

    let dst = crate::io::enumerate_if_exists(Path::new(dst))
        .to_string_lossy()
        .to_string();

    if is_svg_path(&dst) {
        return vectorize(src, &dst, false);
    }

    if is_svg_path(src) {
        return rasterize(src, &dst, RasterizeOptions::default());
    }

    let img = image::open(src).map_err(|e| format!("failed to open image '{src}': {e}"))?;
    crate::io::save_image(img, &dst)?;
    println!("Converted image to {}", dst);
    Ok(())
}

#[derive(Default)]
pub(crate) struct RasterizeOptions {
    pub(crate) scale: Option<f32>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

pub(crate) struct RasterizeArgs {
    pub(crate) options: RasterizeOptions,
    pub(crate) src: String,
    pub(crate) dst: String,
}

pub(crate) fn parse_rasterize_args(args: &[String]) -> Result<RasterizeArgs, String> {
    let mut options = RasterizeOptions::default();
    let mut positionals = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "-s" | "--scale" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --scale".to_string())?;
                options.scale = Some(parse_positive_f32(value, "--scale")?);
                index += 2;
            }
            "-w" | "--width" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --width".to_string())?;
                options.width = Some(parse_positive_u32(value, "--width")?);
                index += 2;
            }
            "-h" | "--height" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --height".to_string())?;
                options.height = Some(parse_positive_u32(value, "--height")?);
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("unrecognized rasterize flag '{value}'"));
            }
            value => {
                positionals.push(value.to_string());
                index += 1;
            }
        }
    }

    match positionals.as_slice() {
        [src, dst] => Ok(RasterizeArgs {
            options,
            src: src.clone(),
            dst: dst.clone(),
        }),
        [src] => Ok(RasterizeArgs {
            options,
            src: src.clone(),
            dst: replace_extension(src, "png"),
        }),
        _ => Err(crate::usage()),
    }
}

#[derive(Debug)]
pub(crate) struct VectorizeArgs {
    pub(crate) src: String,
    pub(crate) dst: String,
    pub(crate) fast: bool,
}

pub(crate) fn parse_vectorize_args(args: &[String]) -> Result<VectorizeArgs, String> {
    let mut positionals = Vec::new();
    let mut fast = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--fast" => {
                fast = true;
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(format!("unrecognized vectorize flag '{value}'"));
            }
            value => {
                positionals.push(value);
                index += 1;
            }
        }
    }

    match positionals.as_slice() {
        [src, dst] => Ok(VectorizeArgs {
            src: src.to_string(),
            dst: dst.to_string(),
            fast,
        }),
        [src] => Ok(VectorizeArgs {
            src: src.to_string(),
            dst: replace_extension(src, "svg"),
            fast,
        }),
        _ => Err(crate::usage()),
    }
}

pub(crate) fn run_vectorize(args: &[String]) -> Result<(), String> {
    let VectorizeArgs { src, dst, fast } = parse_vectorize_args(args)?;
    let dst = crate::io::enumerate_if_exists(Path::new(&dst))
        .to_string_lossy()
        .to_string();
    vectorize(&src, &dst, fast)
}

pub(crate) fn run_rasterize(args: &[String]) -> Result<(), String> {
    let RasterizeArgs { options, src, dst } = parse_rasterize_args(args)?;
    let dst = crate::io::enumerate_if_exists(Path::new(&dst))
        .to_string_lossy()
        .to_string();
    rasterize(&src, &dst, options)
}

fn replace_extension(path: &str, new_ext: &str) -> String {
    let p = Path::new(path);
    p.with_extension(new_ext).to_string_lossy().to_string()
}

fn parse_positive_f32(value: &str, flag: &str) -> Result<f32, String> {
    let parsed: f32 = value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}: use a positive number"))?;

    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!(
            "invalid value '{value}' for {flag}: use a positive number"
        ));
    }

    Ok(parsed)
}

fn parse_positive_u32(value: &str, flag: &str) -> Result<u32, String> {
    let parsed: u32 = value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}: use a positive integer"))?;

    if parsed == 0 {
        return Err(format!(
            "invalid value '{value}' for {flag}: use a positive integer"
        ));
    }

    Ok(parsed)
}

fn vectorize(src: &str, dst: &str, fast: bool) -> Result<(), String> {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);
    let config = if fast {
        fast_vectorize_config()
    } else {
        Config::default()
    };
    let spinner = start_spinner("Vectorizing image...");

    let result =
        vtracer::convert_image_to_svg(src_path, dst_path, config).map_err(
            |e| {
                format!(
                    "failed to vectorize image '{}' to '{}': {e}",
                    src_path.display(),
                    dst_path.display()
                )
            },
        );

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    result?;
    println!("Converted image to {}", dst);
    Ok(())
}

fn rasterize(src: &str, dst: &str, options: RasterizeOptions) -> Result<(), String> {
    let src_path = Path::new(src);
    let dst_path = Path::new(dst);
    let svg_data = fs::read(src_path)
        .map_err(|e| format!("failed to read SVG '{}': {e}", src_path.display()))?;

    let mut usvg_options = Options::default();
    usvg_options.resources_dir = src_path.parent().map(Path::to_path_buf);

    let tree = Tree::from_data(&svg_data, &usvg_options)
        .map_err(|e| format!("failed to parse SVG '{}': {e}", src_path.display()))?;

    let (render_width, render_height, scale_x, scale_y) =
        compute_render_dimensions(tree.size(), &options)?;
    let mut pixmap = Pixmap::new(render_width, render_height).ok_or_else(|| {
        format!(
            "failed to create pixmap for '{}' with size {}x{}",
            src_path.display(),
            render_width,
            render_height
        )
    })?;

    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(
        &tree,
        Transform::from_scale(scale_x, scale_y),
        &mut pixmap_mut,
    );

    save_rendered_pixmap(pixmap, dst_path)?;
    println!("Converted image to {}", dst);
    Ok(())
}

fn compute_render_dimensions(
    size: resvg::usvg::Size,
    options: &RasterizeOptions,
) -> Result<(u32, u32, f32, f32), String> {
    let source_width = size.width();
    let source_height = size.height();

    if let Some(width) = options.width {
        let height = options
            .height
            .unwrap_or_else(|| round_scaled_dimension(source_height, width as f32 / source_width));
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
    let image =
        image::RgbaImage::from_raw(width, height, pixmap.take_demultiplied()).ok_or_else(|| {
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

pub(crate) fn is_svg_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

#[cfg(test)]
mod tests {
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
    fn test_rasterize_creates_png() {
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

        rasterize(
            input_path.to_str().expect("invalid input path"),
            output_path.to_str().expect("invalid output path"),
            RasterizeOptions::default(),
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
    fn test_convert_png_to_webp() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-webp-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_root).expect("failed to create temp dir");

        let input_path = temp_root.join("input.png");
        let output_path = temp_root.join("output.webp");

        let img = image::ImageBuffer::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        image::DynamicImage::ImageRgba8(img)
            .save(&input_path)
            .expect("failed to save test png");

        run_convert(&[
            input_path.to_str().unwrap().to_string(),
            output_path.to_str().unwrap().to_string(),
        ])
        .expect("png to webp conversion failed");

        assert!(output_path.exists());
        let converted = image::open(&output_path).expect("failed to open converted webp");
        assert_eq!(converted.width(), 2);
        assert_eq!(converted.height(), 2);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_convert_webp_to_png() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-webp-to-png-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_root).expect("failed to create temp dir");

        let input_path = temp_root.join("input.webp");
        let output_path = temp_root.join("output.png");

        let img = image::ImageBuffer::from_pixel(2, 2, image::Rgba([0, 255, 0, 255]));
        image::DynamicImage::ImageRgba8(img)
            .save(&input_path)
            .expect("failed to save test webp");

        run_convert(&[
            input_path.to_str().unwrap().to_string(),
            output_path.to_str().unwrap().to_string(),
        ])
        .expect("webp to png conversion failed");

        assert!(output_path.exists());
        let converted = image::open(&output_path).expect("failed to open converted png");
        assert_eq!(converted.width(), 2);
        assert_eq!(converted.height(), 2);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_convert_svg_to_webp() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-svg-to-webp-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_root).expect("failed to create temp dir");

        let input_path = temp_root.join("input.svg");
        let output_path = temp_root.join("output.webp");
        fs::write(
            &input_path,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4" viewBox="0 0 4 4">
  <rect width="4" height="4" fill="blue"/>
</svg>"#,
        )
        .expect("failed to write svg");

        run_convert(&[
            input_path.to_str().unwrap().to_string(),
            output_path.to_str().unwrap().to_string(),
        ])
        .expect("svg to webp conversion failed");

        assert!(output_path.exists());
        let converted = image::open(&output_path).expect("failed to open converted webp");
        assert_eq!(converted.width(), 4);
        assert_eq!(converted.height(), 4);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_parse_rasterize_args_accepts_scale_and_resolution_flags() {
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

        let parsed = parse_rasterize_args(&args).expect("failed to parse rasterize args");
        assert_eq!(parsed.src, "input.svg");
        assert_eq!(parsed.dst, "output.png");
        assert_eq!(parsed.options.scale, Some(2.5));
        assert_eq!(parsed.options.width, Some(200));
        assert_eq!(parsed.options.height, Some(100));
    }

    #[test]
    fn test_parse_vectorize_args_accepts_positionals() {
        let args = vec!["input.png".to_string(), "output.svg".to_string()];
        let parsed = parse_vectorize_args(&args).expect("failed to parse vectorize args");
        assert_eq!(parsed.src, "input.png");
        assert_eq!(parsed.dst, "output.svg");
        assert!(!parsed.fast);
    }

    #[test]
    fn test_parse_vectorize_args_fast_flag() {
        let args = vec![
            "--fast".to_string(),
            "input.png".to_string(),
            "output.svg".to_string(),
        ];
        let parsed = parse_vectorize_args(&args).expect("failed to parse vectorize args");
        assert_eq!(parsed.src, "input.png");
        assert!(parsed.fast);
    }

    #[test]
    fn test_parse_vectorize_args_rejects_flags() {
        let args = vec![
            "--unknown".to_string(),
            "input.png".to_string(),
            "output.svg".to_string(),
        ];
        let err = parse_vectorize_args(&args).expect_err("expected flag rejection");
        assert!(err.contains("unrecognized vectorize flag"));
    }

    #[test]
    fn test_compute_render_dimensions_uses_scale_when_no_resolution_is_set() {
        let size = resvg::usvg::Size::from_wh(10.0, 20.0).expect("valid size");
        let options = RasterizeOptions {
            scale: Some(2.0),
            width: None,
            height: None,
        };

        let (width, height, scale_x, scale_y) =
            compute_render_dimensions(size, &options).expect("dimension computation failed");
        assert_eq!((width, height), (20, 40));
        assert_eq!(scale_x, 2.0);
        assert_eq!(scale_y, 2.0);
    }

    #[test]
    fn test_compute_render_dimensions_uses_explicit_width_and_height() {
        let size = resvg::usvg::Size::from_wh(10.0, 20.0).expect("valid size");
        let options = RasterizeOptions {
            scale: Some(5.0),
            width: Some(80),
            height: Some(60),
        };

        let (width, height, scale_x, scale_y) =
            compute_render_dimensions(size, &options).expect("dimension computation failed");
        assert_eq!((width, height), (80, 60));
        assert_eq!(scale_x, 8.0);
        assert_eq!(scale_y, 3.0);
    }
}
