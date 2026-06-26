use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "simply", about = "Image editing from the terminal", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Args)]
#[command(next_help_heading = "Batch Options")]
pub(crate) struct BatchArgs {
    /// Regex pattern to filter filenames
    #[arg(long)]
    pub pattern: Option<String>,

    /// Output directory for batch results
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Process subdirectories recursively
    #[arg(short = 'R', long)]
    pub recursive: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Mirror an image along the X axis (vertical), Y axis (horizontal), or both
    Flip {
        /// Flip along the X axis (vertical mirror, top to bottom)
        #[arg(short = 'x', long)]
        x: bool,

        /// Flip along the Y axis (horizontal mirror, left to right)
        #[arg(short = 'y', long)]
        y: bool,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Rotate an image by 90, 180, or 270 degrees
    Rotate {
        /// Rotation angle: 90, 180, or 270 (interactive prompt if omitted)
        #[arg(long, value_parser = parse_rotation)]
        angle: Option<u16>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Invert the colors of an image
    Invert {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert an image to grayscale
    Grayscale {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert an image to pure black and white at a brightness cutoff
    Binarize {
        /// Threshold value 0-255 (default: 128). Pixels brighter than this become white, others black
        #[arg(short, long, value_parser = parse_threshold)]
        threshold: Option<u8>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Resize an image to specified dimensions
    Resize {
        /// Target width in pixels
        #[arg(short, long, value_parser = parse_positive_u32)]
        width: Option<u32>,

        /// Target height in pixels
        #[arg(short = 'H', long, value_parser = parse_positive_u32)]
        height: Option<u32>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Scale an image by a factor (e.g. 0.5 to halve, 2.0 to double)
    Scale {
        /// Scale factor (e.g. 0.5 to halve, 2.0 to double)
        #[arg(short = 'f', long, value_parser = parse_positive_f32_factor)]
        factor: Option<f32>,

        /// Scale factor for width only (e.g. 0.5 to halve width)
        #[arg(short = 'x', long, value_parser = parse_positive_f32_x)]
        x: Option<f32>,

        /// Scale factor for height only (e.g. 2.0 to double height)
        #[arg(short = 'y', long, value_parser = parse_positive_f32_y)]
        y: Option<f32>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert between image formats (PNG, JPG, ICO, SVG, WebP)
    Convert {
        /// Output format for batch mode (e.g. png, jpg, webp)
        #[arg(long)]
        format: Option<String>,

        /// Source image path or directory
        src: String,

        /// Output path (interactive format prompt if omitted)
        dst: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert a raster image to SVG
    Vectorize {
        /// Faster conversion with lower fidelity
        #[arg(long)]
        fast: bool,

        /// Disable automatic downscaling — vectorize at full resolution (higher quality, but much slower and more memory-intensive, especially for large images)
        #[arg(long)]
        full_quality: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Source image path or directory
        src: String,

        /// Output SVG path (auto-generated if omitted)
        dst: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Display image metadata and properties
    Info {
        /// Path to the image file
        path: String,
    },

    /// Display an image inline in the terminal (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
    View {
        /// Path to the image file
        path: String,
    },

    /// Add transparent (or colored) padding around an image
    Pad {
        /// Pixels to add on the top edge
        #[arg(long, value_parser = parse_positive_u32)]
        top: Option<u32>,

        /// Pixels to add on the bottom edge
        #[arg(long, value_parser = parse_positive_u32)]
        bottom: Option<u32>,

        /// Pixels to add on the left edge
        #[arg(long, value_parser = parse_positive_u32)]
        left: Option<u32>,

        /// Pixels to add on the right edge
        #[arg(long, value_parser = parse_positive_u32)]
        right: Option<u32>,

        /// Pixels to add on every side (overridden by -x, -y, or individual side flags)
        #[arg(long, value_parser = parse_positive_u32)]
        px: Option<u32>,

        /// Shorthand: pixels to add on both left and right (overridden by --left/--right)
        #[arg(short = 'x', value_parser = parse_positive_u32)]
        horizontal: Option<u32>,

        /// Shorthand: pixels to add on both top and bottom (overridden by --top/--bottom)
        #[arg(short = 'y', value_parser = parse_positive_u32)]
        vertical: Option<u32>,

        /// Fill color as hex (e.g. ffffff, #ff0000, #ff000080). Defaults to transparent
        #[arg(short = 'c', long, value_parser = parse_color)]
        color: Option<[u8; 4]>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert an SVG to a raster image
    Rasterize {
        /// Scale factor for rasterization
        #[arg(short, long, value_parser = parse_positive_f32)]
        scale: Option<f32>,

        /// Output width in pixels
        #[arg(short, long, value_parser = parse_positive_u32)]
        width: Option<u32>,

        /// Output height in pixels
        #[arg(short = 'H', long, value_parser = parse_positive_u32)]
        height: Option<u32>,

        /// Preview the result in the terminal without saving (requires Kitty graphics protocol support (Kitty, WezTerm, or Ghostty))
        #[arg(short = 'p', long)]
        preview: bool,

        /// Source SVG path or directory
        src: String,

        /// Output path (auto-generated if omitted)
        dst: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },
}

fn parse_threshold(s: &str) -> Result<u8, String> {
    s.parse()
        .map_err(|_| format!("invalid threshold '{s}': use an integer from 0 to 255"))
}

fn parse_positive_f32_impl(s: &str, flag: &str) -> Result<f32, String> {
    let v: f32 = s
        .parse()
        .map_err(|_| format!("invalid value '{s}' for {flag}: use a positive number"))?;
    if !v.is_finite() || v <= 0.0 {
        return Err(format!(
            "invalid value '{s}' for {flag}: use a positive number"
        ));
    }
    Ok(v)
}

fn parse_positive_f32(s: &str) -> Result<f32, String> {
    parse_positive_f32_impl(s, "--scale")
}

fn parse_positive_f32_factor(s: &str) -> Result<f32, String> {
    parse_positive_f32_impl(s, "--factor")
}

fn parse_positive_f32_x(s: &str) -> Result<f32, String> {
    parse_positive_f32_impl(s, "--x")
}

fn parse_positive_f32_y(s: &str) -> Result<f32, String> {
    parse_positive_f32_impl(s, "--y")
}

fn parse_positive_u32(s: &str) -> Result<u32, String> {
    let v: u32 = s
        .parse()
        .map_err(|_| format!("invalid value '{s}': use a positive integer"))?;
    if v == 0 {
        return Err(format!(
            "invalid value '{s}': use a positive integer"
        ));
    }
    Ok(v)
}

fn parse_color(s: &str) -> Result<[u8; 4], String> {
    let hex = s.trim_start_matches('#');
    let parse_byte = |h: &str| {
        u8::from_str_radix(h, 16)
            .map_err(|_| format!("invalid color '{s}': use a hex string like ffffff or #ff000080"))
    };
    match hex.len() {
        6 => Ok([
            parse_byte(&hex[0..2])?,
            parse_byte(&hex[2..4])?,
            parse_byte(&hex[4..6])?,
            255,
        ]),
        8 => Ok([
            parse_byte(&hex[0..2])?,
            parse_byte(&hex[2..4])?,
            parse_byte(&hex[4..6])?,
            parse_byte(&hex[6..8])?,
        ]),
        _ => Err(format!(
            "invalid color '{s}': use a 6-digit (rrggbb) or 8-digit (rrggbbaa) hex string"
        )),
    }
}

fn parse_rotation(s: &str) -> Result<u16, String> {
    match s {
        "90" => Ok(90),
        "180" => Ok(180),
        "270" => Ok(270),
        _ => Err(format!("invalid rotation '{s}': use 90, 180, or 270")),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Command {
        let cli = Cli::parse_from(args);
        cli.command
    }

    fn try_parse(args: &[&str]) -> Result<Command, clap::Error> {
        Cli::try_parse_from(args).map(|cli| cli.command)
    }

    #[test]
    fn test_flip_basic() {
        match parse(&["simply", "flip", "image.png"]) {
            Command::Flip {
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_with_output() {
        match parse(&["simply", "flip", "image.png", "out.png"]) {
            Command::Flip {
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "image.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_replace() {
        match parse(&["simply", "flip", "--replace", "image.png"]) {
            Command::Flip {
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_replace_short() {
        match parse(&["simply", "flip", "-r", "image.png"]) {
            Command::Flip {
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_x_flag() {
        match parse(&["simply", "flip", "-x", "image.png"]) {
            Command::Flip { x: true, y: false, path, .. } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_y_flag() {
        match parse(&["simply", "flip", "-y", "image.png"]) {
            Command::Flip { x: false, y: true, path, .. } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_xy_flags() {
        match parse(&["simply", "flip", "-x", "-y", "image.png"]) {
            Command::Flip { x: true, y: true, path, .. } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_y_basic() {
        match parse(&["simply", "flip", "-y", "image.png"]) {
            Command::Flip {
                x: false,
                y: true,
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_y_with_output() {
        match parse(&["simply", "flip", "-y", "image.png", "out.png"]) {
            Command::Flip {
                y: true,
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "image.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_y_replace() {
        match parse(&["simply", "flip", "-y", "-r", "image.png"]) {
            Command::Flip {
                y: true,
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rotate_with_angle() {
        match parse(&["simply", "rotate", "--angle", "90", "image.png"]) {
            Command::Rotate {
                angle: Some(90),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rotate_without_angle() {
        match parse(&["simply", "rotate", "image.png"]) {
            Command::Rotate {
                angle: None, path, ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rotate_invalid_angle() {
        let result = try_parse(&["simply", "rotate", "--angle", "45", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rotate_replace() {
        match parse(&[
            "simply",
            "rotate",
            "--replace",
            "--angle",
            "180",
            "image.png",
        ]) {
            Command::Rotate {
                angle: Some(180),
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_invert() {
        match parse(&["simply", "invert", "image.png"]) {
            Command::Invert {
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_invert_with_output() {
        match parse(&["simply", "invert", "image.png", "out.png"]) {
            Command::Invert {
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "image.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_invert_replace() {
        match parse(&["simply", "invert", "--replace", "image.png"]) {
            Command::Invert {
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_grayscale() {
        match parse(&["simply", "grayscale", "image.png"]) {
            Command::Grayscale {
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_grayscale_replace() {
        match parse(&["simply", "grayscale", "-r", "image.png"]) {
            Command::Grayscale {
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_convert() {
        match parse(&["simply", "convert", "in.png", "out.jpg"]) {
            Command::Convert {
                src,
                dst: Some(dst),
                ..
            } => {
                assert_eq!(src, "in.png");
                assert_eq!(dst, "out.jpg");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_vectorize() {
        match parse(&["simply", "vectorize", "in.png", "out.svg"]) {
            Command::Vectorize {
                fast: false,
                src,
                dst: Some(dst),
                ..
            } => {
                assert_eq!(src, "in.png");
                assert_eq!(dst, "out.svg");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_vectorize_fast() {
        match parse(&["simply", "vectorize", "--fast", "in.png"]) {
            Command::Vectorize {
                fast: true,
                src,
                dst: None,
                ..
            } => assert_eq!(src, "in.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_vectorize_full_quality() {
        match parse(&["simply", "vectorize", "--full-quality", "in.png"]) {
            Command::Vectorize {
                full_quality: true,
                src,
                dst: None,
                ..
            } => assert_eq!(src, "in.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rasterize() {
        match parse(&["simply", "rasterize", "in.svg", "out.png"]) {
            Command::Rasterize {
                scale: None,
                width: None,
                height: None,
                src,
                dst: Some(dst),
                ..
            } => {
                assert_eq!(src, "in.svg");
                assert_eq!(dst, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rasterize_with_flags() {
        match parse(&[
            "simply",
            "rasterize",
            "-s",
            "2.5",
            "-w",
            "200",
            "-H",
            "100",
            "in.svg",
            "out.png",
        ]) {
            Command::Rasterize {
                scale: Some(s),
                width: Some(200),
                height: Some(100),
                src,
                dst: Some(dst),
                ..
            } => {
                assert!((s - 2.5).abs() < f32::EPSILON);
                assert_eq!(src, "in.svg");
                assert_eq!(dst, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_rasterize_no_output() {
        match parse(&["simply", "rasterize", "in.svg"]) {
            Command::Rasterize { src, dst: None, .. } => assert_eq!(src, "in.svg"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_batch_flags_on_invert() {
        match parse(&[
            "simply",
            "invert",
            "--pattern",
            ".*\\.jpg$",
            "--output-dir",
            "/tmp/out",
            "--recursive",
            "image.png",
        ]) {
            Command::Invert { batch, path, .. } => {
                assert_eq!(batch.pattern.as_deref(), Some(".*\\.jpg$"));
                assert_eq!(batch.output_dir, Some(PathBuf::from("/tmp/out")));
                assert!(batch.recursive);
                assert_eq!(path, "image.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_resize_with_dimensions() {
        match parse(&["simply", "resize", "--width", "800", "-H", "600", "image.png"]) {
            Command::Resize {
                width: Some(800),
                height: Some(600),
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_resize_no_dimensions() {
        match parse(&["simply", "resize", "image.png"]) {
            Command::Resize {
                width: None,
                height: None,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_resize_replace() {
        match parse(&["simply", "resize", "-r", "--width", "100", "-H", "100", "image.png"]) {
            Command::Resize {
                replace: true,
                width: Some(100),
                height: Some(100),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_resize_with_output() {
        match parse(&["simply", "resize", "--width", "50", "-H", "50", "in.png", "out.png"]) {
            Command::Resize {
                width: Some(50),
                height: Some(50),
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "in.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_default() {
        match parse(&["simply", "binarize", "image.png"]) {
            Command::Binarize {
                threshold: None,
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_with_threshold() {
        match parse(&["simply", "binarize", "--threshold", "200", "image.png"]) {
            Command::Binarize {
                threshold: Some(200),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_short_threshold() {
        match parse(&["simply", "binarize", "-t", "100", "image.png"]) {
            Command::Binarize {
                threshold: Some(100),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_replace() {
        match parse(&["simply", "binarize", "-r", "image.png"]) {
            Command::Binarize {
                replace: true,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_with_output() {
        match parse(&["simply", "binarize", "in.png", "out.png"]) {
            Command::Binarize {
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "in.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_binarize_invalid_threshold() {
        let result = try_parse(&["simply", "binarize", "--threshold", "999", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_binarize_non_numeric_threshold() {
        let result = try_parse(&["simply", "binarize", "--threshold", "abc", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resize_zero_width_rejected() {
        let result = try_parse(&["simply", "resize", "--width", "0", "-H", "100", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pad_no_flags() {
        match parse(&["simply", "pad", "image.png"]) {
            Command::Pad {
                top: None,
                bottom: None,
                left: None,
                right: None,
                horizontal: None,
                vertical: None,
                color: None,
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_individual_sides() {
        match parse(&["simply", "pad", "--top", "10", "--bottom", "20", "--left", "5", "--right", "15", "image.png"]) {
            Command::Pad {
                top: Some(10),
                bottom: Some(20),
                left: Some(5),
                right: Some(15),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_horizontal_shorthand() {
        match parse(&["simply", "pad", "-x", "40", "image.png"]) {
            Command::Pad {
                horizontal: Some(40),
                vertical: None,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_vertical_shorthand() {
        match parse(&["simply", "pad", "-y", "15", "image.png"]) {
            Command::Pad {
                vertical: Some(15),
                horizontal: None,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_color_6digit() {
        match parse(&["simply", "pad", "-c", "ff0000", "--top", "5", "image.png"]) {
            Command::Pad {
                color: Some([255, 0, 0, 255]),
                top: Some(5),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_color_8digit_with_hash() {
        match parse(&["simply", "pad", "--color", "#00ff0080", "--top", "5", "image.png"]) {
            Command::Pad {
                color: Some([0, 255, 0, 128]),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_replace() {
        match parse(&["simply", "pad", "-r", "--top", "10", "image.png"]) {
            Command::Pad {
                replace: true,
                top: Some(10),
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_with_output() {
        match parse(&["simply", "pad", "--top", "5", "in.png", "out.png"]) {
            Command::Pad {
                top: Some(5),
                path,
                output: Some(out),
                ..
            } => {
                assert_eq!(path, "in.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_px_flag() {
        match parse(&["simply", "pad", "--px", "20", "image.png"]) {
            Command::Pad {
                px: Some(20),
                top: None,
                bottom: None,
                left: None,
                right: None,
                horizontal: None,
                vertical: None,
                path,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_pad_invalid_color_rejected() {
        let result = try_parse(&["simply", "pad", "--color", "zzzzzz", "--top", "5", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pad_batch_flags() {
        match parse(&["simply", "pad", "-x", "10", "--output-dir", "/tmp/out", "images/"]) {
            Command::Pad {
                horizontal: Some(10),
                batch,
                ..
            } => assert_eq!(batch.output_dir, Some(std::path::PathBuf::from("/tmp/out"))),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_convert_with_format_flag() {
        match parse(&["simply", "convert", "--format", "webp", "/photos"]) {
            Command::Convert {
                format: Some(fmt),
                src,
                dst: None,
                ..
            } => {
                assert_eq!(fmt, "webp");
                assert_eq!(src, "/photos");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_with_factor() {
        match parse(&["simply", "scale", "--factor", "0.5", "image.png"]) {
            Command::Scale {
                factor: Some(f),
                replace: false,
                path,
                output: None,
                ..
            } => {
                assert!((f - 0.5).abs() < f32::EPSILON);
                assert_eq!(path, "image.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_no_factor() {
        match parse(&["simply", "scale", "image.png"]) {
            Command::Scale {
                factor: None,
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_short_factor() {
        match parse(&["simply", "scale", "-f", "2", "image.png"]) {
            Command::Scale { factor: Some(f), .. } => assert!((f - 2.0).abs() < f32::EPSILON),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_replace() {
        match parse(&["simply", "scale", "-r", "--factor", "1.5", "image.png"]) {
            Command::Scale { replace: true, factor: Some(f), path, .. } => {
                assert!((f - 1.5).abs() < f32::EPSILON);
                assert_eq!(path, "image.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_with_output() {
        match parse(&["simply", "scale", "--factor", "2", "in.png", "out.png"]) {
            Command::Scale { factor: Some(f), path, output: Some(out), .. } => {
                assert!((f - 2.0).abs() < f32::EPSILON);
                assert_eq!(path, "in.png");
                assert_eq!(out, "out.png");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_scale_zero_factor_rejected() {
        let result = try_parse(&["simply", "scale", "--factor", "0", "image.png"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_scale_negative_factor_rejected() {
        let result = try_parse(&["simply", "scale", "--factor", "-1", "image.png"]);
        assert!(result.is_err());
    }
}
