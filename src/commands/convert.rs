use std::fs;
use std::io::{IsTerminal, stdin};
use std::path::Path;

use cliclack::select;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use vtracer::{ColorImage, Config};

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

pub(crate) fn run_convert(src: &str, dst: &str) -> Result<(), String> {
    let dst = crate::io::enumerate_if_exists(Path::new(dst))
        .to_string_lossy()
        .to_string();

    if is_svg_path(&dst) {
        return vectorize(src, &dst, false, false, false);
    }

    if is_svg_path(src) {
        return rasterize(src, &dst, RasterizeOptions::default(), false);
    }

    let img = image::open(src).map_err(|e| format!("convert: failed to open image '{src}': {e}"))?;
    crate::io::save_image(img, &dst)?;
    println!("Converted image to {}", dst);
    Ok(())
}

#[derive(Clone, Copy, Default)]
pub(crate) struct RasterizeOptions {
    pub(crate) scale: Option<f32>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

pub(crate) struct RasterizeArgs {
    pub(crate) options: RasterizeOptions,
    pub(crate) src: String,
    pub(crate) dst: String,
    pub(crate) preview: bool,
}

#[derive(Debug)]
pub(crate) struct VectorizeArgs {
    pub(crate) src: String,
    pub(crate) dst: String,
    pub(crate) fast: bool,
    pub(crate) full_quality: bool,
    pub(crate) preview: bool,
}

pub(crate) fn run_vectorize(args: VectorizeArgs) -> Result<(), String> {
    let dst = crate::io::enumerate_if_exists(Path::new(&args.dst))
        .to_string_lossy()
        .to_string();
    vectorize(&args.src, &dst, args.fast, args.full_quality, args.preview)
}

pub(crate) fn run_rasterize(args: RasterizeArgs) -> Result<(), String> {
    let dst = crate::io::enumerate_if_exists(Path::new(&args.dst))
        .to_string_lossy()
        .to_string();
    rasterize(&args.src, &dst, args.options, args.preview)
}

fn vectorize(src: &str, dst: &str, fast: bool, full_quality: bool, preview: bool) -> Result<(), String> {
    if is_svg_path(src) {
        return Err(format!(
            "vectorize: unsupported file format '{}'",
            Path::new(src).display()
        ));
    }

    let src_path = Path::new(src);
    let config = if fast { fast_vectorize_config() } else { Config::default() };

    let (color_img, config, orig_w, orig_h) = prepare_vectorize_input(src_path, config, full_quality)?;
    let scaled_w = color_img.width;
    let scaled_h = color_img.height;

    let spinner = start_spinner(if preview { "Vectorizing image for preview..." } else { "Vectorizing image..." });
    let svg_result = vtracer::convert(color_img, config)
        .map_err(|e| format!("vectorize: failed to vectorize '{}': {e}", src_path.display()));
    if let Some(pb) = spinner { pb.finish_and_clear(); }
    let svg_file = svg_result?;

    let svg_str = format!("{svg_file}");

    if preview {
        let usvg_options = Options::default();
        let tree = Tree::from_data(svg_str.as_bytes(), &usvg_options)
            .map_err(|e| format!("vectorize: failed to parse vectorized SVG for preview: {e}"))?;
        let size = tree.size().to_int_size();
        let mut pixmap = Pixmap::new(size.width(), size.height())
            .ok_or_else(|| "vectorize: failed to create pixmap for preview".to_string())?;
        resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
        let image = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(size.width(), size.height(), pixmap.take_demultiplied())
                .ok_or_else(|| "vectorize: failed to build image buffer for preview".to_string())?,
        );
        return crate::commands::view::display_image(image);
    }

    let svg_str = if scaled_w != orig_w || scaled_h != orig_h {
        patch_svg_dimensions(svg_str, scaled_w, scaled_h, orig_w, orig_h)
    } else {
        svg_str
    };

    let dst_path = Path::new(dst);
    fs::write(dst_path, &svg_str)
        .map_err(|e| format!("vectorize: failed to write '{}': {e}", dst_path.display()))?;
    println!("Converted image to {}", dst);
    Ok(())
}

const MAX_LONG_EDGE: usize = 2000;

