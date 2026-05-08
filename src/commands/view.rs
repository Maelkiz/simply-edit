use std::io::{self, Cursor, IsTerminal, Write};

use base64::Engine;
use image::{DynamicImage, imageops::FilterType};
use terminal_size::{Width, terminal_size};

const CHUNK_SIZE: usize = 4096;

pub fn run_view(path: &str) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("view: failed to open '{path}': {e}"))?;

    if detect_kitty_support() {
        display_kitty(img)
    } else {
        display_fallback(path, &img);
        Ok(())
    }
}

fn detect_kitty_support() -> bool {
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return true;
    }
    if std::env::var("TERM").map(|t| t == "xterm-kitty").unwrap_or(false) {
        return true;
    }
    std::env::var("TERM_PROGRAM")
        .map(|p| matches!(p.as_str(), "WezTerm" | "ghostty"))
        .unwrap_or(false)
}

/// Returns the terminal's usable pixel width, or None if unavailable.
///
/// Uses TIOCGWINSZ to read ws_xpixel. Falls back to None on non-Unix platforms
/// or when the terminal doesn't report pixel dimensions.
#[cfg(unix)]
fn terminal_pixel_width() -> Option<u32> {
    use std::os::unix::io::AsRawFd;

    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let fd = io::stdout().as_raw_fd();
    let ret = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
    if ret == 0 && ws.ws_xpixel > 0 {
        Some(ws.ws_xpixel as u32)
    } else {
        None
    }
}

#[cfg(not(unix))]
fn terminal_pixel_width() -> Option<u32> {
    None
}

fn fit_to_terminal(img: DynamicImage) -> (DynamicImage, Option<u16>) {
    // Primary: pixel-accurate resize — scale down if image is wider than terminal
    if let Some(px_width) = terminal_pixel_width() {
        if img.width() > px_width {
            return (
                img.resize(px_width, u32::MAX, FilterType::Lanczos3),
                None,
            );
        }
        return (img, None);
    }

    // Fallback: only column count available — pass c= to let the terminal scale
    if let Some((Width(cols), _)) = terminal_size() {
        return (img, Some(cols));
    }

    // No terminal info: send as-is
    (img, None)
}

fn display_kitty(img: DynamicImage) -> Result<(), String> {
    let (img, cols_hint) = fit_to_terminal(img);

    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("view: failed to encode image as PNG: {e}"))?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|c| std::str::from_utf8(c).expect("base64 is always valid UTF-8"))
        .collect();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let first_params = match cols_hint {
        Some(cols) => format!("a=T,f=100,q=1,c={cols},m="),
        None => "a=T,f=100,q=1,m=".to_string(),
    };

    let total = chunks.len();
    for (i, chunk) in chunks.iter().enumerate() {
        let m = if i == total - 1 { 0 } else { 1 };
        if i == 0 {
            write!(out, "\x1b_G{first_params}{m};{chunk}\x1b\\")
        } else {
            write!(out, "\x1b_Gm={m};{chunk}\x1b\\")
        }
        .map_err(|e| format!("view: write error: {e}"))?;
    }

    writeln!(out).map_err(|e| format!("view: write error: {e}"))?;
    out.flush().map_err(|e| format!("view: write error: {e}"))?;
    Ok(())
}

fn display_fallback(path: &str, img: &DynamicImage) {
    if io::stderr().is_terminal() {
        eprintln!(
            "note: Kitty image protocol not detected; run in Kitty, WezTerm, or Ghostty to view images inline"
        );
        eprintln!();
    }
    println!("File:   {path}");
    println!("Size:   {}×{} pixels", img.width(), img.height());
    println!("Color:  {:?}", img.color());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_view_missing_file() {
        let result = run_view("/nonexistent/path/image.png");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.starts_with("view: failed to open '/nonexistent/path/image.png'"));
    }

    #[test]
    fn test_fit_to_terminal_small_image_unchanged() {
        // A tiny 1×1 image should never be upscaled regardless of terminal size
        let img = DynamicImage::new_rgb8(1, 1);
        let original_width = img.width();
        let (fitted, _) = fit_to_terminal(img);
        assert!(fitted.width() <= original_width.max(1));
    }
}
