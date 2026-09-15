use std::fs;

use crate::common::{TestDir, create_png, create_svg, run, run_with_env, run_with_stdin, stderr};

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
    let output = run(&["rasterize", "--scale"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("--scale"));
}

#[test]
fn test_rasterize_missing_value_for_width_rejected() {
    let output = run(&["rasterize", "--width"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("--width"));
}

#[test]
fn test_rasterize_invalid_scale_rejected() {
    let output = run(&["rasterize", "--scale", "abc", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("abc"));
}

#[test]
fn test_rasterize_zero_width_rejected() {
    let output = run(&["rasterize", "--width", "0", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("--width"));
}

#[test]
fn test_rasterize_negative_scale_rejected() {
    let output = run(&["rasterize", "--scale", "-5", "in.svg", "out.png"]);
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
    let output = run(&["resize", "--width", "0", "--height", "100", "image.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("--width"));
}

#[test]
fn test_resize_zero_height_rejected() {
    let output = run(&["resize", "--width", "100", "--height", "0", "image.png"]);
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("invalid value '0'") && err.contains("height"));
}

#[test]
fn test_scale_invalid_factor_rejected() {
    let output = run(&["scale", "--factor", "abc", "image.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("abc"));
}

#[test]
fn test_resize_scale_flag_unrecognized() {
    let temp = TestDir::new("simply-resize-err");
    let input = temp.path().join("img.png");
    create_png(&input, 4, 4, [255, 0, 0, 255]);

    // --scale is no longer a valid flag on resize; clap should reject it
    let output = run(&[
        "resize",
        "--scale",
        "2",
        input.to_str().expect("valid input path"),
    ]);
    assert!(!output.status.success());
}

#[test]
fn test_resize_invalid_mode_input_rejected() {
    let temp = TestDir::new("simply-resize-err");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 12, 6, [100, 100, 100, 255]);

    // "3" is not a valid mode (only 1 or 2)
    let output = run_with_stdin(
        &[
            "resize",
            "--width",
            "24",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
        "3\n",
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid resize mode"));
}

#[test]
fn test_preview_rejected_in_batch_flip() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&["flip", "--preview", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_preview_rejected_in_batch_rotate() {
    let temp = TestDir::new("simply-preview-batch-err");
    let output = run(&[
        "rotate",
        "--angle",
        "90",
        "--preview",
        temp.path().to_str().unwrap(),
    ]);
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
    let output = run(&[
        "resize",
        "--width",
        "4",
        "--height",
        "4",
        "--preview",
        temp.path().to_str().unwrap(),
    ]);
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

    let output = run(&[
        "binarize",
        "--foo",
        input.to_str().expect("valid input path"),
    ]);
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

#[test]
fn test_cutout_missing_path_prints_usage() {
    let output = run(&["cutout"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_cutout_nonexistent_file_fails() {
    let output = run(&["cutout", "this/path/does/not/exist.png"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to open image"));
}

#[test]
fn test_cutout_rejects_jpeg_destination() {
    let temp = TestDir::new("simply-cutout-dst-err");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        input.to_str().expect("valid input path"),
        temp.path().join("out.jpg").to_str().expect("valid path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("cannot store transparency"));
}

#[test]
fn test_cutout_rejects_replace_on_jpeg_source() {
    let temp = TestDir::new("simply-cutout-replace-err");
    let png = temp.path().join("src.png");
    let jpeg = temp.path().join("photo.jpg");
    create_png(&png, 2, 2, [255, 0, 0, 255]);
    image::open(&png)
        .expect("source png")
        .to_rgb8()
        .save(&jpeg)
        .expect("failed to write jpeg");

    let output = run(&[
        "cutout",
        "--fast",
        "--replace",
        jpeg.to_str().expect("valid path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("only png and webp can store transparency"));
}

#[test]
fn test_cutout_rejects_svg_input() {
    let temp = TestDir::new("simply-cutout-svg-err");
    let input = temp.path().join("shape.svg");
    create_svg(&input, 10, 10, "red");

    let output = run(&["cutout", "--fast", input.to_str().expect("valid path")]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported file format"));
}

#[test]
fn test_cutout_rejects_invalid_tolerance() {
    let temp = TestDir::new("simply-cutout-tol-err");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        "--tolerance",
        "900",
        input.to_str().expect("valid path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("invalid tolerance"));
}

#[test]
fn test_preview_rejected_in_batch_cutout() {
    let temp = TestDir::new("simply-cutout-preview-batch-err");
    let output = run(&[
        "cutout",
        "--fast",
        "--preview",
        temp.path().to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--preview cannot be used in batch mode"));
}

#[test]
fn test_cutout_trim_on_fully_removed_image_errors() {
    let temp = TestDir::new("simply-cutout-trim-err");
    let input = temp.path().join("blank.png");
    create_png(&input, 6, 6, [255, 255, 255, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        "--trim",
        input.to_str().expect("valid path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("empty image"));
}

#[test]
fn test_cutout_download_model_reports_a_cached_model_without_network() {
    let temp = TestDir::new("simply-model-cached");
    fs::write(temp.path().join("u2net.onnx"), b"stand-in weights").expect("write stub model");

    let output = run_with_env(
        &["cutout", "--download-model"],
        &[("SIMPLY_MODEL_DIR", temp.path())],
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("already present"));
}

#[test]
fn test_cutout_download_model_conflicts_with_an_image_path() {
    let temp = TestDir::new("simply-model-conflict");
    let input = temp.path().join("input.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "cutout",
        "--download-model",
        input.to_str().expect("valid path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("cannot be used with"));
}

#[test]
fn test_cutout_requires_a_path_without_download_model() {
    let output = run(&["cutout"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Usage:"));
}

#[test]
fn test_cutout_without_a_cached_model_errors_instead_of_downloading() {
    let temp = TestDir::new("simply-cutout-nomodel");
    let models = TestDir::new("simply-cutout-nomodel-cache");
    let input = temp.path().join("input.png");
    create_png(&input, 4, 4, [255, 255, 255, 255]);

    // No TTY here, so the neural default must refuse rather than fetch 168 MB.
    let output = run_with_env(
        &["cutout", input.to_str().expect("valid path")],
        &[("SIMPLY_MODEL_DIR", models.path())],
    );
    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("--download-model"), "{err}");
    assert!(err.contains("--fast"), "{err}");
    assert!(!models.path().join("u2net.onnx").exists());
}

#[test]
fn test_cutout_fast_never_consults_the_model_cache() {
    let temp = TestDir::new("simply-cutout-fast-nomodel");
    let models = TestDir::new("simply-cutout-fast-nomodel-cache");
    let input = temp.path().join("input.png");
    create_png(&input, 4, 4, [255, 255, 255, 255]);

    let output = run_with_env(
        &["cutout", "--fast", input.to_str().expect("valid path")],
        &[("SIMPLY_MODEL_DIR", models.path())],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(temp.path().join("input_cutout.png").exists());
}

#[test]
fn test_cutout_rejects_an_unreadable_model_file() {
    let temp = TestDir::new("simply-cutout-badmodel");
    let models = TestDir::new("simply-cutout-badmodel-cache");
    let input = temp.path().join("input.png");
    create_png(&input, 4, 4, [255, 255, 255, 255]);
    fs::write(models.path().join("u2net.onnx"), b"not an onnx graph").expect("write stub");

    let output = run_with_env(
        &["cutout", input.to_str().expect("valid path")],
        &[("SIMPLY_MODEL_DIR", models.path())],
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to load the background removal model"));
}
