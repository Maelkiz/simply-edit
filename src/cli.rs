use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "simply", about = "simply-edit", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct BatchArgs {
    /// Regex pattern to filter filenames (batch mode)
    #[arg(long)]
    pub pattern: Option<String>,

    /// Output directory for batch results
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Process subdirectories recursively (batch mode)
    #[arg(short = 'R', long)]
    pub recursive: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Flip an image horizontally or vertically
    Flip {
        /// Flip horizontally (bypasses interactive prompt)
        #[arg(long)]
        horizontal: bool,

        /// Flip vertically (bypasses interactive prompt)
        #[arg(long)]
        vertical: bool,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        #[command(flatten)]
        batch: BatchArgs,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,
    },

    /// Rotate an image by 90, 180, or 270 degrees
    Rotate {
        /// Rotation angle: 90, 180, or 270 (interactive prompt if omitted)
        #[arg(long, value_parser = parse_rotation)]
        angle: Option<u16>,

        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        #[command(flatten)]
        batch: BatchArgs,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,
    },

    /// Invert the colors of an image
    Invert {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        #[command(flatten)]
        batch: BatchArgs,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,
    },

    /// Convert an image to grayscale
    Grayscale {
        /// Overwrite target file (source if no output path given)
        #[arg(short, long)]
        replace: bool,

        #[command(flatten)]
        batch: BatchArgs,

        /// Path to image file or directory
        path: String,

        /// Output path (auto-generated if omitted)
        output: Option<String>,
    },

    /// Convert between image formats (PNG, JPG, ICO, SVG, WebP)
    Convert {
        /// Output format for batch mode (e.g. png, jpg, webp)
        #[arg(long)]
        format: Option<String>,

        #[command(flatten)]
        batch: BatchArgs,

        /// Source image path or directory
        src: String,

        /// Output path (required for single-file mode)
        dst: Option<String>,
    },

    /// Convert a raster image to SVG
    Vectorize {
        /// Faster conversion with lower fidelity
        #[arg(long)]
        fast: bool,

        #[command(flatten)]
        batch: BatchArgs,

        /// Source image path or directory
        src: String,

        /// Output SVG path (auto-generated if omitted)
        dst: Option<String>,
    },

    /// Convert an SVG to a raster image
    Rasterize {
        /// Scale factor for rasterization
        #[arg(short, long)]
        scale: Option<f32>,

        /// Output width in pixels
        #[arg(short, long)]
        width: Option<u32>,

        /// Output height in pixels
        #[arg(short = 'H', long)]
        height: Option<u32>,

        #[command(flatten)]
        batch: BatchArgs,

        /// Source SVG path or directory
        src: String,

        /// Output path (auto-generated if omitted)
        dst: Option<String>,
    },
}

fn parse_rotation(s: &str) -> Result<u16, String> {
    match s {
        "90" => Ok(90),
        "180" => Ok(180),
        "270" => Ok(270),
        _ => Err(format!("invalid rotation '{s}': use 90, 180, or 270")),
    }
}

pub(crate) fn flip_axis_from_flags(
    horizontal: bool,
    vertical: bool,
) -> Result<Option<crate::commands::transforms::FlipAxis>, String> {
    match (horizontal, vertical) {
        (true, true) => Err("flip: choose only one of --horizontal or --vertical".to_string()),
        (true, false) => Ok(Some(crate::commands::transforms::FlipAxis::Horizontal)),
        (false, true) => Ok(Some(crate::commands::transforms::FlipAxis::Vertical)),
        (false, false) => Ok(None),
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
    fn test_flip_horizontal() {
        match parse(&["simply", "flip", "--horizontal", "image.png"]) {
            Command::Flip {
                horizontal: true,
                vertical: false,
                replace: false,
                path,
                output: None,
                ..
            } => assert_eq!(path, "image.png"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_flip_vertical_with_output() {
        match parse(&["simply", "flip", "--vertical", "image.png", "out.png"]) {
            Command::Flip {
                horizontal: false,
                vertical: true,
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
    fn test_flip_axis_from_flags_both() {
        assert!(flip_axis_from_flags(true, true).is_err());
    }

    #[test]
    fn test_flip_axis_from_flags_none() {
        assert_eq!(flip_axis_from_flags(false, false).unwrap(), None);
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