fn prepare_vectorize_input(
    src_path: &Path,
    config: Config,
    full_quality: bool,
) -> Result<(ColorImage, Config, usize, usize), String> {
    let img = image::open(src_path)
        .map_err(|e| format!("vectorize: failed to open image '{}': {e}", src_path.display()))?;

    let orig_w = img.width() as usize;
    let orig_h = img.height() as usize;

    let (img, config) = if !full_quality && orig_w.max(orig_h) > MAX_LONG_EDGE {
        let scale = MAX_LONG_EDGE as f64 / orig_w.max(orig_h) as f64;
        let resized = img.resize(MAX_LONG_EDGE as u32, MAX_LONG_EDGE as u32, image::imageops::FilterType::Triangle);
        let config = Config {
            filter_speckle: ((config.filter_speckle as f64 * scale).floor() as usize).max(1),
            length_threshold: config.length_threshold * scale,
            ..config
        };
        (resized, config)
    } else {
        (img, config)
    };

    let rgba = img.to_rgba8();
    let w = rgba.width() as usize;
    let h = rgba.height() as usize;
    let color_img = ColorImage { pixels: rgba.into_raw(), width: w, height: h };

    Ok((color_img, config, orig_w, orig_h))
}

fn patch_svg_dimensions(svg: String, scaled_w: usize, scaled_h: usize, orig_w: usize, orig_h: usize) -> String {
    // vtracer's opening tag format is fixed — safe to do a targeted replace
    let old_tag = format!(
        r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="{scaled_w}" height="{scaled_h}">"#
    );
    let new_tag = format!(
        r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="{orig_w}" height="{orig_h}" viewBox="0 0 {scaled_w} {scaled_h}">"#
    );
    svg.replacen(&old_tag, &new_tag, 1)
}

fn rasterize(src: &str, dst: &str, options: RasterizeOptions, preview: bool) -> Result<(), String> {
    if !is_svg_path(src) {
        return Err(format!(
            "rasterize: unsupported file format '{}': only SVG files are accepted",
            Path::new(src).display()
        ));
    }

    let src_path = Path::new(src);
    let dst_path = Path::new(dst);
    let svg_data = fs::read(src_path)
        .map_err(|e| format!("rasterize: failed to read SVG '{}': {e}", src_path.display()))?;

    let usvg_options = Options {
        resources_dir: src_path.parent().map(Path::to_path_buf),
        ..Options::default()
    };

    let tree = Tree::from_data(&svg_data, &usvg_options)
        .map_err(|e| format!("rasterize: failed to parse SVG '{}': {e}", src_path.display()))?;

    let (render_width, render_height, scale_x, scale_y) =
        compute_render_dimensions(tree.size(), &options)?;
    let mut pixmap = Pixmap::new(render_width, render_height).ok_or_else(|| {
        format!(
            "rasterize: failed to create pixmap for '{}' with size {}x{}",
            src_path.display(),
            render_width,
            render_height
        )
    })?;

    resvg::render(
        &tree,
        Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    if preview {
        let image = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(render_width, render_height, pixmap.take_demultiplied())
                .ok_or_else(|| {
                    format!(
                        "rasterize: failed to build image buffer for preview of '{}'",
                        src_path.display()
                    )
                })?,
        );
        return crate::commands::view::display_image(image);
    }

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
                "rasterize: failed to build image buffer for '{}' with size {}x{}",
                output_path.display(),
                width,
                height
            )
        })?;

    crate::io::save_image(image::DynamicImage::ImageRgba8(image), output_path)
}

pub(crate) fn prompt_convert_format(src: &str) -> Result<String, String> {
    let src_ext = Path::new(src)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let src_ext = if src_ext == "jpeg" { "jpg" } else { src_ext.as_str() }.to_string();

    let all_formats: &[&str] = if is_svg_path(src) {
        &["png", "jpg", "webp", "ico"]
    } else {
        &["png", "jpg", "webp", "ico", "svg"]
    };

    let formats: Vec<&str> = all_formats
        .iter()
        .filter(|&&f| f != src_ext.as_str())
        .copied()
        .collect();

    if !stdin().is_terminal() {
        return prompt_convert_format_non_tty(&formats);
    }

    let mut prompt = select("Choose output format:");
    for fmt in &formats {
        prompt = prompt.item(fmt, fmt.to_uppercase(), "");
    }
    let choice = prompt
        .interact()
        .map_err(|e| format!("failed to read format: {e}"))?;

    Ok((*choice).to_string())
}

