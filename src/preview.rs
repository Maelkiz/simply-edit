use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{self, Event, KeyEvent};
use crossterm::terminal;
use image::{DynamicImage, ImageBuffer, Rgba, imageops::FilterType};

use crate::commands::view::{delete_kitty_image, display_raw_rgba, terminal_pixel_size};

static RESIZED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigwinch(_: libc::c_int) {
    RESIZED.store(true, Ordering::Relaxed);
}

pub(crate) struct LivePreview {
    /// Downscaled copy of the source image used for fast per-frame transforms.
    pub(crate) scaled: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// Pixel dimensions of the terminal at the time the preview was created (or last resized).
    term_px: (Option<u32>, Option<u32>),
}

impl LivePreview {
    /// Enter raw mode, register the SIGWINCH handler, downscale `img` to fit the terminal,
    /// and save the cursor position ready for the first `render` call.
    pub(crate) fn new(img: &DynamicImage) -> Result<Self, String> {
        #[cfg(unix)]
        unsafe {
            libc::signal(libc::SIGWINCH, handle_sigwinch as libc::sighandler_t);
        }
        RESIZED.store(false, Ordering::Relaxed);

        terminal::enable_raw_mode().map_err(|e| format!("preview: failed to enter raw mode: {e}"))?;

        // Save cursor position so render() can return to the same spot each frame.
        let stdout = io::stdout();
        let mut out = stdout.lock();
        write!(out, "\x1b7").map_err(|e| format!("preview: write error: {e}"))?;
        out.flush().map_err(|e| format!("preview: write error: {e}"))?;
        drop(out);

        let term_px = terminal_pixel_size();
        let scaled = scale_to_terminal(img, term_px);
        Ok(Self { scaled, term_px })
    }

    /// Restore cursor to the saved position, print `label`, then render the RGBA buffer.
    pub(crate) fn render(&self, rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>, label: &str) -> Result<(), String> {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        // Restore cursor to saved position and erase everything below it.
        write!(out, "\x1b8\x1b[J").map_err(|e| format!("preview: write error: {e}"))?;
        // Print the label on its own line, then move to the next line for the image.
        writeln!(out, "{label}\r").map_err(|e| format!("preview: write error: {e}"))?;
        out.flush().map_err(|e| format!("preview: write error: {e}"))?;
        drop(out);

        display_raw_rgba(rgba.width(), rgba.height(), rgba.as_raw())
    }

    /// Re-query terminal size, re-downscale `img`, and re-render with `label`.
    pub(crate) fn handle_resize(
        &mut self,
        img: &DynamicImage,
        rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        label: &str,
    ) -> Result<(), String> {
        RESIZED.store(false, Ordering::Relaxed);
        self.term_px = terminal_pixel_size();
        self.scaled = scale_to_terminal(img, self.term_px);
        self.render(rgba, label)
    }

    /// Check whether a SIGWINCH has been received since the last `handle_resize`.
    pub(crate) fn needs_resize() -> bool {
        RESIZED.load(Ordering::Relaxed)
    }

    /// Delete the displayed image, restore the cursor, and disable raw mode.
    pub(crate) fn finish(&self) -> Result<(), String> {
        delete_kitty_image()?;
        let stdout = io::stdout();
        let mut out = stdout.lock();
        // Restore cursor and clear below so the shell prompt appears cleanly.
        write!(out, "\x1b8\x1b[J").map_err(|e| format!("preview: write error: {e}"))?;
        out.flush().map_err(|e| format!("preview: write error: {e}"))?;
        drop(out);
        terminal::disable_raw_mode().map_err(|e| format!("preview: failed to exit raw mode: {e}"))
    }

    /// Read one key event from the terminal (blocking).
    pub(crate) fn read_key() -> Result<KeyEvent, String> {
        loop {
            match event::read().map_err(|e| format!("preview: failed to read key: {e}"))? {
                Event::Key(k) => return Ok(k),
                // Ignore resize and other events; the SIGWINCH handler sets the flag instead.
                _ => continue,
            }
        }
    }
}

impl Drop for LivePreview {
    fn drop(&mut self) {
        // Best-effort cleanup if the caller forgot to call finish() (e.g. on early return).
        let _ = terminal::disable_raw_mode();
    }
}

/// Downscale `img` to fit within the terminal pixel dimensions, preserving aspect ratio.
/// If terminal pixel size is unknown, returns a copy at original dimensions.
fn scale_to_terminal(img: &DynamicImage, term_px: (Option<u32>, Option<u32>)) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (px_w, px_h) = term_px;
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
        img.resize(new_w, new_h, FilterType::Lanczos3).to_rgba8()
    } else {
        img.to_rgba8()
    }
}
