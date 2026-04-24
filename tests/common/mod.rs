#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_simply")
}

pub fn make_temp_dir(prefix: &str) -> PathBuf {
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

pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    pub fn new(prefix: &str) -> Self {
        Self {
            path: make_temp_dir(prefix),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn create_png(path: &Path, width: u32, height: u32, rgba: [u8; 4]) {
    let img = image::ImageBuffer::from_pixel(width, height, image::Rgba(rgba));
    image::DynamicImage::ImageRgba8(img)
        .save(path)
        .expect("failed to save png fixture");
}

pub fn create_svg(path: &Path, width: u32, height: u32, fill: &str) {
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="{width}" height="{height}" fill="{fill}"/>
</svg>"#
    );
    fs::write(path, svg).expect("failed to save svg fixture");
}

pub fn run(args: &[&str]) -> Output {
    Command::new(binary_path())
        .args(args)
        .output()
        .expect("failed to run simply binary")
}

pub fn run_with_stdin(args: &[&str], stdin_input: &str) -> Output {
    let mut child = Command::new(binary_path())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn simply binary");

    {
        let stdin = child.stdin.as_mut().expect("failed to open child stdin");
        stdin
            .write_all(stdin_input.as_bytes())
            .expect("failed to write stdin to simply binary");
    }

    child
        .wait_with_output()
        .expect("failed to collect simply binary output")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

pub fn assert_valid_image(path: &Path) {
    image::open(path).expect("expected a valid image output");
}
