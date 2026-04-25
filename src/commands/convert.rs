use std::fs;
use std::path::Path;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use vtracer::Config;

pub(crate) fn run_convert(args: &[String]) -> Result<(), String> {
    let ConvertArgs { options, src, dst } = parse_convert_args(args)?;
    let dst = crate::io::enumerate_if_exists(Path::new(&dst))
        .to_string_lossy()
        .to_string();

    if is_svg_path(&dst) {
        if !options.is_empty() {
            return Err(
                "scale and width/height flags are only supported for SVG to image conversion"
                    .to_string(),
            );
        }

        return convert_image_to_svg(&src, &dst);
    }

    if is_svg_path(&src) {
        return convert_svg_to_image(&src, &dst, options);
    }

    if !options.is_empty() {
        return Err(
            "scale and width/height flags are only supported for SVG to image conversion"
                .to_string(),
        );
    }

    let img = image::open(&src).map_err(|e| format!("failed to open image '{src}': {e}"))?;
    crate::io::save_image(img, &dst)?;
    println!("Converted image to {}", dst);
    Ok(())
}

#[derive(Default)]
pub(crate) struct ConvertOptions {
    pub(crate) scale: Option<f32>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

impl ConvertOptions {
    fn is_empty(&self) -> bool {
        self.scale.is_none() && self.width.is_none() && self.height.is_none()
    }
}

pub(crate) struct ConvertArgs {
    pub(crate) options: ConvertOptions,
    pub(crate) src: String,
    pub(crate) dst: String,
}

pub(crate) fn parse_convert_args(args: &[String]) -> Result<ConvertArgs, String> {
    let mut options = ConvertOptions::default();
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
                return Err(format!("unrecognized convert flag '{value}'"));
            }
            value => {
                positionals.push(value.to_string());
                index += 1;
            }
        }
    }

    match positionals.as_slice() {
        [src, dst] => Ok(ConvertArgs {
            options,
            src: src.clone(),
            dst: dst.clone(),
        }),
        _ => Err(crate::usage()),
    }
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

pub(crate) fn convert_svg_to_image(
    src: &str,
    dst: &str,
    options: ConvertOptions,
) -> Result<(), String> {
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

pub(crate) fn compute_render_dimensions(
    size: resvg::usvg::Size,
    options: &ConvertOptions,
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
        let options = ConvertOptions {
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
        let options = ConvertOptions {
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
