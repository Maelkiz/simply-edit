use std::fs;

use crate::common::{TestDir, create_png, create_svg, run, run_with_stdin, stderr};

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
fn test_resize_missing_path_prints_usage() {
    let output = run(&["resize"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_resize_unknown_flag_rejected() {
    let output = run(&["resize", "--foo", "image.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--foo"));
}

#[test]
fn test_resize_zero_width_rejected() {
    let output = run(&["resize", "--width", "0", "-H", "100", "image.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("--width"));
}

#[test]
fn test_resize_zero_height_rejected() {
    let output = run(&["resize", "--width", "100", "-H", "0", "image.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("height"));
}

#[test]
fn test_resize_invalid_scale_rejected() {
    let output = run(&["resize", "--scale", "abc", "image.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("abc"));
}

#[test]
fn test_resize_scale_and_width_rejected() {
    let temp = TestDir::new("simply-resize-err");
    let input = temp.path().join("img.png");
    create_png(&input, 4, 4, [255, 0, 0, 255]);

    let output = run(&["resize", "--scale", "2", "--width", "100", input.to_str().expect("valid input path")]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--scale cannot be combined"));
}

#[test]
fn test_resize_scale_and_height_rejected() {
    let temp = TestDir::new("simply-resize-err");
    let input = temp.path().join("img.png");
    create_png(&input, 4, 4, [255, 0, 0, 255]);

    let output = run(&["resize", "--scale", "2", "-H", "100", input.to_str().expect("valid input path")]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--scale cannot be combined"));
}

#[test]
fn test_resize_invalid_mode_input_rejected() {
    let temp = TestDir::new("simply-resize-err");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 12, 6, [100, 100, 100, 255]);

    // "3" is not a valid mode (only 1 or 2)
    let output = run_with_stdin(
        &["resize", "--width", "24", input.to_str().expect("valid input path"), out.to_str().expect("valid output path")],
        "3\n",
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid resize mode"));
}

#[test]
fn test_preview_rejected_in_batch_flip() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["flip", "--horizontal", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_rotate() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["rotate", "--angle", "90", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_invert() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["invert", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_grayscale() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["grayscale", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_resize() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["resize", "--scale", "2", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_rasterize() {
    let temp = TestDir::new("simply-preview-batch-err");
    create_svg(&temp.path().join("a.svg"), 4, 4, "#ff0000");
    let output = run(&["rasterize", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_vectorize() {
    let temp = TestDir::new("simply-preview-batch-err");
    create_png(&temp.path().join("a.png"), 4, 4, [255, 0, 0, 255]);
    let output = run(&["vectorize", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_view_nonexistent_file_fails() {
    let output = run(&["view", "/no/such/file.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open"));
}

#[test]
fn test_view_non_image_file_fails() {
    let temp = TestDir::new("simply-view-err");
    let bad = temp.path().join("data.txt");
    fs::write(&bad, b"not an image").expect("failed to write file");

    let output = run(&["view", bad.to_str().expect("valid path")]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open"));
}

#[test]
fn test_binarize_missing_path_prints_usage() {
    let output = run(&["binarize"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_binarize_nonexistent_file_fails() {
    let output = run(&["binarize", "this/path/does/not/exist.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open image"));
}

#[test]
fn test_binarize_unknown_flag_rejected() {
    let temp = TestDir::new("simply-binarize-err");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&["binarize", "--foo", input.to_str().expect("valid input path")]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--foo"));
}

#[test]
fn test_preview_rejected_in_batch_binarize() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["binarize", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
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