fn prompt_convert_format_non_tty(formats: &[&str]) -> Result<String, String> {
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read format from stdin: {e}"))?;

    let trimmed = input.trim();
    if let Ok(n) = trimmed.parse::<usize>()
        && n >= 1 && n <= formats.len()
    {
        return Ok(formats[n - 1].to_string());
    }
    for fmt in formats {
        if fmt.eq_ignore_ascii_case(trimmed) {
            return Ok(fmt.to_string());
        }
    }
    Err(format!(
        "invalid format '{trimmed}': enter a number (1-{}) or a format name",
        formats.len()
    ))
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
            false,
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

        run_convert(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        )
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

        run_convert(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        )
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

        run_convert(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        )
        .expect("svg to webp conversion failed");

        assert!(output_path.exists());
        let converted = image::open(&output_path).expect("failed to open converted webp");
        assert_eq!(converted.width(), 4);
        assert_eq!(converted.height(), 4);

        let _ = fs::remove_dir_all(&temp_root);
    }

    fn make_temp_png(dir: &std::path::Path, name: &str, w: u32, h: u32) -> std::path::PathBuf {
        let path = dir.join(name);
        image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
            w, h, image::Rgba([100u8, 100, 100, 255]),
        ))
        .save(&path)
        .expect("failed to save test png");
        path
    }

    #[test]
    fn test_prepare_vectorize_input_no_downscale_when_under_threshold() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-pvi-under-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        let img_path = make_temp_png(&temp_root, "input.png", 1000, 500);

        let (color_img, out_config, orig_w, orig_h) =
            prepare_vectorize_input(&img_path, Config::default(), false).unwrap();

        assert_eq!((orig_w, orig_h), (1000, 500));
        assert_eq!((color_img.width, color_img.height), (1000, 500));
        assert_eq!(out_config.filter_speckle, 4);
        assert!((out_config.length_threshold - 4.0).abs() < f64::EPSILON);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_prepare_vectorize_input_downscales_large_image() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-pvi-large-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        // Long edge 2200 > 2000 threshold; scale = 2000/2200 ≈ 0.9091
        let img_path = make_temp_png(&temp_root, "input.png", 2200, 100);

        let (color_img, out_config, orig_w, orig_h) =
            prepare_vectorize_input(&img_path, Config::default(), false).unwrap();

        let scale = 2000.0_f64 / 2200.0;
        assert_eq!((orig_w, orig_h), (2200, 100));
        assert_eq!(color_img.width, 2000);
        assert_eq!(color_img.height, (100.0 * scale).round() as usize);
        // filter_speckle: floor(4 * scale) = floor(3.636) = 3
        assert_eq!(out_config.filter_speckle, (4.0_f64 * scale).floor() as usize);
        assert!((out_config.length_threshold - 4.0 * scale).abs() < 1e-9);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_prepare_vectorize_input_full_quality_skips_downscale() {
        let temp_root = std::env::temp_dir().join(format!(
            "simply-edit-pvi-fullq-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&temp_root).unwrap();
        let img_path = make_temp_png(&temp_root, "input.png", 2200, 100);

        let (color_img, out_config, orig_w, orig_h) =
            prepare_vectorize_input(&img_path, Config::default(), true).unwrap();

        assert_eq!((orig_w, orig_h), (2200, 100));
        assert_eq!((color_img.width, color_img.height), (2200, 100));
        assert_eq!(out_config.filter_speckle, 4);
        assert!((out_config.length_threshold - 4.0).abs() < f64::EPSILON);

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_patch_svg_dimensions() {
        let svg = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!-- Generator: visioncortex VTracer 0.6.5 -->\n",
            "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" width=\"2000\" height=\"837\">\n",
            "</svg>\n",
        ).to_string();

        let patched = patch_svg_dimensions(svg, 2000, 837, 3440, 1440);

        assert!(patched.contains(r#"width="3440""#), "original width not restored");
        assert!(patched.contains(r#"height="1440""#), "original height not restored");
        assert!(patched.contains(r#"viewBox="0 0 2000 837""#), "viewBox not injected");
        assert!(!patched.contains(r#"width="2000" height="837""#), "old tag still present");
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
