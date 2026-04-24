#[derive(Debug, Clone)]
pub(crate) enum ParsedCommand {
    Help,
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
        [_, command, rest @ ..] if command == "flip" => parse_flip_command(rest),

        [_, command, rest @ ..] if command == "rotate" => parse_rotate_command(rest),

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

        [_, command, rest @ ..] if command == "convert" => Ok(ParsedCommand::Convert {
            args: rest.to_vec(),
        }),
        _ => Err(usage()),
    }
}

pub(crate) fn usage() -> String {
    [
        "simply-edit",
        "",
        "Usage:",
        "  simply --help",
        "  simply flip [--horizontal|--vertical] [-r|--replace] <path-to-image> [output-path]",
        "  simply rotate [90|180|270] [-r|--replace] <path-to-image> [output-path]",
        "  simply invert [-r|--replace] <path-to-image> [output-path]",
        "  simply grayscale [-r|--replace] <path-to-image> [output-path]",
        "  simply convert [-s|--scale <factor>] [-w|--width <px>] [-h|--height <px>] <path-to-image> <new-path>",
        "",
        "Notes:",
        "  rotate supports explicit degrees: 90, 180, 270",
        "  rotate without a degree starts an interactive degree prompt",
        "  convert -s/-w/-h are only supported for SVG input converted to raster output",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_contains_all_commands() {
        let usage_text = usage();
        assert!(usage_text.contains("simply --help"));
        assert!(usage_text.contains("flip"));
        assert!(usage_text.contains("rotate"));
        assert!(usage_text.contains("invert"));
        assert!(usage_text.contains("grayscale"));
        assert!(usage_text.contains("convert"));
    }

    #[test]
    fn test_usage_contains_replace_flag_info() {
        let usage_text = usage();
        assert!(usage_text.contains("-r|--replace"));
    }

    #[test]
    fn test_usage_is_non_empty() {
        let usage_text = usage();
        assert!(!usage_text.is_empty());
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
