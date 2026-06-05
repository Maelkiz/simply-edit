mod batch;
mod cli;
mod commands;
mod io;
mod preview;

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
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if is_batch(&path, &batch) {
                if preview {
                    return Err("flip: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("flip: failed to open image '{}': {e}", file.display()))?;
                    let flipped = img.flipv();
                    let out_path = batch::resolve_output_path(file, "flipv", &options);
                    io::save_image(flipped, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_flip(&path, output, commands::transforms::FlipAxis::Vertical)
            }
        }
        Command::Flop {
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if is_batch(&path, &batch) {
                if preview {
                    return Err("flop: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("flop: failed to open image '{}': {e}", file.display()))?;
                    let flopped = img.fliph();
                    let out_path = batch::resolve_output_path(file, "fliph", &options);
                    io::save_image(flopped, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_flip(&path, output, commands::transforms::FlipAxis::Horizontal)
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
                        .map_err(|e| format!("rotate: failed to open image '{}': {e}", file.display()))?;
                    let rotated = match deg {
                        90 => img.rotate90(),
                        180 => img.rotate180(),
                        270 => img.rotate270(),
                        _ => return Err(format!("rotate: invalid rotation '{deg}': use 90, 180, or 270")),
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
                if angle.is_none() {
                    commands::transforms::interactive_rotate(&path, output)
                } else {
                    commands::transforms::run_rotate(&path, output, angle)
                }
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
                        .map_err(|e| format!("invert: failed to open image '{}': {e}", file.display()))?;
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
                        .map_err(|e| format!("grayscale: failed to open image '{}': {e}", file.display()))?;
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
        Command::Binarize {
            threshold,
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            if threshold.is_none() && !is_batch(&path, &batch) {
                let output = output_mode(replace, preview, output);
                return commands::transforms::interactive_binarize(&path, output);
            }
            let threshold = threshold.unwrap_or(128);
            if is_batch(&path, &batch) {
                if preview {
                    return Err("binarize: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("binarize: failed to open image '{}': {e}", file.display()))?;
                    let binarized = commands::transforms::binarize_image(img, threshold);
                    let out_path = batch::resolve_output_path(file, "binarize", &options);
                    io::save_image(binarized, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_binarize(&path, output, threshold)
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
                        "resize: batch mode requires --scale or both --width and --height \
                         (single-dimension aspect-ratio resize is only available interactively)"
                            .to_string(),
                    );
                }
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("resize: failed to open image '{}': {e}", file.display()))?;
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
                let dst = match dst {
                    Some(d) => d,
                    None => {
                        let fmt = commands::convert::prompt_convert_format(&src)?;
                        let src_path = Path::new(&src);
                        let stem = src_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        let parent = src_path.parent().unwrap_or(Path::new("."));
                        parent.join(format!("{stem}.{fmt}")).to_string_lossy().to_string()
                    }
                };
                commands::convert::run_convert(&src, &dst)
            }
        }
        Command::Vectorize {
            fast,
            full_quality,
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
                        full_quality,
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
                commands::convert::run_vectorize(VectorizeArgs { src, dst, fast, full_quality, preview })
            }
        }
        Command::Pad {
            top,
            bottom,
            left,
            right,
            px,
            horizontal,
            vertical,
            color,
            replace,
            preview,
            batch,
            path,
            output,
        } => {
            let any_size_flag = top.is_some()
                || bottom.is_some()
                || left.is_some()
                || right.is_some()
                || horizontal.is_some()
                || vertical.is_some()
                || px.is_some();
            let fallback = if any_size_flag { 0 } else { 20 };
            let top = top.unwrap_or_else(|| vertical.unwrap_or_else(|| px.unwrap_or(fallback)));
            let bottom = bottom.unwrap_or_else(|| vertical.unwrap_or_else(|| px.unwrap_or(fallback)));
            let left = left.unwrap_or_else(|| horizontal.unwrap_or_else(|| px.unwrap_or(fallback)));
            let right = right.unwrap_or_else(|| horizontal.unwrap_or_else(|| px.unwrap_or(fallback)));
            let color = image::Rgba(color.unwrap_or([0, 0, 0, 0]));
            if is_batch(&path, &batch) {
                if preview {
                    return Err("pad: --preview cannot be used in batch mode".to_string());
                }
                let options = batch::to_batch_options(&batch)?;
                let result = batch::run_batch(Path::new(&path), &options, |file| {
                    let img = image::open(file)
                        .map_err(|e| format!("pad: failed to open image '{}': {e}", file.display()))?;
                    let padded = commands::transforms::pad_image(img, top, right, bottom, left, color);
                    let out_path = batch::resolve_output_path(file, "pad", &options);
                    io::save_image(padded, &out_path)?;
                    Ok(out_path.to_string_lossy().to_string())
                })?;
                batch::print_summary(&result);
                Ok(())
            } else {
                let output = output_mode(replace, preview, output);
                commands::transforms::run_pad(&path, output, top, right, bottom, left, color)
            }
        }
        Command::Info { path } => commands::info::run_info(&path),
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
