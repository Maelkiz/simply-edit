use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_simply")
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn test_cli_without_args_prints_usage_error() {
    let output = Command::new(binary_path())
        .output()
        .expect("failed to run simply binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_convert_rejects_unknown_flag() {
    let output = Command::new(binary_path())
        .args(["convert", "--bogus", "in.svg", "out.png"])
        .output()
        .expect("failed to run simply binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unrecognized convert flag '--bogus'"));
}

#[test]
fn test_cli_invert_creates_output_file() {
    let temp_dir = make_temp_dir("simply-cli-smoke");
    let input_path = temp_dir.join("input.png");
    let output_path = temp_dir.join("output.png");

    let img = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
    image::DynamicImage::ImageRgba8(img)
        .save(&input_path)
        .expect("failed to save input image");

    let output = Command::new(binary_path())
        .args([
            "invert",
            input_path.to_str().expect("invalid input path"),
            output_path.to_str().expect("invalid output path"),
        ])
        .output()
        .expect("failed to run simply binary");

    assert!(output.status.success());
    assert!(output_path.exists());

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path);
    let _ = fs::remove_dir(&temp_dir);
}
