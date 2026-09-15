use std::process::Command;

use crate::common::{
    TestDir, assert_valid_image, binary_path, create_png, create_svg, run, run_with_stdin,
};

#[test]
fn test_flip_generated_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_flipv.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&[
        "flip",
        "--vertical",
        input.to_str().expect("valid input path"),
    ]);
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
        "--vertical",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_flip_y_generated_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_fliph.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&[
        "flip",
        "--horizontal",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_flip_y_explicit_output() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run(&[
        "flip",
        "--horizontal",
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

    let output = run(&[
        "invert",
        "--replace",
        input.to_str().expect("valid input path"),
    ]);
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
        "--scale",
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
        "--width",
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
        "--height",
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
        "--width",
        "12",
        "--height",
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
        "--width",
        "20",
        "--height",
        "10",
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
        "--width",
        "20",
        "--height",
        "10",
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
        "--width",
        "4",
        "--height",
        "4",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(input.exists());

    let img = image::open(&input).expect("failed to open replaced image");
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 4);
}

#[test]
fn test_scale_with_factor_flag() {
    let temp = TestDir::new("simply-scale-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 4, 6, [200, 100, 50, 255]);

    let output = run(&[
        "scale",
        "--factor",
        "2",
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
        &[
            "resize",
            "--width",
            "24",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
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
        &[
            "resize",
            "--width",
            "24",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
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
        &[
            "resize",
            "--height",
            "12",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
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
        &[
            "resize",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
        "20\n10\n",
    );
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open resized image");
    assert_eq!(img.width(), 20);
    assert_eq!(img.height(), 10);
}

#[test]
fn test_view_fails_with_error_in_non_kitty_terminal() {
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

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Kitty"));
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
    assert!(
        stdout.contains("File: photo.png"),
        "missing File line: {stdout}"
    );
    assert!(
        stdout.contains("Format: PNG"),
        "missing Format line: {stdout}"
    );
    assert!(
        stdout.contains("Dimensions: 3\u{00d7}2"),
        "missing Dimensions line: {stdout}"
    );
    assert!(stdout.contains("Size:"), "missing Size line: {stdout}");
    assert!(stdout.contains("Color:"), "missing Color section: {stdout}");
    assert!(
        stdout.contains("Metadata:"),
        "missing Metadata section: {stdout}"
    );
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
    assert!(
        stdout.contains("ICC profile: none"),
        "expected no ICC: {stdout}"
    );
}

#[test]
fn test_info_color_fields_rgb_png() {
    let temp = TestDir::new("simply-info");
    let input = temp.path().join("rgb.png");
    create_png(&input, 2, 2, [10, 20, 30, 255]);

    let output = run(&["info", input.to_str().expect("valid input path")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Space: sRGB"),
        "expected sRGB space: {stdout}"
    );
    assert!(
        stdout.contains("Depth: 8-bit"),
        "expected 8-bit depth: {stdout}"
    );
}

#[test]
fn test_binarize_generated_output_mode() {
    let temp = TestDir::new("simply-binarize-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_binarize.png");
    create_png(&input, 2, 2, [200, 200, 200, 255]);

    let output = run(&[
        "binarize",
        "--threshold",
        "128",
        input.to_str().expect("valid input path"),
    ]);
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

    let output = run(&[
        "binarize",
        "--replace",
        "--threshold",
        "128",
        input.to_str().expect("valid input path"),
    ]);
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
        "--threshold",
        "128",
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
    assert!(
        stderr.contains("info:"),
        "expected 'info:' prefix in error: {stderr}"
    );
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

    let output = run(&[
        "pad",
        "--top",
        "5",
        input.to_str().expect("valid input path"),
    ]);
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
        "--left",
        "10",
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

    let output = run(&[
        "pad",
        "--replace",
        "--bottom",
        "8",
        input.to_str().expect("valid input path"),
    ]);
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
        "--top",
        "2",
        "--bottom",
        "4",
        "--left",
        "6",
        "--right",
        "8",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&out).expect("failed to open padded image");
    assert_eq!(img.width(), 5 + 6 + 8);
    assert_eq!(img.height(), 3 + 2 + 4);
}

#[test]
fn test_pad_horizontal_flag() {
    let temp = TestDir::new("simply-pad-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("out.png");
    create_png(&input, 4, 3, [200, 100, 50, 255]);

    let output = run(&[
        "pad",
        "--horizontal",
        "10",
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
        "--top",
        "3",
        "--color",
        "ff0000ff",
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

/// Runs a shorthand and its long form on identical inputs and asserts both
/// produce the same generated filename with byte-identical content.
fn assert_shorthand_matches(shorthand: &[&str], long_form: &[&str], suffix: &str) {
    let temp = TestDir::new("simply-shorthand-int");
    let short_in = temp.path().join("short.png");
    let long_in = temp.path().join("long.png");
    create_png(&short_in, 4, 2, [220, 30, 30, 255]);
    create_png(&long_in, 4, 2, [220, 30, 30, 255]);

    let mut short_args: Vec<&str> = shorthand.to_vec();
    short_args.push(short_in.to_str().expect("valid input path"));
    let mut long_args: Vec<&str> = long_form.to_vec();
    long_args.push(long_in.to_str().expect("valid input path"));

    assert!(run(&short_args).status.success());
    assert!(run(&long_args).status.success());

    let short_out = temp.path().join(format!("short_{suffix}.png"));
    let long_out = temp.path().join(format!("long_{suffix}.png"));
    assert!(
        short_out.exists(),
        "shorthand did not produce {}",
        short_out.display()
    );
    assert_valid_image(&short_out);
    assert_eq!(
        std::fs::read(&short_out).expect("read shorthand output"),
        std::fs::read(&long_out).expect("read long-form output"),
    );
}

#[test]
fn test_shorthand_fliph_matches_flag() {
    assert_shorthand_matches(&["fliph"], &["flip", "--horizontal"], "fliph");
}

#[test]
fn test_shorthand_flipv_matches_flag() {
    assert_shorthand_matches(&["flipv"], &["flip", "--vertical"], "flipv");
}

#[test]
fn test_shorthand_rotate90_matches_flag() {
    assert_shorthand_matches(&["rotate90"], &["rotate", "--angle", "90"], "rotate90");
}

#[test]
fn test_shorthand_rotate180_matches_flag() {
    assert_shorthand_matches(&["rotate180"], &["rotate", "--angle", "180"], "rotate180");
}

#[test]
fn test_shorthand_rotate270_matches_flag() {
    assert_shorthand_matches(&["rotate270"], &["rotate", "--angle", "270"], "rotate270");
}

#[test]
fn test_shorthand_accepts_replace_flag() {
    let temp = TestDir::new("simply-shorthand-int");
    let input = temp.path().join("img.png");
    create_png(&input, 4, 2, [220, 30, 30, 255]);

    let output = run(&[
        "fliph",
        "--replace",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(input.exists());
    assert!(!temp.path().join("img_fliph.png").exists());
    assert_valid_image(&input);
}

#[test]
fn test_shorthand_accepts_explicit_output() {
    let temp = TestDir::new("simply-shorthand-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 4, 2, [220, 30, 30, 255]);

    let output = run(&[
        "rotate90",
        input.to_str().expect("valid input path"),
        out.to_str().expect("valid output path"),
    ]);
    assert!(output.status.success());
    assert_valid_image(&out);
}

#[test]
fn test_shorthand_works_in_batch_mode() {
    let temp = TestDir::new("simply-shorthand-batch");
    let out = TestDir::new("simply-shorthand-batch-out");
    for name in ["a.png", "b.png"] {
        create_png(&temp.path().join(name), 4, 2, [220, 30, 30, 255]);
    }

    let output = run(&[
        "flipv",
        temp.path().to_str().expect("valid input dir"),
        "--output-dir",
        out.path().to_str().expect("valid output dir"),
    ]);
    assert!(output.status.success());
    assert!(out.path().join("a_flipv.png").exists());
    assert!(out.path().join("b_flipv.png").exists());
}

/// Writes a 2x2 PNG whose four pixels are all distinct, so a mirror in either
/// direction is detectable from the pixel values alone.
fn create_asymmetric_png(path: &std::path::Path) {
    let mut img = image::RgbaImage::new(2, 2);
    img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
    img.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
    img.put_pixel(1, 1, image::Rgba([255, 255, 0, 255]));
    image::DynamicImage::ImageRgba8(img)
        .save(path)
        .expect("failed to save asymmetric png fixture");
}

fn pixels(path: &std::path::Path) -> Vec<[u8; 4]> {
    let img = image::open(path).expect("failed to open image").to_rgba8();
    img.pixels().map(|p| p.0).collect()
}

#[test]
fn test_flip_horizontal_mirrors_left_to_right() {
    let temp = TestDir::new("simply-flip-axis");
    let input = temp.path().join("img.png");
    create_asymmetric_png(&input);

    let output = run(&[
        "flip",
        "--horizontal",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let before = pixels(&input);
    let after = pixels(&temp.path().join("img_fliph.png"));
    // Rows keep their order; columns swap within each row.
    assert_eq!(after, vec![before[1], before[0], before[3], before[2]]);
}

#[test]
fn test_flip_vertical_mirrors_top_to_bottom() {
    let temp = TestDir::new("simply-flip-axis");
    let input = temp.path().join("img.png");
    create_asymmetric_png(&input);

    let output = run(&[
        "flip",
        "--vertical",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let before = pixels(&input);
    let after = pixels(&temp.path().join("img_flipv.png"));
    // Columns keep their order; rows swap.
    assert_eq!(after, vec![before[2], before[3], before[0], before[1]]);
}

#[test]
fn test_cutout_generated_output_makes_background_transparent() {
    let temp = TestDir::new("simply-cutout-int");
    let input = temp.path().join("flat.png");
    let generated = temp.path().join("flat_cutout.png");
    create_png(&input, 8, 8, [255, 255, 255, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);

    let img = image::open(&generated).expect("cutout output should be a valid image");
    assert_eq!(img.to_rgba8().get_pixel(0, 0).0[3], 0);
}

#[test]
fn test_cutout_forces_png_for_jpeg_source() {
    let temp = TestDir::new("simply-cutout-jpeg");
    let png = temp.path().join("src.png");
    let jpeg = temp.path().join("photo.jpg");
    create_png(&png, 8, 8, [200, 200, 200, 255]);
    image::open(&png)
        .expect("source png")
        .to_rgb8()
        .save(&jpeg)
        .expect("failed to write jpeg");

    let output = run(&["cutout", "--fast", jpeg.to_str().expect("valid input path")]);
    assert!(output.status.success());
    assert!(temp.path().join("photo_cutout.png").exists());
    assert!(!temp.path().join("photo_cutout.jpg").exists());
}

#[test]
fn test_cutout_preserves_dimensions() {
    let temp = TestDir::new("simply-cutout-dims");
    let input = temp.path().join("img.png");
    create_png(&input, 6, 4, [10, 10, 10, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let img = image::open(temp.path().join("img_cutout.png")).expect("valid output");
    assert_eq!((img.width(), img.height()), (6, 4));
}

#[test]
fn test_cutout_replace_rewrites_png_in_place() {
    let temp = TestDir::new("simply-cutout-replace");
    let input = temp.path().join("img.png");
    create_png(&input, 5, 5, [40, 90, 160, 255]);

    let output = run(&[
        "cutout",
        "--fast",
        "--replace",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&input).expect("valid replaced image");
    assert_eq!(img.to_rgba8().get_pixel(0, 0).0[3], 0);
}

#[test]
fn test_cutout_trim_crops_to_subject() {
    let temp = TestDir::new("simply-cutout-trim");
    let input = temp.path().join("logo.png");

    // White field with an off-centre red block: the flood fill clears the
    // white, and --trim should shrink the output to just the block.
    let mut img = image::RgbaImage::from_pixel(20, 20, image::Rgba([255, 255, 255, 255]));
    for y in 5..11 {
        for x in 8..12 {
            img.put_pixel(x, y, image::Rgba([200, 30, 30, 255]));
        }
    }
    img.save(&input).expect("failed to write input");

    let output = run(&[
        "cutout",
        "--fast",
        "--trim",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let cut = image::open(temp.path().join("logo_cutout.png")).expect("valid output");
    assert_eq!((cut.width(), cut.height()), (4, 6));
}

#[test]
fn test_cutout_without_trim_keeps_original_dimensions() {
    let temp = TestDir::new("simply-cutout-no-trim");
    let input = temp.path().join("logo.png");

    let mut img = image::RgbaImage::from_pixel(20, 20, image::Rgba([255, 255, 255, 255]));
    for y in 5..11 {
        for x in 8..12 {
            img.put_pixel(x, y, image::Rgba([200, 30, 30, 255]));
        }
    }
    img.save(&input).expect("failed to write input");

    let output = run(&[
        "cutout",
        "--fast",
        input.to_str().expect("valid input path"),
    ]);
    assert!(output.status.success());

    let cut = image::open(temp.path().join("logo_cutout.png")).expect("valid output");
    assert_eq!((cut.width(), cut.height()), (20, 20));
}
