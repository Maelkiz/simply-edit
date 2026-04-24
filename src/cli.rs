#[derive(Debug, Clone)]
pub(crate) enum ParsedCommand {
    Help,
    CommandHelp(String),
    Flip {
        path: String,
        output: ParsedOutput,
        axis: ParsedFlipAxis,
    },
    Rotate {
        mode: ParsedRotateMode,
        path: String,
        output: ParsedOutput,
    },
    Invert {
        path: String,
        output: ParsedOutput,
    },
    Grayscale {
        path: String,
        output: ParsedOutput,
    },
    Convert {
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum ParsedOutput {
    Generated,
    Explicit(String),
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParsedFlipAxis {
    Prompt,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParsedRotateMode {
    Prompt,
    Explicit(u16),
}

pub(crate) fn parse_command(args: &[String]) -> Result<ParsedCommand, String> {
    match args {
        [_, command] if matches!(command.as_str(), "help" | "--help" | "-h") => {
            Ok(ParsedCommand::Help)
        }
        [_, command, rest @ ..] if command == "flip" => {
            if rest.iter().any(|a| a == "--help") {
                return Ok(ParsedCommand::CommandHelp("flip".to_string()));
            }
            parse_flip_command(rest)
        }

        [_, command, rest @ ..] if command == "rotate" => {
            if rest.iter().any(|a| a == "--help") {
                return Ok(ParsedCommand::CommandHelp("rotate".to_string()));
            }
            parse_rotate_command(rest)
        }

        [_, command, rest @ ..] if command == "invert" && rest.iter().any(|a| a == "--help") => {
            Ok(ParsedCommand::CommandHelp("invert".to_string()))
        }
        [_, command, flag, path] if command == "invert" && crate::io::is_replace_flag(flag) => {
            Ok(ParsedCommand::Invert {
                path: path.clone(),
                output: ParsedOutput::Replace,
            })
        }
        [_, command, path] if command == "invert" => Ok(ParsedCommand::Invert {
            path: path.clone(),
            output: ParsedOutput::Generated,
        }),
        [_, command, path, output] if command == "invert" => Ok(ParsedCommand::Invert {
            path: path.clone(),
            output: ParsedOutput::Explicit(output.clone()),
        }),

        [_, command, rest @ ..] if command == "grayscale" && rest.iter().any(|a| a == "--help") => {
            Ok(ParsedCommand::CommandHelp("grayscale".to_string()))
        }
        [_, command, flag, path] if command == "grayscale" && crate::io::is_replace_flag(flag) => {
            Ok(ParsedCommand::Grayscale {
                path: path.clone(),
                output: ParsedOutput::Replace,
            })
        }
        [_, command, path] if command == "grayscale" => Ok(ParsedCommand::Grayscale {
            path: path.clone(),
            output: ParsedOutput::Generated,
        }),
        [_, command, path, output] if command == "grayscale" => Ok(ParsedCommand::Grayscale {
            path: path.clone(),
            output: ParsedOutput::Explicit(output.clone()),
        }),

        [_, command, rest @ ..] if command == "convert" && rest.iter().any(|a| a == "--help") => {
            Ok(ParsedCommand::CommandHelp("convert".to_string()))
        }
        [_, command, rest @ ..] if command == "convert" => Ok(ParsedCommand::Convert {
            args: rest.to_vec(),
        }),
        _ => Err(usage()),
    }
}

pub(crate) fn usage() -> String {
    [
        "",
        "simply-edit",
        "",
        "Usage:",
        "  simply <command> <args>",
        "",
        "For details on a specific command, run:",
        "  simply <command> --help",
        "",
        "Commands:",
        "  simply flip <args>",
        "  simply rotate <args>",
        "  simply invert <args>",
        "  simply grayscale <args>",
        "  simply convert <args>",
        "",
    ]
    .join("\n")
}

pub(crate) fn command_usage(command: &str) -> String {
    match command {
        "flip" => [
            "simply flip — Flip an image horizontally or vertically",
            "",
            "Usage:",
            "  simply flip [options] <path-to-image> [output-path]",
            "",
            "Options:",
            "  --horizontal    Flip horizontally (bypasses interactive prompt)",
            "  --vertical      Flip vertically (bypasses interactive prompt)",
            "  -r, --replace   Replace the source file instead of writing a new one",
            "",
            "Without --horizontal or --vertical, an interactive prompt lets you choose.",
            "If no output path is given, one is generated automatically (e.g. image_fliph.png).",
        ]
        .join("\n"),
        "rotate" => [
            "simply rotate — Rotate an image by 90, 180, or 270 degrees",
            "",
            "Usage:",
            "  simply rotate [90|180|270] [options] <path-to-image> [output-path]",
            "",
            "Options:",
            "  -r, --replace   Replace the source file instead of writing a new one",
            "",
            "Without a degree argument, an interactive prompt lets you choose.",
            "If no output path is given, one is generated automatically (e.g. image_rotate.png).",
        ]
        .join("\n"),
        "invert" => [
            "simply invert — Invert the colors of an image",
            "",
            "Usage:",
            "  simply invert [options] <path-to-image> [output-path]",
            "",
            "Options:",
            "  -r, --replace   Replace the source file instead of writing a new one",
            "",
            "If no output path is given, one is generated automatically (e.g. image_invert.png).",
        ]
        .join("\n"),
        "grayscale" => [
            "simply grayscale — Convert an image to grayscale",
            "",
            "Usage:",
            "  simply grayscale [options] <path-to-image> [output-path]",
            "",
            "Options:",
            "  -r, --replace   Replace the source file instead of writing a new one",
            "",
            "If no output path is given, one is generated automatically (e.g. image_grayscale.png).",
        ]
        .join("\n"),
        "convert" => [
            "simply convert — Convert between image formats (PNG, JPG, ICO, SVG)",
            "",
            "Usage:",
            "  simply convert [options] <path-to-image> <output-path>",
            "",
            "Options (SVG input to raster output only):",
            "  -s, --scale <factor>   Scale factor for SVG rasterization",
            "  -w, --width <px>       Output width in pixels",
            "  -h, --height <px>      Output height in pixels",
            "",
            "The output format is determined by the output path extension.",
        ]
        .join("\n"),
        _ => usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_contains_all_commands() {
        let usage_text = usage();
        assert!(usage_text.contains("simply flip"));
        assert!(usage_text.contains("simply rotate"));
        assert!(usage_text.contains("simply invert"));
        assert!(usage_text.contains("simply grayscale"));
        assert!(usage_text.contains("simply convert"));
    }

    #[test]
    fn test_usage_mentions_command_help() {
        let usage_text = usage();
        assert!(usage_text.contains("--help"));
    }

    #[test]
    fn test_usage_is_non_empty() {
        let usage_text = usage();
        assert!(!usage_text.is_empty());
    }

    #[test]
    fn test_command_help_flip() {
        let args = vec!["simply".into(), "flip".into(), "--help".into()];
        match parse_command(&args).unwrap() {
            ParsedCommand::CommandHelp(cmd) => assert_eq!(cmd, "flip"),
            _ => panic!("expected CommandHelp"),
        }
    }

    #[test]
    fn test_command_help_rotate() {
        let args = vec!["simply".into(), "rotate".into(), "--help".into()];
        match parse_command(&args).unwrap() {
            ParsedCommand::CommandHelp(cmd) => assert_eq!(cmd, "rotate"),
            _ => panic!("expected CommandHelp"),
        }
    }

    #[test]
    fn test_command_help_invert() {
        let args = vec!["simply".into(), "invert".into(), "--help".into()];
        match parse_command(&args).unwrap() {
            ParsedCommand::CommandHelp(cmd) => assert_eq!(cmd, "invert"),
            _ => panic!("expected CommandHelp"),
        }
    }

    #[test]
    fn test_command_help_grayscale() {
        let args = vec!["simply".into(), "grayscale".into(), "--help".into()];
        match parse_command(&args).unwrap() {
            ParsedCommand::CommandHelp(cmd) => assert_eq!(cmd, "grayscale"),
            _ => panic!("expected CommandHelp"),
        }
    }

    #[test]
    fn test_command_help_convert() {
        let args = vec!["simply".into(), "convert".into(), "--help".into()];
        match parse_command(&args).unwrap() {
            ParsedCommand::CommandHelp(cmd) => assert_eq!(cmd, "convert"),
            _ => panic!("expected CommandHelp"),
        }
    }

    #[test]
    fn test_command_usage_contains_flags() {
        assert!(command_usage("flip").contains("--horizontal"));
        assert!(command_usage("flip").contains("--vertical"));
        assert!(command_usage("flip").contains("--replace"));
        assert!(command_usage("rotate").contains("90|180|270"));
        assert!(command_usage("rotate").contains("--replace"));
        assert!(command_usage("invert").contains("--replace"));
        assert!(command_usage("grayscale").contains("--replace"));
        assert!(command_usage("convert").contains("--scale"));
        assert!(command_usage("convert").contains("--width"));
        assert!(command_usage("convert").contains("--height"));
    }

    #[test]
    fn test_parse_command_flip_replace() {
        let args = vec![
            "simply".to_string(),
            "flip".to_string(),
            "--replace".to_string(),
            "image.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse flip command");
        match parsed {
            ParsedCommand::Flip {
                path,
                output: ParsedOutput::Replace,
                axis: ParsedFlipAxis::Prompt,
            } => {
                assert_eq!(path, "image.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }

    #[test]
    fn test_parse_command_flip_horizontal_explicit_output() {
        let args = vec![
            "simply".to_string(),
            "flip".to_string(),
            "--horizontal".to_string(),
            "image.png".to_string(),
            "out.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse horizontal flip command");
        match parsed {
            ParsedCommand::Flip {
                path,
                output: ParsedOutput::Explicit(output),
                axis: ParsedFlipAxis::Horizontal,
            } => {
                assert_eq!(path, "image.png");
                assert_eq!(output, "out.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }

    #[test]
    fn test_parse_command_flip_vertical_replace_order_independent() {
        let args = vec![
            "simply".to_string(),
            "flip".to_string(),
            "--replace".to_string(),
            "--vertical".to_string(),
            "image.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse vertical replace flip command");
        match parsed {
            ParsedCommand::Flip {
                path,
                output: ParsedOutput::Replace,
                axis: ParsedFlipAxis::Vertical,
            } => {
                assert_eq!(path, "image.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }

    #[test]
    fn test_parse_command_flip_rejects_conflicting_axis_flags() {
        let args = vec![
            "simply".to_string(),
            "flip".to_string(),
            "--horizontal".to_string(),
            "--vertical".to_string(),
            "image.png".to_string(),
        ];

        let err = parse_command(&args).expect_err("expected conflicting axis flags to fail");
        assert!(err.contains("choose only one"));
    }

    #[test]
    fn test_parse_command_convert_collects_rest_args() {
        let args = vec![
            "simply".to_string(),
            "convert".to_string(),
            "-s".to_string(),
            "2".to_string(),
            "in.svg".to_string(),
            "out.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse convert command");
        match parsed {
            ParsedCommand::Convert { args } => {
                assert_eq!(args, vec!["-s", "2", "in.svg", "out.png"]);
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }

    #[test]
    fn test_parse_command_help_variants() {
        for arg in ["help", "--help", "-h"] {
            let args = vec!["simply".to_string(), arg.to_string()];
            let parsed = parse_command(&args).expect("failed to parse help command");

            match parsed {
                ParsedCommand::Help => {}
                _ => panic!("expected help command variant"),
            }
        }
    }

    #[test]
    fn test_parse_command_rotate_prompt_generated() {
        let args = vec![
            "simply".to_string(),
            "rotate".to_string(),
            "image.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse interactive rotate command");
        match parsed {
            ParsedCommand::Rotate {
                mode: ParsedRotateMode::Prompt,
                path,
                output: ParsedOutput::Generated,
            } => {
                assert_eq!(path, "image.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }

    #[test]
    fn test_parse_command_rotate_explicit_generated() {
        let args = vec![
            "simply".to_string(),
            "rotate".to_string(),
            "90".to_string(),
            "image.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse explicit rotate command");
        match parsed {
            ParsedCommand::Rotate {
                mode: ParsedRotateMode::Explicit(degrees),
                path,
                output: ParsedOutput::Generated,
            } => {
                assert_eq!(degrees, 90);
                assert_eq!(path, "image.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
    }
}

fn parse_flip_command(rest: &[String]) -> Result<ParsedCommand, String> {
    let mut replace = false;
    let mut axis = ParsedFlipAxis::Prompt;
    let mut positionals: Vec<String> = Vec::new();

    for arg in rest {
        if crate::io::is_replace_flag(arg) {
            replace = true;
            continue;
        }

        match arg.as_str() {
            "--horizontal" => {
                if axis == ParsedFlipAxis::Vertical {
                    return Err("flip: choose only one of --horizontal or --vertical".to_string());
                }
                axis = ParsedFlipAxis::Horizontal;
            }
            "--vertical" => {
                if axis == ParsedFlipAxis::Horizontal {
                    return Err("flip: choose only one of --horizontal or --vertical".to_string());
                }
                axis = ParsedFlipAxis::Vertical;
            }
            _ if arg.starts_with('-') => {
                return Err(format!(
                    "flip: unrecognized flag '{arg}' (supported: --horizontal, --vertical, -r, --replace)"
                ));
            }
            _ => positionals.push(arg.clone()),
        }
    }

    if positionals.is_empty() || positionals.len() > 2 {
        return Err(usage());
    }

    let output = if replace {
        if positionals.len() != 1 {
            return Err("flip: output-path is not allowed with -r/--replace".to_string());
        }
        ParsedOutput::Replace
    } else {
        match positionals.len() {
            1 => ParsedOutput::Generated,
            2 => ParsedOutput::Explicit(positionals[1].clone()),
            _ => return Err(usage()),
        }
    };

    Ok(ParsedCommand::Flip {
        path: positionals[0].clone(),
        output,
        axis,
    })
}

fn parse_rotate_command(rest: &[String]) -> Result<ParsedCommand, String> {
    let mut replace = false;
    let mut positionals: Vec<String> = Vec::new();

    for arg in rest {
        if crate::io::is_replace_flag(arg) {
            replace = true;
            continue;
        }

        if arg.starts_with('-') {
            return Err(format!(
                "rotate: unrecognized flag '{arg}' (supported: -r, --replace)"
            ));
        }

        positionals.push(arg.clone());
    }

    if positionals.is_empty() {
        return Err(usage());
    }

    let (mode, path, output_path) = match positionals.len() {
        1 => {
            if parse_supported_rotate_degree(&positionals[0]).is_some() {
                return Err(usage());
            }
            (ParsedRotateMode::Prompt, positionals[0].clone(), None)
        }
        2 => {
            if let Some(deg) = parse_supported_rotate_degree(&positionals[0]) {
                (ParsedRotateMode::Explicit(deg), positionals[1].clone(), None)
            } else {
                (
                    ParsedRotateMode::Prompt,
                    positionals[0].clone(),
                    Some(positionals[1].clone()),
                )
            }
        }
        3 => {
            let deg = parse_supported_rotate_degree(&positionals[0]).ok_or_else(|| {
                format!(
                    "invalid rotation '{}': use 90, 180, or 270",
                    positionals[0]
                )
            })?;
            (
                ParsedRotateMode::Explicit(deg),
                positionals[1].clone(),
                Some(positionals[2].clone()),
            )
        }
        _ => return Err(usage()),
    };

    let output = if replace {
        if output_path.is_some() {
            return Err("rotate: output-path is not allowed with -r/--replace".to_string());
        }
        ParsedOutput::Replace
    } else {
        match output_path {
            Some(path) => ParsedOutput::Explicit(path),
            None => ParsedOutput::Generated,
        }
    };

    Ok(ParsedCommand::Rotate { mode, path, output })
}

fn parse_supported_rotate_degree(value: &str) -> Option<u16> {
    match value {
        "90" => Some(90),
        "180" => Some(180),
        "270" => Some(270),
        _ => None,
    }
}
