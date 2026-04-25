use std::path::Path;

use crate::common::{TestDir, assert_valid_image, create_png, run, run_with_stdin};

fn pixel(path: &Path, x: u32, y: u32) -> [u8; 4] {
    image::open(path)
        .expect("failed to open image")
        .to_rgba8()
        .get_pixel(x, y)
        .0
}

#[test]
fn test_invert_handles_1x1_image() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("tiny.png");
    let output = temp.path().join("tiny_out.png");
    create_png(&input, 1, 1, [0, 0, 0, 255]);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());

    let out = image::open(&output).expect("failed to open output");
    assert_eq!(out.width(), 1);
    assert_eq!(out.height(), 1);
    assert_eq!(pixel(&output, 0, 0), [255, 255, 255, 255]);
}

#[test]
fn test_rotate_90_swaps_non_square_dimensions() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("rect.png");
    let output = temp.path().join("rot90.png");
    create_png(&input, 7, 3, [30, 40, 50, 255]);

    let result = run(&[
        "rotate",
        "90",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());

    let out = image::open(&output).expect("failed to open rotated output");
    assert_eq!(out.width(), 3);
    assert_eq!(out.height(), 7);
}

#[test]
fn test_invert_preserves_alpha_for_transparent_pixels() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("alpha.png");
    let output = temp.path().join("alpha_out.png");
    create_png(&input, 1, 1, [25, 50, 75, 0]);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());

    let out_px = pixel(&output, 0, 0);
    assert_eq!(out_px[3], 0);
}

#[test]
fn test_invert_twice_roundtrip_returns_original() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("input.png");
    let once = temp.path().join("once.png");
    let twice = temp.path().join("twice.png");
    create_png(&input, 2, 2, [64, 128, 192, 200]);

    let step1 = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        once.to_str().expect("valid output path"),
    ]);
    assert!(step1.status.success());

    let step2 = run(&[
        "invert",
        once.to_str().expect("valid input path"),
        twice.to_str().expect("valid output path"),
    ]);
    assert!(step2.status.success());

    assert_eq!(pixel(&input, 0, 0), pixel(&twice, 0, 0));
}

#[test]
fn test_generated_mode_uses_expected_suffix_name() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("name.png");
    let generated = temp.path().join("name_grayscale.png");
    create_png(&input, 2, 2, [100, 50, 20, 255]);

    let result = run(&["grayscale", input.to_str().expect("valid input path")]);
    assert!(result.status.success());
    assert!(generated.exists());
    assert_valid_image(&generated);
}

#[test]
fn test_explicit_mode_enumerates_existing_file() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("input.png");
    let output = temp.path().join("output.png");
    let enumerated = temp.path().join("output1.png");

    create_png(&input, 1, 1, [10, 20, 30, 255]);
    create_png(&output, 1, 1, [250, 240, 230, 255]);
    let original = pixel(&output, 0, 0);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());

    assert_eq!(pixel(&output, 0, 0), original);
    assert!(enumerated.exists());
    assert_eq!(pixel(&enumerated, 0, 0), [245, 235, 225, 255]);
}

#[test]
fn test_explicit_mode_fails_for_missing_directory() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("input.png");
    let output = temp.path().join("does-not-exist").join("out.png");
    create_png(&input, 2, 2, [10, 20, 30, 255]);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);

    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("failed to save image"));
}

#[test]
fn test_replace_mode_preserves_original_filename() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("keep_name.png");
    create_png(&input, 1, 1, [10, 20, 30, 255]);

    let result = run(&[
        "invert",
        "--replace",
        input.to_str().expect("valid input path"),
    ]);
    assert!(result.status.success());
    assert!(input.exists());

    let px = pixel(&input, 0, 0);
    assert_eq!(px, [245, 235, 225, 255]);
}

#[test]
fn test_replace_mode_cleans_up_tmp_file() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("sample.png");
    let tmp = temp.path().join("sample_invert.simple-edit-tmp.png");
    create_png(&input, 1, 1, [10, 20, 30, 255]);

    let result = run(&[
        "invert",
        "-r",
        input.to_str().expect("valid input path"),
    ]);
    assert!(result.status.success());
    assert!(!tmp.exists());
}

#[test]
fn test_replace_mode_overwrites_explicit_target() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("input.png");
    let target = temp.path().join("target.png");

    create_png(&input, 1, 1, [10, 20, 30, 255]);
    create_png(&target, 1, 1, [250, 240, 230, 255]);
    let before = pixel(&target, 0, 0);

    let result = run(&[
        "invert",
        "-r",
        input.to_str().expect("valid input path"),
        target.to_str().expect("valid target path"),
    ]);
    assert!(result.status.success());

    let after = pixel(&target, 0, 0);
    assert_ne!(before, after);
    assert_eq!(after, [245, 235, 225, 255]);
}

#[test]
fn test_multiple_generated_operations_do_not_conflict() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("chain.png");
    let step1 = temp.path().join("chain_invert.png");
    let step2 = temp.path().join("chain_invert_fliph.png");
    create_png(&input, 2, 2, [12, 34, 56, 255]);

    let first = run(&["invert", input.to_str().expect("valid input path")]);
    assert!(first.status.success());
    assert!(step1.exists());

    let second = run_with_stdin(&["flip", step1.to_str().expect("valid input path")], "1\n");
    assert!(second.status.success());
    assert!(step2.exists());
}

#[test]
fn test_spaces_in_file_paths_are_supported() {
    let temp = TestDir::new("simply-phase2");
    let input = temp.path().join("my image.png");
    let output = temp.path().join("my output.png");
    create_png(&input, 2, 2, [200, 100, 50, 255]);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());
    assert!(output.exists());
}

#[test]
fn test_absolute_paths_work_for_transforms() {
    let temp = TestDir::new("simply-phase2");
    let input = std::fs::canonicalize(temp.path().join("abs_in.png")).unwrap_or_else(|_| {
        let p = temp.path().join("abs_in.png");
        p
    });
    let output = temp.path().join("abs_out.png");

    create_png(&input, 2, 2, [1, 2, 3, 255]);

    let result = run(&[
        "invert",
        input.to_str().expect("valid input path"),
        output.to_str().expect("valid output path"),
    ]);
    assert!(result.status.success());
    assert!(output.exists());
}
