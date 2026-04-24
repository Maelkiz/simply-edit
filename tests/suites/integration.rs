use crate::common::{TestDir, assert_valid_image, create_png, create_svg, run, run_with_stdin};

#[test]
fn test_flip_horizontal_generated_output_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let generated = temp.path().join("img_fliph.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run_with_stdin(&["flip", input.to_str().expect("valid input path")], "1\n");
    assert!(output.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_flip_vertical_explicit_output_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    let out = temp.path().join("custom.png");
    create_png(&input, 3, 2, [220, 30, 30, 255]);

    let output = run_with_stdin(
        &[
            "flip",
            input.to_str().expect("valid input path"),
            out.to_str().expect("valid output path"),
        ],
        "2\n",
    );
    assert!(output.status.success());
    assert!(out.exists());
    assert_valid_image(&out);
}

#[test]
fn test_flip_horizontal_flag_bypasses_prompt() {
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
fn test_flip_vertical_flag_bypasses_prompt() {
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
fn test_invert_replace_mode() {
    let temp = TestDir::new("simply-phase1-int");
    let input = temp.path().join("img.png");
    create_png(&input, 1, 1, [10, 20, 30, 255]);

    let before = image::open(&input).expect("failed to load initial image");
    let before_px = before.to_rgba8().get_pixel(0, 0).0;

    let output = run(&[
        "invert",
        "-r",
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
fn test_convert_svg_to_png_with_scale() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 3, 4, "#ff0000");

    let output = run(&[
        "convert",
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
fn test_convert_svg_to_png_with_width_preserves_aspect_ratio() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#00ff00");

    let output = run(&[
        "convert",
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
fn test_convert_svg_to_png_with_height_preserves_aspect_ratio() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#0000ff");

    let output = run(&[
        "convert",
        "-h",
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
fn test_convert_svg_to_png_with_width_and_height() {
    let temp = TestDir::new("simply-phase1-int");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#aabbcc");

    let output = run(&[
        "convert",
        "-w",
        "12",
        "-h",
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
