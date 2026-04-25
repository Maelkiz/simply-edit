mod cli;
mod commands;
mod io;

use clap::Parser;

use cli::{Cli, Command};

enum OutputMode<'a> {
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
            path,
            output,
        } => {
            let axis = cli::flip_axis_from_flags(horizontal, vertical)?;
            let output = output_mode(replace, output, "flip");
            commands::transforms::run_flip(&path, output, axis)
        }
        Command::Rotate {
            angle,
            replace,
            path,
            output,
        } => {
            let output = output_mode(replace, output, "rotate");
            commands::transforms::run_rotate(&path, output, angle)
        }
        Command::Invert {
            replace,
            path,
            output,
        } => {
            let output = output_mode(replace, output, "invert");
            commands::transforms::run_invert(&path, output)
        }
        Command::Grayscale {
            replace,
            path,
            output,
        } => {
            let output = output_mode(replace, output, "grayscale");
            commands::transforms::run_grayscale(&path, output)
        }
        Command::Convert { src, dst } => {
            commands::convert::run_convert(&[src, dst])
        }
        Command::Vectorize { fast, src, dst } => {
            let mut args = Vec::new();
            if fast {
                args.push("--fast".to_string());
            }
            args.push(src);
            if let Some(d) = dst {
                args.push(d);
            }
            commands::convert::run_vectorize(&args)
        }
        Command::Rasterize {
            scale,
            width,
            height,
            src,
            dst,
        } => {
            let mut args = Vec::new();
            if let Some(s) = scale {
                args.push("-s".to_string());
                args.push(s.to_string());
            }
            if let Some(w) = width {
                args.push("-w".to_string());
                args.push(w.to_string());
            }
            if let Some(h) = height {
                args.push("-h".to_string());
                args.push(h.to_string());
            }
            args.push(src);
            if let Some(d) = dst {
                args.push(d);
            }
            commands::convert::run_rasterize(&args)
        }
    }
}

fn output_mode<'a>(replace: bool, output: Option<String>, suffix: &'a str) -> OutputMode<'a> {
    if replace {
        OutputMode::Replace(output)
    } else {
        match output {
            Some(path) => OutputMode::Explicit(path),
            None => OutputMode::Generated(suffix),
        }
    }
}
