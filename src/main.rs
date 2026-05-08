mod batch;
mod cli;
mod commands;
mod io;

use std::path::Path;

use clap::Parser;

use cli::{BatchArgs, Cli, Command};
use commands::convert::{RasterizeArgs, RasterizeOptions, VectorizeArgs};

enum OutputMode {
    Generated,
    Explicit(String),
    Replace(Option<String>),
    Preview,
}

pub(crate) enum SaveMode<'a> {
    Generated(&'a str),
    Explicit(String),
    Replace(Option<String>),
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Command::Flip {
            horizontal,
            vertical,
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            let axis = cli::flip_axis_from_flags(horizontal, vertical)?;
            if is_batch(&path, &batch) {
                if preview {
                    return Err("flip: --preview cannot be used in batch mode".to_string());
                }
                if axis.is_none() {
                    return Err(
                        "flip: --horizontal or --vertical required in batch mode".to_string()
                    );
                }
                let axis = axis.unwrap();
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("failed to open image '{}': {e}", file.display()))?;
                    let (flipped, suffix) = match axis {
                        commands::transforms::FlipAxis::Horizontal => (img.fliph(), "fliph"),
                        commands::transforms::FlipAxis::Vertical => (img.flipv(), "flipv"),
                    };
                    let out_path = batch::resolve_output_path(file, suffix, &options);
                    io::save_image(flipped, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_flip(&path, output, axis)
            }
        }
        Command::Rotate {
            angle,
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if is_batch(&path, &batch) {
                if preview {
                    return Err("rotate: --preview cannot be used in batch mode".to_string());
                }
                if angle.is_none() {
                    return Err("rotate: --angle required in batch mode".to_string());
                }
                let deg = angle.unwrap();
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("failed to open image '{}': {e}", file.display()))?;
                    let rotated = match deg {
                        90 => img.rotate90(),
                        180 => img.rotate180(),
                        270 => img.rotate270(),
                        _ => return Err(format!("invalid rotation '{deg}': use 90, 180, or 270")),
                    };
                    let suffix = format!("rotate{deg}");
                    let out_path = batch::resolve_output_path(file, &suffix, &options);
                    io::save_image(rotated, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_rotate(&path, output, angle)
            }
        }
        Command::Invert {
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if is_batch(&path, &batch) {
                if preview {
                    return Err("invert: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("failed to open image '{}': {e}", file.display()))?;
                    let inverted = commands::transforms::invert_colors(img);
                    let out_path = batch::resolve_output_path(file, "invert", &options);
                    io::save_image(inverted, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_invert(&path, output)
            }
        }
        Command::Grayscale {
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if is_batch(&path, &batch) {
                if preview {
                    return Err("grayscale: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("failed to open image '{}': {e}", file.display()))?;
                    let gray = img.grayscale();
                    let out_path = batch::resolve_output_path(file, "grayscale", &options);
                    io::save_image(gray, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_grayscale(&path, output)
            }
        }
        Command::Resize {
            width,
            height,
            scale,
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if width.is_some() && scale.is_some() || height.is_some() && scale.is_some() {
                return Err(
                    "resize: --scale cannot be combined with --width or --height".to_string(),
                );
            }
            if is_batch(&path, &batch) {
                if preview {
                    return Err("resize: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let scale = scale;
                if scale.is_none() && (width.is_none() || height.is_none()) {
                    return Err(
                        "resize: --scale or both --width and --height required in batch mode"
                            .to_string(),
                    );
                }
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("failed to open image '{}': {e}", file.display()))?;
                    let (w, h) = if let Some(s) = scale {
                        (
                            (img.width() as f64 * s as f64).round() as u32,
                            (img.height() as f64 * s as f64).round() as u32,
                        )
                    } else {
                        (width.unwrap(), height.unwrap())
                    };
                    let resized =
                        img.resize_exact(w.max(1), h.max(1), image::imageops::FilterType::Lanczos3);
                    let suffix = format!("resize{w}x{h}");
                    let out_path = batch::resolve_output_path(file, &suffix, &options);
                    io::save_image(resized, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_resize(&path, output, width, height, scale)
            }
        }
        Command::Convert {
            format,
            batch,
            src,
            dst,
        } => {
            if is_batch(&src, &batch) {
                let fmt = format.ok_or_else(|| {
                    "convert: --format required in batch mode (e.g. --format png)".to_string()
                })?;
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&src), &options, |file| {
                    let out_path = batch::resolve_output_path_with_ext(file, &fmt, &options);
                    let out_str = out_path.to_string_lossy().to_string();
                    let src_str = file.to_string_lossy().to_string();
                    commands::convert::run_convert(&src_str, &out_str)?;
                    Ok(out_str)
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let dst = dst.ok_or_else(|| {
                    "convert: output path required (e.g. simply convert input.png output.jpg)"
                        .to_string()
                })?;
                commands::convert::run_convert(&src, &dst)
            }
        }
        Command::Vectorize {
            fast,
            preview,
            batch,
            src,
            dst,
        } => {
            if is_batch(&src, &batch) {
                if preview {
                    return Err("vectorize: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&src), &options, |file| {
                    let out_path = batch::resolve_output_path_with_ext(file, "svg", &options);
                    let out_str = out_path.to_string_lossy().to_string();
                    let src_str = file.to_string_lossy().to_string();
                    commands::convert::run_vectorize(VectorizeArgs {
                        src: src_str,
                        dst: out_str.clone(),
                        fast,
                        preview: false,
                    })?;
                    Ok(out_str)
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let dst = dst.unwrap_or_else(|| {
                    Path::new(&src)
                        .with_extension("svg")
                        .to_string_lossy()
                        .to_string()
                });
                commands::convert::run_vectorize(VectorizeArgs { src, dst, fast, preview })
            }
        }
        Command::View { path } => commands::view::run_view(&path),
        Command::Rasterize {
            scale,
            width,
            height,
            preview,
            batch,
            src,
            dst,
        } => {
            if is_batch(&src, &batch) {
                if preview {
                    return Err("rasterize: --preview cannot be used in batch mode".to_string());
                }
                let raster_opts = RasterizeOptions {
                    scale,
                    width,
                    height,
                };
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch_svg(Path::new(&src), &options, |file| {
                    let out_path =
                        batch::resolve_output_path_with_ext(file, "png", &options);
                    let out_str = out_path.to_string_lossy().to_string();
                    let src_str = file.to_string_lossy().to_string();
                    commands::convert::run_rasterize(RasterizeArgs {
                        options: raster_opts,
                        src: src_str,
                        dst: out_str.clone(),
                        preview: false,
                    })?;
                    Ok(out_str)
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let dst = dst.unwrap_or_else(|| {
                    Path::new(&src)
                        .with_extension("png")
                        .to_string_lossy()
                        .to_string()
                });
                commands::convert::run_rasterize(RasterizeArgs {
                    options: RasterizeOptions {
                        scale,
                        width,
                        height,
                    },
                    src,
                    dst,
                    preview,
                })
            }
        }
    }
}

fn is_batch(path: &str, batch: &BatchArgs) -> bool {
    Path::new(path).is_dir()
        || batch.pattern.is_some()
        || batch.output_dir.is_some()
        || batch.recursive
}

fn output_mode(replace: bool, preview: bool, output: Option<String>) -> OutputMode {
    if preview {
        OutputMode::Preview
    } else if replace {
        OutputMode::Replace(output)
    } else {
        match output {
            Some(path) => OutputMode::Explicit(path),
            None => OutputMode::Generated,
        }
    }
}
