use std::process::Command;

use crate::common::{TestDir, assert_valid_image, binary_path, create_png, create_svg, run, run_with_stdin};

#[test]
fn test_flip_generated_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_flipv.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&["flip", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_flip_explicit_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&[
        "flip",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_flop_generated_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_fliph.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&["flop", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_flop_explicit_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&[
        "flop",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_invert_replace_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    create_png(&input, 1, 1, [10, 20, 30, 255]);

    let before = image::open(&input).expect("failed to load initial image");
    let before_px = before.to_rgba8().get_pixel(0, 0).0;

    let output = run(&["invert", "-r", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(input.exists());

    let after = image::open(&input).expect("failed to load transformed image");
    let after_px = after.to_rgba8().get_pixel(0, 0).0;
    assert_ne!(before_px, after_px);
}

#[test]
fn test_grayscale_generated_output_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_grayscale.png");
    create_png(&input, 2, 2, [250, 120, 10, 255]);

    let output = run(&["grayscale", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_rotate_explicit_output_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("rotated.png");
    create_png(&input, 3, 2, [0, 180, 180, 255]);

    let output = run(&[
        "rotate",
        "--angle",
        "90",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());

    let rotated = image::open(&out).expect("failed to open rotated output");
    assert_eq!(rotated.width(), 2);
    assert_eq!(rotated.height(), 3);
}

#[test]
fn test_rotate_interactive_generated_output_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_rotate180.png");
    create_png(&input, 3, 2, [0, 180, 180, 255]);

    let output = run_with_stdin(
        &["rotate", input.to_str().expect("valid input path")],
        "2\n",
    );
    assert!(output.status.success());
    assert!(generated.exists());

    let rotated = image::open(&generated).expect("failed to open rotated output");
    assert_eq!(rotated.width(), 3);
    assert_eq!(rotated.height(), 2);
}

#[test]
fn test_invalid_flag_syntax_for_transform_fails() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    create_png(&input, 2, 2, [255, 0, 0, 255]);

    let output = run(&[
        "invert",
        "--replace=true",
        input.to_str().expect("valid input path"),
    ]);
    assert!(!output.status.success());
}

#[test]
fn test_rasterize_svg_to_png_with_scale() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 3, 4, "#ff0000");

    let output = run(&[
        "rasterize",
        "-s",
        "2",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());

    let img = image::open(&dst).expect("failed to open converted png");
    assert_eq!(img.width(), 6);
    assert_eq!(img.height(), 8);
}

#[test]
fn test_rasterize_svg_to_png_with_width_preserves_aspect_ratio() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#00ff00");

    let output = run(&[
        "rasterize",
        "-w",
        "20",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted png");
    assert_eq!(img.width(), 20);
    assert_eq!(img.height(), 10);
}

#[test]
fn test_rasterize_svg_to_png_with_height_preserves_aspect_ratio() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#0000ff");

    let output = run(&[
        "rasterize",
        "-H",
        "15",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted png");
    assert_eq!(img.width(), 30);
    assert_eq!(img.height(), 15);
}

#[test]
fn test_rasterize_svg_to_png_with_width_and_height() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#aabbcc");

    let output = run(&[
        "rasterize",
        "-w",
        "12",
        "-H",
        "9",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted png");
    assert_eq!(img.width(), 12);
    assert_eq!(img.height(), 9);
}

#[test]
fn test_vectorize_image_to_svg() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.png");
    let dst = temp.path().join("out.svg");
    create_png(&src, 4, 4, [255, 255, 255, 255]);

    let output = run(&[
        "vectorize",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());

    let body = std::fs::read_to_string(&dst).expect("failed to read output svg");
    assert!(body.contains("<svg"));
}

#[test]
fn test_rasterize_svg_to_png_default() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 8, 6, "#112233");

    let output = run(&[
        "rasterize",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());

    let img = image::open(&dst).expect("failed to open converted png");
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 6);
}

#[test]
fn test_convert_image_to_svg() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.png");
    let dst = temp.path().join("out.svg");
    create_png(&src, 4, 4, [255, 255, 255, 255]);

    let output = run(&[
        "convert",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());

    let body = std::fs::read_to_string(&dst).expect("failed to read output svg");
    assert!(body.contains("<svg"));
}

#[test]
fn test_convert_png_to_jpg() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.png");
    let dst = temp.path().join("out.jpg");
    create_png(&src, 3, 3, [120, 120, 120, 255]);

    let output = run(&[
        "convert",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());
    assert_valid_image(&dst);
}

#[test]
fn test_resize_with_explicit_width_and_height() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 8, 8, [100, 150, 200, 255]);

    let output = run(&[
        "resize",
        "--width", "20",
        "-H", "10",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 20);
    assert_eq!(img.height(), 10);
}

#[test]
fn test_resize_generated_output_suffix() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_resize20x10.png");
    create_png(&input, 8, 8, [100, 150, 200, 255]);

    let output = run(&[
        "resize",
        "--width", "20",
        "-H", "10",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_resize_replace_mode() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    create_png(&input, 8, 8, [100, 150, 200, 255]);

    let output = run(&[
        "resize",
        "--replace",
        "--width", "4",
        "-H", "4",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(input.exists());

    let img = image::open(&input).expect("failed to open replaced image");
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 4);
}

#[test]
fn test_resize_with_scale_flag() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 4, 6, [200, 100, 50, 255]);

    let output = run(&[
        "resize",
        "--scale", "2",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open scaled image");
    assert_eq!(img.width(), 8);
    assert_eq!(img.height(), 12);
}

#[test]
fn test_resize_width_only_preserve_aspect_ratio() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 12, 6, [200, 100, 50, 255]);

    // "1" selects "Preserve aspect ratio"
    let output = run_with_stdin(
        &["resize", "--width", "24", input.to_str().expect("valid input path"), out.to_str().expect("valid output path")],
        "1\n",
    );
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 24);
    assert_eq!(img.height(), 12);
}

#[test]
fn test_resize_width_only_stretch() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 12, 6, [200, 100, 50, 255]);

    // "2" selects "Stretch"
    let output = run_with_stdin(
        &["resize", "--width", "24", input.to_str().expect("valid input path"), out.to_str().expect("valid output path")],
        "2\n",
    );
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 24);
    assert_eq!(img.height(), 6);
}

