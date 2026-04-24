mod cli;
mod commands;
mod io;

use std::env;

enum OutputMode<'a> {
    Generated(&'a str),
    Explicit(String),
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
    match cli::parse_command(&args)? {
        cli::ParsedCommand::Help => {
            println!("{}", usage());
            Ok(())
        }
        cli::ParsedCommand::Flip { path, output } => {
            commands::transforms::run_flip(&path, to_output_mode(output, "flip"))
        }
        cli::ParsedCommand::Rotate {
            degrees,
            path,
            output,
        } => commands::transforms::run_rotate(&degrees, &path, to_output_mode(output, "rotate")),
        cli::ParsedCommand::Invert { path, output } => {
            commands::transforms::run_invert(&path, to_output_mode(output, "invert"))
        }
        cli::ParsedCommand::Grayscale { path, output } => {
            commands::transforms::run_grayscale(&path, to_output_mode(output, "grayscale"))
        }
        cli::ParsedCommand::Convert { args } => commands::convert::run_convert(&args),
    }
}

fn usage() -> String {
    cli::usage()
}

fn to_output_mode<'a>(output: cli::ParsedOutput, generated_suffix: &'a str) -> OutputMode<'a> {
    match output {
        cli::ParsedOutput::Generated => OutputMode::Generated(generated_suffix),
        cli::ParsedOutput::Explicit(path) => OutputMode::Explicit(path),
        cli::ParsedOutput::Replace => OutputMode::Replace,
    }
}
