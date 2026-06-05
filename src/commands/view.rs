use std::io::{self, Cursor, Write};

use base64::Engine;
use image::{DynamicImage, imageops::FilterType};
use terminal_size::{Width, terminal_size};

const CHUNK_SIZE: usize = 4096;

pub fn display_image(img: DynamicImage) -> Result<(), String> {
    if !detect_kitty_support() {
        return Err(
            "preview requires a terminal with Kitty graphics protocol support (Kitty, WezTerm, or Ghostty)".to_string(),
        );
    }
    display_kitty(img)
}

/// Transmit raw 32-bit RGBA pixels to the terminal using the Kitty graphics protocol.
/// Uses image ID 1 so frames can be replaced via `delete_kitty_image`.
pub(crate) fn display_raw_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(rgba);
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|c| std::str::from_utf8(c).expect("base64 is always valid UTF-8"))
        .collect();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let total = chunks.len();
    for (i, chunk) in chunks.iter().enumerate() {
        let m = if i == total - 1 { 0 } else { 1 };
        if i == 0 {
            write!(out, "\x1b_Ga=T,f=32,s={width},v={height},i=1,q=1,m={m};{chunk}\x1b\\")
        } else {
            write!(out, "\x1b_Gm={m};{chunk}\x1b\\")
        }
        .map_err(|e| format!("view: write error: {e}"))?;
    }

    writeln!(out).map_err(|e| format!("view: write error: {e}"))?;
    out.flush().map_err(|e| format!("view: write error: {e}"))?;
    Ok(())
}

/// Delete the image with ID 1 from the terminal, clearing the cells it occupied.
pub(crate) fn delete_kitty_image() -> Result<(), String> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write!(out, "\x1b_Ga=d,d=I,i=1\x1b\\").map_err(|e| format!("view: write error: {e}"))?;
    out.flush().map_err(|e| format!("view: write error: {e}"))
}

pub fn run_view(path: &str) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("view: failed to open '{path}': {e}"))?;

    if !detect_kitty_support() {
        return Err(
            "view: requires a terminal with Kitty graphics protocol support (Kitty, WezTerm, or Ghostty)".to_string(),
        );
    }
    display_kitty(img)
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

/// Returns the terminal's usable pixel (width, height), or None for each if unavailable.
///
/// Height has 2 rows subtracted to leave room for the shell prompt below the image.
/// Falls back to (None, None) on non-Unix platforms or when the terminal doesn't report pixels.
#[cfg(unix)]
pub(crate) fn terminal_pixel_size() -> (Option<u32>, Option<u32>) {
    use std::os::unix::io::AsRawFd;

    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let fd = io::stdout().as_raw_fd();
    let ret = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
    if ret != 0 {
        return (None, None);
    }
    let w = if ws.ws_xpixel > 0 { Some(ws.ws_xpixel as u32) } else { None };
    let h = if ws.ws_ypixel > 0 && ws.ws_row > 0 {
        let row_px = ws.ws_ypixel as u32 / ws.ws_row as u32;
        Some((ws.ws_ypixel as u32).saturating_sub(row_px * 2))
    } else {
        None
    };
    (w, h)
}

#[cfg(not(unix))]
pub(crate) fn terminal_pixel_size() -> (Option<u32>, Option<u32>) {
    (None, None)
}

fn fit_to_terminal(img: DynamicImage) -> (DynamicImage, Option<u16>) {
    let (px_w, px_h) = terminal_pixel_size();
    if px_w.is_some() || px_h.is_some() {
        let scale_w = px_w.filter(|&w| img.width() > w).map(|w| w as f32 / img.width() as f32);
        let scale_h = px_h.filter(|&h| img.height() > h).map(|h| h as f32 / img.height() as f32);
        let scale = match (scale_w, scale_h) {
            (Some(sw), Some(sh)) => Some(sw.min(sh)),
            (Some(sw), None) => Some(sw),
            (None, Some(sh)) => Some(sh),
            (None, None) => None,
        };
        if let Some(s) = scale {
            let new_w = ((img.width() as f32 * s).round() as u32).max(1);
            let new_h = ((img.height() as f32 * s).round() as u32).max(1);
            return (img.resize(new_w, new_h, FilterType::Lanczos3), None);
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