#[test]
fn test_resize_height_only_preserve_aspect_ratio() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 12, 6, [200, 100, 50, 255]);

    // "1" selects "Preserve aspect ratio"
    let output = run_with_stdin(
        &["resize", "-H", "12", input.to_str().expect("valid input path"), out.to_str().expect("valid output path")],
        "1\n",
    );
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 24);
    assert_eq!(img.height(), 12);
}

#[test]
fn test_resize_interactive_both_dimensions_via_stdin() {
    let temp = TestDir::new("simply-resize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 8, 8, [100, 150, 200, 255]);

    // No width/height flags — prompt asks for both
    let output = run_with_stdin(
        &["resize", input.to_str().expect("valid input path"), out.to_str().expect("valid output path")],
        "20\n10\n",
    );
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 20);
    assert_eq!(img.height(), 10);
}

#[test]
fn test_view_prints_metadata_in_non_kitty_terminal() {
    let temp = TestDir::new("simply-view-int");
    let input = temp.path().join("img.png");
    create_png(&input, 5, 3, [200, 100, 50, 255]);

    let output = Command::new(binary_path())
        .args(["view", input.to_str().expect("valid input path")])
        .env_remove("KITTY_WINDOW_ID")
        .env("TERM", "xterm-256color")
        .env_remove("TERM_PROGRAM")
        .output()
        .expect("failed to run simply binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("5") && stdout.contains("3"));
}

#[test]
fn test_convert_creates_output_in_nested_directory() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.png");
    let nested = temp.path().join("nested");
    std::fs::create_dir_all(&nested).expect("failed to create nested dir");
    let dst = nested.join("out.jpg");
    create_png(&src, 3, 3, [200, 10, 30, 255]);

    let output = run(&[
        "convert",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert!(dst.exists());
}

#[test]
fn test_info_basic_png() {
    let temp = TestDir::new("simply-info");
    let input = temp.path().join("photo.png");
    create_png(&input, 3, 2, [100, 150, 200, 255]);

    let output = run(&["info", input.to_str().expect("valid input path")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File: photo.png"), "missing File line: {stdout}");
    assert!(stdout.contains("Format: PNG"), "missing Format line: {stdout}");
    assert!(stdout.contains("Dimensions: 3\u{00d7}2"), "missing Dimensions line: {stdout}");
    assert!(stdout.contains("Size:"), "missing Size line: {stdout}");
    assert!(stdout.contains("Color:"), "missing Color section: {stdout}");
    assert!(stdout.contains("Metadata:"), "missing Metadata section: {stdout}");
}

#[test]
fn test_info_no_exif_png() {
    let temp = TestDir::new("simply-info");
    let input = temp.path().join("plain.png");
    create_png(&input, 4, 4, [255, 0, 0, 255]);

    let output = run(&["info", input.to_str().expect("valid input path")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EXIF: no"), "expected no EXIF: {stdout}");
    assert!(stdout.contains("ICC profile: none"), "expected no ICC: {stdout}");
}

#[test]
fn test_info_color_fields_rgb_png() {
    let temp = TestDir::new("simply-info");
    let input = temp.path().join("rgb.png");
    create_png(&input, 2, 2, [10, 20, 30, 255]);

    let output = run(&["info", input.to_str().expect("valid input path")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Space: sRGB"), "expected sRGB space: {stdout}");
    assert!(stdout.contains("Depth: 8-bit"), "expected 8-bit depth: {stdout}");
}

#[test]
fn test_binarize_generated_output_mode() {
    let temp = TestDir::new("simply-binarize-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_binarize.png");
    create_png(&input, 2, 2, [200, 200, 200, 255]);

    let output = run(&["binarize", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_binarize_with_threshold_flag() {
    let temp = TestDir::new("simply-binarize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 2, 2, [100, 100, 100, 255]);

    let output = run(&[
        "binarize",
        "--threshold",
        "50",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);

    let img = image::open(&out).expect("failed to open binarized image");
    let pixel = img.to_rgba8().get_pixel(0, 0).0;
    assert_eq!(pixel[0], 255);
    assert_eq!(pixel[1], 255);
    assert_eq!(pixel[2], 255);
}

#[test]
fn test_binarize_replace_mode() {
    let temp = TestDir::new("simply-binarize-int");
    let input = temp.path().join("img.png");
    create_png(&input, 1, 1, [100, 100, 100, 255]);

    let before = image::open(&input).expect("failed to load initial image");
    let before_px = before.to_rgba8().get_pixel(0, 0).0;

    let output = run(&["binarize", "-r", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(input.exists());

    let after = image::open(&input).expect("failed to load transformed image");
    let after_px = after.to_rgba8().get_pixel(0, 0).0;
    assert_ne!(before_px, after_px);
}

#[test]
fn test_binarize_explicit_output() {
    let temp = TestDir::new("simply-binarize-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 3, 3, [200, 200, 200, 255]);

    let output = run(&[
        "binarize",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_info_missing_file() {
    let output = run(&["info", "/nonexistent/does-not-exist.png"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("info:"), "expected 'info:' prefix in error: {stderr}");
    assert!(
        stderr.contains("does-not-exist.png"),
        "expected path in error: {stderr}"
    );
}

#[test]
fn test_pad_generated_output() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_pad.png");
    create_png(&input, 4, 3, [100, 150, 200, 255]);

    let output = run(&["pad", "--top", "5", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_pad_explicit_output() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 4, 3, [100, 150, 200, 255]);

    let output = run(&[
        "pad",
        "--left", "10",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_pad_replace_mode() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    create_png(&input, 4, 3, [100, 150, 200, 255]);

    let output = run(&["pad", "--replace", "--bottom", "8", input.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(input.exists());

    let img = image::open(&input).expect("failed to open replaced image");
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 11);
}

#[test]
fn test_pad_dimensions_correct() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 5, 3, [200, 100, 50, 255]);

    let output = run(&[
        "pad",
        "--top", "2",
        "--bottom", "4",
        "--left", "6",
        "--right", "8",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open padded image");
    assert_eq!(img.width(), 5 + 6 + 8);
    assert_eq!(img.height(), 3 + 2 + 4);
}

#[test]
fn test_pad_horizontal_shorthand() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 4, 3, [200, 100, 50, 255]);

    let output = run(&[
        "pad",
        "-x", "10",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open padded image");
    assert_eq!(img.width(), 4 + 10 + 10);
    assert_eq!(img.height(), 3);
}

#[test]
fn test_pad_color_pixels() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 2, 2, [50, 100, 150, 255]);

    let output = run(&[
        "pad",
        "--top", "3",
        "--color", "ff0000ff",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open padded image");
    let px = img.to_rgba8().get_pixel(0, 0).0;
    assert_eq!(px[0], 255, "expected red channel 255");
    assert_eq!(px[1], 0, "expected green channel 0");
    assert_eq!(px[2], 0, "expected blue channel 0");
}

#[test]
fn test_pad_default_padding() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 4, 3, [100, 150, 200, 255]);

    // No size flags: defaults to 20px on all sides
    let output = run(&[
        "pad",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open padded image");
    assert_eq!(img.width(), 4 + 20 + 20);
    assert_eq!(img.height(), 3 + 20 + 20);
}
