use std::fs;

use crate::common::{TestDir, create_png, run, run_with_stdin, stderr};

#[test]
fn test_no_args_prints_usage() {
    let output = run(&[]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_unknown_command_prints_usage() {
    let output = run(&["unknown", "image.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_flip_missing_path_prints_usage() {
    let output = run(&["flip"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_flip_empty_non_tty_input_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run_with_stdin(&["flip", input.to_str().expect("valid input path")], "\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid flip direction"));
}

#[test]
fn test_flip_text_non_tty_input_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run_with_stdin(
        &["flip", input.to_str().expect("valid input path")],
        "horizontal\n",
    );

    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid flip direction"));
}

#[test]
fn test_flip_conflicting_axis_flags_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "flip",
        "--horizontal",
        "--vertical",
        input.to_str().expect("valid input path"),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("choose only one"));
}

#[test]
fn test_flip_unknown_flag_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&["flip", "--fast", input.to_str().expect("valid input path")]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--fast"));
}

#[test]
fn test_rotate_missing_path_prints_usage() {
    let output = run(&["rotate"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_rotate_invalid_degrees_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "rotate",
        "--angle",
        "45",
        input.to_str().expect("valid input path"),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid rotation '45'"));
}

#[test]
fn test_rotate_non_numeric_degrees_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "rotate",
        "--angle",
        "abc",
        input.to_str().expect("valid input path"),
    ]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid rotation 'abc'"));
}

#[test]
fn test_rotate_interactive_non_tty_invalid_input_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run_with_stdin(
        &["rotate", input.to_str().expect("valid input path")],
        "45\n",
    );

    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid rotation '45'"));
}

#[test]
fn test_rasterize_missing_value_for_scale_rejected() {
    let output = run(&["rasterize", "-s"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("--scale") || err.contains("-s"));
}

#[test]
fn test_rasterize_missing_value_for_width_rejected() {
    let output = run(&["rasterize", "-w"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("--width") || err.contains("-w"));
}

#[test]
fn test_rasterize_invalid_scale_rejected() {
    let output = run(&["rasterize", "-s", "abc", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("abc"));
}

#[test]
fn test_rasterize_zero_width_rejected() {
    let output = run(&["rasterize", "-w", "0", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("--width"));
}

#[test]
fn test_rasterize_negative_scale_rejected() {
    let output = run(&["rasterize", "-s", "-5", "in.svg", "out.png"]);
    assert!(!output.status.success());
}

#[test]
fn test_rasterize_unknown_flag_rejected() {
    let output = run(&["rasterize", "--unknown", "in.svg", "out.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--unknown"));
}

#[test]
fn test_convert_unknown_flag_rejected() {
    let output = run(&["convert", "--unknown", "in.svg", "out.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--unknown"));
}

#[test]
fn test_vectorize_unknown_flag_rejected() {
    let output = run(&["vectorize", "--unknown", "in.png", "out.svg"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--unknown"));
}

#[test]
fn test_nonexistent_input_file_fails() {
    let output = run(&["invert", "this/path/does/not/exist.png", "out.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open image"));
}

#[test]
fn test_invalid_image_file_fails() {
    let temp = TestDir::new("simply-phase1-errors");
    let bad = temp.path().join("bad.png");
    let out = temp.path().join("out.png");
    fs::write(&bad, b"not a png").expect("failed to write invalid input file");

    let output = run(&[
        "invert",
        bad.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open image"));
}

#[test]
fn test_unsupported_output_format_rejected() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("input.png");
    let out = temp.path().join("output.bmp");
    create_png(&input, 2, 2, [10, 20, 30, 255]);

    let output = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported format 'bmp'"));
}

#[test]
fn test_convert_invalid_svg_parse_fails() {
    let temp = TestDir::new("simply-phase1-errors");
    let input = temp.path().join("bad.svg");
    let out = temp.path().join("out.png");
    fs::write(&input, "<svg><broken></svg>").expect("failed to write invalid svg");

    let output = run(&[
        "convert",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to parse SVG"));
}
