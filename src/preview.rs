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

        // Save cursor position, hide the real cursor (we use a reverse-video block instead).
        let stdout = io::stdout();
        let mut out = stdout.lock();
        write!(out, "\x1b7\x1b[?25l").map_err(|e| format!("preview: write error: {e}"))?;
        out.flush().map_err(|e| format!("preview: write error: {e}"))?;
        drop(out);

        let term_px = terminal_pixel_size();
        let scaled = scale_to_terminal(img, term_px);
        Ok(Self { scaled, term_px })
    }

    /// Restore cursor to the saved position, clear below, and render the RGBA buffer.
    /// After this call the cursor is positioned at the start of the line below the image,
    /// ready for the caller to print a prompt.
    pub(crate) fn render(&self, rgba: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<(), String> {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        write!(out, "\x1b8\x1b[J").map_err(|e| format!("preview: write error: {e}"))?;
        out.flush().map_err(|e| format!("preview: write error: {e}"))?;
        drop(out);
        display_raw_rgba(rgba.width(), rgba.height(), rgba.as_raw())
    }

    /// Clear the SIGWINCH flag, re-query terminal size, and re-downscale `img`.
    /// The caller is responsible for applying any transform and calling `render` afterwards.
    pub(crate) fn handle_resize(&mut self, img: &DynamicImage) {
        RESIZED.store(false, Ordering::Relaxed);
        self.term_px = terminal_pixel_size();
        self.scaled = scale_to_terminal(img, self.term_px);
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
        // Restore cursor position, clear below, and show the real cursor again.
        write!(out, "\x1b8\x1b[J\x1b[?25h").map_err(|e| format!("preview: write error: {e}"))?;
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

/// Print a cliclack-styled active input prompt below a live preview image.
///
/// Renders the three-line cliclack active-input layout:
///   ◆  {label}          ← cyan ◆, two spaces, prompt label
///   │  {typed}█         ← cyan │, two spaces, current input, reverse-video cursor block
///   └                   ← cyan └
///
/// Raw mode disables output post-processing, so `\r\n` is used for line breaks.
pub(crate) fn print_prompt(label: &str, typed: &str) -> Result<(), String> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write!(
        out,
        "\r\x1b[36m◆\x1b[0m  {label}\r\n\
         \r\x1b[36m│\x1b[0m  {typed}\x1b[7m \x1b[0m\r\n\
         \r\x1b[36m└\x1b[0m  \r\n"
    )
    .map_err(|e| format!("preview: write error: {e}"))?;
    out.flush().map_err(|e| format!("preview: write error: {e}"))
}

impl Drop for LivePreview {
    fn drop(&mut self) {
        // Best-effort cleanup if the caller forgot to call finish() (e.g. on early return).
        let _ = write!(io::stdout(), "\x1b[?25h");
        let _ = terminal::disable_raw_mode();
    }
}

/// Fraction of terminal pixel height reserved for the live preview.
/// Keeping this constant ensures the preview occupies a stable number of terminal rows
/// regardless of image dimensions or orientation.
const PREVIEW_HEIGHT_FRACTION: f32 = 0.65;

/// Scale `img` to fit the preview pixel budget, then pad to exactly `budget_h` tall.
/// The budget is `PREVIEW_HEIGHT_FRACTION × term_h_px`, so the rendered area is always
/// the same height in terminal rows even when the image is rotated or resized.
fn scale_to_terminal(img: &DynamicImage, term_px: (Option<u32>, Option<u32>)) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (px_w, px_h) = term_px;
    let budget_h = px_h.map(|h| ((h as f32 * PREVIEW_HEIGHT_FRACTION).round() as u32).max(1));

    let scale_w = px_w.filter(|&w| img.width() > w).map(|w| w as f32 / img.width() as f32);
    let scale_h = budget_h.filter(|&h| img.height() > h).map(|h| h as f32 / img.height() as f32);
    let scale = match (scale_w, scale_h) {
        (Some(sw), Some(sh)) => Some(sw.min(sh)),
        (Some(sw), None) => Some(sw),
        (None, Some(sh)) => Some(sh),
        (None, None) => None,
    };
    let scaled = if let Some(s) = scale {
        let new_w = ((img.width() as f32 * s).round() as u32).max(1);
        let new_h = ((img.height() as f32 * s).round() as u32).max(1);
        img.resize(new_w, new_h, FilterType::Lanczos3).to_rgba8()
    } else {
        img.to_rgba8()
    };

    // Pad to exactly budget_h so the preview always occupies the same number of terminal rows.
    if let Some(bh) = budget_h {
        if scaled.height() < bh {
            let mut canvas = ImageBuffer::from_pixel(scaled.width(), bh, Rgba([0, 0, 0, 0]));
            image::imageops::overlay(&mut canvas, &scaled, 0, 0);
            return canvas;
        }
    }
    scaled
}
