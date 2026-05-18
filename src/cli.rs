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
    /// Mirror an image vertically (top to bottom)
    Flip {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
        #[arg(short = 'p', long)]
        preview: bool,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Mirror an image horizontally (left to right)
    Flop {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Scale factor (e.g. 0.5 to halve, 2.0 to double)
        #[arg(short, long, value_parser = parse_positive_f32)]
        scale: Option<f32>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

        /// Output path (required for single-file mode)
        dst: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,
    },

    /// Convert a raster image to SVG
    Vectorize {
        /// Faster conversion with lower fidelity
        #[arg(long)]
        fast: bool,

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

    /// Display an image in the terminal (inline in Kitty, WezTerm, or Ghostty)
    View {
        /// Path to the image file
        path: String,
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

        /// Preview the result in the terminal without saving (requires Kitty, WezTerm, or Ghostty)
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

fn parse_positive_f32(s: &str) -> Result<f32, String> {
    let v: f32 = s
        .parse()
        .map_err(|_| format!("invalid value '{s}' for --scale: use a positive number"))?;
    if !v.is_finite() || v <= 0.0 {
        return Err(format!(
            "invalid value '{s}' for --scale: use a positive number"
        ));
    }
    Ok(v)
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
    fn test_flop_basic() {
        match parse(&["simply", "flop", "image.png"]) {
            Command::Flop {
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flop_with_output() {
        match parse(&["simply", "flop", "image.png", "out.png"]) {
            Command::Flop {
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
    fn test_flop_replace() {
        match parse(&["simply", "flop", "-r", "image.png"]) {
            Command::Flop {
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
}
