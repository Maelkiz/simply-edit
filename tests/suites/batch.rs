use std::fs;

use crate::common::{TestDir, create_png, create_svg, run};

fn batch_dir_with_images(prefix: &str, count: u32) -> TestDir {
    let temp = TestDir::new(prefix);
    for i in 0..count {
        create_png(
            &temp.path().join(format!("img_{i}.png")),
            4,
            4,
            [100 + (i as u8 * 30), 50, 50, 255],
        );
    }
    temp
}

#[test]
fn test_batch_invert_processes_all_files() {
    let temp = batch_dir_with_images("batch-invert", 3);
    let out = TestDir::new("batch-invert-out");

    let output = run(&[
        "invert",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3 succeeded"));
    assert!(stdout.contains("0 failed"));

    let files: Vec<_> = fs::read_dir(out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 3);
}

#[test]
fn test_batch_grayscale_produces_valid_output() {
    let temp = batch_dir_with_images("batch-gray", 2);
    let out = TestDir::new("batch-gray-out");

    let output = run(&[
        "grayscale",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    for entry in fs::read_dir(out.path()).unwrap() {
        let path = entry.unwrap().path();
        let img = image::open(&path).expect("output should be valid image");
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
    }
}

#[test]
fn test_batch_flip_requires_axis_flag() {
    let temp = batch_dir_with_images("batch-flip-noaxis", 1);

    let output = run(&["flip", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--horizontal or --vertical required"));
}

#[test]
fn test_batch_flip_horizontal() {
    let temp = batch_dir_with_images("batch-fliph", 2);
    let out = TestDir::new("batch-fliph-out");

    let output = run(&[
        "flip",
        "--horizontal",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 succeeded"));
}

#[test]
fn test_batch_rotate_requires_angle_flag() {
    let temp = batch_dir_with_images("batch-rot-noangle", 1);

    let output = run(&["rotate", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--angle required"));
}

#[test]
fn test_batch_rotate_90() {
    let temp = TestDir::new("batch-rot90");
    create_png(&temp.path().join("wide.png"), 8, 4, [200, 100, 50, 255]);
    let out = TestDir::new("batch-rot90-out");

    let output = run(&[
        "rotate",
        "--angle",
        "90",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let files: Vec<_> = fs::read_dir(out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    let img = image::open(files[0].path()).unwrap();
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 8);
}

#[test]
fn test_batch_with_pattern_filters_files() {
    let temp = TestDir::new("batch-pattern");
    create_png(&temp.path().join("photo_a.png"), 4, 4, [100, 100, 100, 255]);
    create_png(&temp.path().join("photo_b.png"), 4, 4, [150, 150, 150, 255]);
    create_png(
        &temp.path().join("screenshot.png"),
        4,
        4,
        [200, 200, 200, 255],
    );
    let out = TestDir::new("batch-pattern-out");

    let output = run(&[
        "invert",
        temp.path().to_str().unwrap(),
        "--pattern",
        r"^photo_",
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 succeeded"));
}

#[test]
fn test_batch_with_output_dir_creates_files_there() {
    let temp = batch_dir_with_images("batch-outdir", 2);
    let out = TestDir::new("batch-outdir-out");

    let output = run(&[
        "invert",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let src_files: Vec<_> = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let dst_files: Vec<_> = fs::read_dir(out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(src_files.len(), 2);
    assert_eq!(dst_files.len(), 2);
}

#[test]
fn test_batch_recursive_finds_nested_files() {
    let temp = TestDir::new("batch-recursive");
    let sub = temp.path().join("subdir");
    fs::create_dir_all(&sub).unwrap();
    create_png(&temp.path().join("top.png"), 4, 4, [50, 50, 50, 255]);
    create_png(&sub.join("nested.png"), 4, 4, [100, 100, 100, 255]);
    let out = TestDir::new("batch-recursive-out");

    let output = run(&[
        "invert",
        "-R",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 succeeded"));
}

#[test]
fn test_batch_with_corrupt_file_reports_partial_failure() {
    let temp = TestDir::new("batch-corrupt");
    create_png(&temp.path().join("good.png"), 4, 4, [100, 100, 100, 255]);
    fs::write(temp.path().join("bad.png"), b"not an image").unwrap();
    let out = TestDir::new("batch-corrupt-out");

    let output = run(&[
        "invert",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 succeeded"));
    assert!(stdout.contains("1 failed"));
}

#[test]
fn test_batch_empty_dir_reports_no_files() {
    let temp = TestDir::new("batch-empty-e2e");

    let output = run(&["invert", temp.path().to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No matching files found"));
}

#[test]
fn test_batch_convert_requires_format() {
    let temp = batch_dir_with_images("batch-conv-nofmt", 1);

    let output = run(&["convert", temp.path().to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--format required"));
}

#[test]
fn test_batch_convert_with_format() {
    let temp = batch_dir_with_images("batch-conv", 2);
    let out = TestDir::new("batch-conv-out");

    let output = run(&[
        "convert",
        "--format",
        "jpg",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 succeeded"));

    let jpg_count = fs::read_dir(out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "jpg"))
        .count();
    assert_eq!(jpg_count, 2);
}

#[test]
fn test_batch_rasterize_processes_svgs() {
    let temp = TestDir::new("batch-rast");
    create_svg(&temp.path().join("a.svg"), 10, 10, "#ff0000");
    create_svg(&temp.path().join("b.svg"), 20, 20, "#00ff00");
    create_png(&temp.path().join("skip.png"), 4, 4, [0, 0, 0, 255]);
    let out = TestDir::new("batch-rast-out");

    let output = run(&[
        "rasterize",
        temp.path().to_str().unwrap(),
        "--output-dir",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 succeeded"));

    let png_count = fs::read_dir(out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "png"))
        .count();
    assert_eq!(png_count, 2);
}

#[test]
fn test_single_file_still_works_after_batch_changes() {
    let temp = TestDir::new("single-regression");
    let input = temp.path().join("img.png");
    let output_path = temp.path().join("out.png");
    create_png(&input, 4, 4, [200, 100, 50, 255]);

    let output = run(&[
        "invert",
        input.to_str().unwrap(),
        output_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(output_path.exists());
    image::open(&output_path).expect("should be valid image");
}
