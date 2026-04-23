#[derive(Debug, Clone)]
pub(crate) enum ParsedCommand {
    Help,
    Fliph {
        path: String,
        output: ParsedOutput,
    },
    Flipv {
        path: String,
        output: ParsedOutput,
    },
    Rotate {
        degrees: String,
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

pub(crate) fn parse_command(args: &[String]) -> Result<ParsedCommand, String> {
    match args {
        [_, command] if matches!(command.as_str(), "help" | "--help" | "-h") => {
            Ok(ParsedCommand::Help)
        }
        [_, command, flag, path] if command == "fliph" && crate::io::is_replace_flag(flag) => {
            Ok(ParsedCommand::Fliph {
                path: path.clone(),
                output: ParsedOutput::Replace,
            })
        }
        [_, command, path] if command == "fliph" => Ok(ParsedCommand::Fliph {
            path: path.clone(),
            output: ParsedOutput::Generated,
        }),
        [_, command, path, output] if command == "fliph" => Ok(ParsedCommand::Fliph {
            path: path.clone(),
            output: ParsedOutput::Explicit(output.clone()),
        }),

        [_, command, flag, path] if command == "flipv" && crate::io::is_replace_flag(flag) => {
            Ok(ParsedCommand::Flipv {
                path: path.clone(),
                output: ParsedOutput::Replace,
            })
        }
        [_, command, path] if command == "flipv" => Ok(ParsedCommand::Flipv {
            path: path.clone(),
            output: ParsedOutput::Generated,
        }),
        [_, command, path, output] if command == "flipv" => Ok(ParsedCommand::Flipv {
            path: path.clone(),
            output: ParsedOutput::Explicit(output.clone()),
        }),

        [_, command, degrees, flag, path]
            if command == "rotate" && crate::io::is_replace_flag(flag) =>
        {
            Ok(ParsedCommand::Rotate {
                degrees: degrees.clone(),
                path: path.clone(),
                output: ParsedOutput::Replace,
            })
        }
        [_, command, degrees, path] if command == "rotate" => Ok(ParsedCommand::Rotate {
            degrees: degrees.clone(),
            path: path.clone(),
            output: ParsedOutput::Generated,
        }),
        [_, command, degrees, path, output] if command == "rotate" => Ok(ParsedCommand::Rotate {
            degrees: degrees.clone(),
            path: path.clone(),
            output: ParsedOutput::Explicit(output.clone()),
        }),

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
        "  simply fliph [-r|--replace] <path-to-image> [output-path]",
        "  simply flipv [-r|--replace] <path-to-image> [output-path]",
        "  simply rotate <degrees> [-r|--replace] <path-to-image> [output-path]",
        "  simply invert [-r|--replace] <path-to-image> [output-path]",
        "  simply grayscale [-r|--replace] <path-to-image> [output-path]",
        "  simply convert [-s|--scale <factor>] [-w|--width <px>] [-h|--height <px>] <path-to-image> <new-path>",
        "",
        "Notes:",
        "  rotate <degrees> supports: 90, 180, 270",
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
        assert!(usage_text.contains("fliph"));
        assert!(usage_text.contains("flipv"));
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
    fn test_parse_command_fliph_replace() {
        let args = vec![
            "simply".to_string(),
            "fliph".to_string(),
            "--replace".to_string(),
            "image.png".to_string(),
        ];

        let parsed = parse_command(&args).expect("failed to parse fliph command");
        match parsed {
            ParsedCommand::Fliph {
                path,
                output: ParsedOutput::Replace,
            } => {
                assert_eq!(path, "image.png");
            }
            _ => panic!("unexpected parsed command variant"),
        }
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
}
