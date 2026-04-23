use std::fs;

use crate::common::{TestDir, assert_valid_image, create_png, create_svg, run};

#[test]
fn test_convert_scale_half_produces_half_size() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 20, 10, "#ff0000");

    let output = run(&[
        "convert",
        "-s",
        "0.5",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted image");
    assert_eq!(img.width(), 10);
    assert_eq!(img.height(), 5);
}

#[test]
fn test_convert_scale_double_produces_double_size() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 6, 4, "#00ff00");

    let output = run(&[
        "convert",
        "-s",
        "2",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted image");
    assert_eq!(img.width(), 12);
    assert_eq!(img.height(), 8);
}

#[test]
fn test_convert_small_scale_still_produces_non_zero_dimensions() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 5, "#0000ff");

    let output = run(&[
        "convert",
        "-s",
        "0.1",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted image");
    assert!(img.width() >= 1);
    assert!(img.height() >= 1);
}

#[test]
fn test_convert_width_minimum_value_one_is_valid() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 10, "#abcdef");

    let output = run(&[
        "convert",
        "-w",
        "1",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted image");
    assert_eq!(img.width(), 1);
}

#[test]
fn test_convert_height_minimum_value_one_is_valid() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("in.svg");
    let dst = temp.path().join("out.png");
    create_svg(&src, 10, 10, "#fedcba");

    let output = run(&[
        "convert",
        "-h",
        "1",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let img = image::open(&dst).expect("failed to open converted image");
    assert_eq!(img.height(), 1);
}

#[test]
fn test_convert_rejects_nan_scale() {
    let output = run(&["convert", "-s", "NaN", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value 'NaN' for --scale"));
}

#[test]
fn test_convert_rejects_infinite_scale() {
    let output = run(&["convert", "-s", "inf", "in.svg", "out.png"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value 'inf' for --scale"));
}

#[test]
fn test_rotate_rejects_overflow_degrees_value() {
    let temp = TestDir::new("simply-phase3");
    let input = temp.path().join("in.png");
    let output_path = temp.path().join("out.png");
    create_png(&input, 2, 2, [1, 2, 3, 255]);

    let output = run(&[
        "rotate",
        "999999999999999",
        input.to_str().expect("valid input path"),
        output_path.to_str().expect("valid output path"),
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid rotation"));
}

#[test]
fn test_empty_input_path_is_rejected() {
    let temp = TestDir::new("simply-phase3");
    let out = temp.path().join("out.png");

    let output = run(&["invert", "", out.to_str().expect("valid output path")]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to open image"));
}

#[test]
fn test_convert_png_to_jpg_to_png_roundtrip_outputs_valid_images() {
    let temp = TestDir::new("simply-phase3");
    let src_png = temp.path().join("input.png");
    let mid_jpg = temp.path().join("mid.jpg");
    let out_png = temp.path().join("output.png");
    create_png(&src_png, 16, 16, [180, 90, 40, 255]);

    let first = run(&[
        "convert",
        src_png.to_str().expect("valid source path"),
        mid_jpg.to_str().expect("valid destination path"),
    ]);
    assert!(first.status.success());
    assert_valid_image(&mid_jpg);

    let second = run(&[
        "convert",
        mid_jpg.to_str().expect("valid source path"),
        out_png.to_str().expect("valid destination path"),
    ]);
    assert!(second.status.success());
    assert_valid_image(&out_png);
}

#[test]
fn test_convert_svg_with_embedded_style_renders_content() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("styled.svg");
    let dst = temp.path().join("styled.png");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8" viewBox="0 0 8 8">
  <style>
    .fill { fill: #00ff00; }
  </style>
  <rect class="fill" width="8" height="8"/>
</svg>"#;
    fs::write(&src, svg).expect("failed to write styled svg");

    let output = run(&[
        "convert",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());

    let px = image::open(&dst)
        .expect("failed to open rendered png")
        .to_rgba8()
        .get_pixel(0, 0)
        .0;
    assert!(px[1] > 200);
    assert!(px[3] > 0);
}

#[test]
fn test_convert_large_svg_succeeds() {
    let temp = TestDir::new("simply-phase3");
    let src = temp.path().join("large.svg");
    let dst = temp.path().join("large.png");
    create_svg(&src, 1024, 1024, "#123456");

    let output = run(&[
        "convert",
        src.to_str().expect("valid source path"),
        dst.to_str().expect("valid destination path"),
    ]);
    assert!(output.status.success());
    assert_valid_image(&dst);
}
