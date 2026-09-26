use std::time::Duration;

use image::{DynamicImage, RgbaImage};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Line;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

use super::preview_source::{checkerboard_composite, prepare_base};
use super::terminal::TuiSession;
use super::widgets;

/// What the event loop should do after a screen has handled a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Nothing visible changed.
    Continue,
    /// The screen state changed and the preview must be rebuilt.
    Redraw,
    /// The user accepted the current state.
    Confirm,
    /// The user backed out; nothing should be saved.
    Cancel,
}

/// One interactive prompt: its state, how keys change it, and how it is drawn.
///
/// `handle_key` must be pure (no I/O) so screens can be tested without a terminal.
pub(crate) trait Screen {
    /// Short heading shown at the top of the screen.
    fn title(&self) -> String;
    /// Update state for one key press. Ctrl+C is handled by the event loop and never reaches here.
    fn handle_key(&mut self, key: KeyEvent) -> Outcome;
    /// Produce the preview image for the current state from the downscaled source.
    fn frame(&self, base: &RgbaImage) -> RgbaImage;
    /// Lines describing the current state (select list, slider, fields).
    fn controls(&self) -> Vec<Line<'static>>;
    /// Key hints for the footer as `(key, action)` pairs.
    fn hints(&self) -> Vec<(&'static str, &'static str)>;
    /// Output dimensions for a source of `size`, shown in the header. Defaults to unchanged.
    fn result_size(&self, size: (u32, u32)) -> (u32, u32) {
        size
    }
}

/// Esc and `q` are the conventional ways to back out of a screen.
pub(crate) fn is_cancel_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
}

fn is_interrupt(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

/// Apply every pending key to `screen` and return the single outcome to act on.
///
/// Confirm and Cancel stop processing immediately; otherwise any Redraw wins over Continue, so a
/// burst of held-down keys produces one redraw instead of one per key.
pub(crate) fn apply_keys<S: Screen>(
    screen: &mut S,
    keys: impl IntoIterator<Item = KeyEvent>,
) -> Outcome {
    let mut outcome = Outcome::Continue;
    for key in keys {
        if is_interrupt(&key) {
            return Outcome::Cancel;
        }
        match screen.handle_key(key) {
            Outcome::Continue => {}
            Outcome::Redraw => outcome = Outcome::Redraw,
            done @ (Outcome::Confirm | Outcome::Cancel) => return done,
        }
    }
    outcome
}

/// Block for the next event, then drain everything else already queued.
fn read_batch() -> Result<Vec<Event>, String> {
    let err = |e: std::io::Error| format!("failed to read terminal input: {e}");
    let mut events = vec![event::read().map_err(err)?];
    while event::poll(Duration::ZERO).map_err(err)? {
        events.push(event::read().map_err(err)?);
    }
    Ok(events)
}

/// Run `screen` full-screen until the user confirms or cancels.
///
/// `file` is the name shown in the header. Returns the final screen state on confirm and `None`
/// on cancel. The terminal is restored before this returns, so callers can print normally
/// afterwards.
pub(crate) fn run_screen<S: Screen>(
    file: &str,
    img: &DynamicImage,
    mut screen: S,
) -> Result<Option<S>, String> {
    let base = prepare_base(img);
    let size = (img.width(), img.height());

    let mut session = TuiSession::enter()?;
    // Must run after entering the alternate screen and before reading any events: it writes
    // queries to the terminal and reads the replies from stdin.
    let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
    let build = |screen: &S| {
        let frame = checkerboard_composite(&screen.frame(&base));
        picker.new_resize_protocol(DynamicImage::ImageRgba8(frame))
    };
    let mut preview = build(&screen);

    loop {
        session
            .terminal
            .draw(|f| draw(f, &screen, file, size, &mut preview))
            .map_err(|e| format!("failed to draw: {e}"))?;

        // A resize alone needs no new preview; the redraw at the top of the loop re-fits it.
        let keys = read_batch()?.into_iter().filter_map(|e| match e {
            Event::Key(k) if k.kind == KeyEventKind::Press => Some(k),
            _ => None,
        });
        match apply_keys(&mut screen, keys) {
            Outcome::Confirm => return Ok(Some(screen)),
            Outcome::Cancel => return Ok(None),
            Outcome::Redraw => preview = build(&screen),
            Outcome::Continue => {}
        }
    }
}

fn draw<S: Screen>(
    f: &mut Frame,
    screen: &S,
    file: &str,
    size: (u32, u32),
    preview: &mut StatefulProtocol,
) {
    let controls = screen.controls();
    let areas = widgets::layout(f.area(), controls.len() as u16);
    widgets::render_header(
        f,
        areas.header,
        &screen.title(),
        file,
        size,
        screen.result_size(size),
    );
    widgets::render_preview(f, areas.preview, preview);
    widgets::render_controls(f, areas.controls, controls);
    widgets::render_footer(f, areas.footer, &screen.hints());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal screen: a counter moved with Right, confirmed with Enter.
    #[derive(Default)]
    struct Counter {
        value: u32,
    }

    impl Screen for Counter {
        fn title(&self) -> String {
            "counter".into()
        }
        fn handle_key(&mut self, key: KeyEvent) -> Outcome {
            if is_cancel_key(&key) {
                return Outcome::Cancel;
            }
            match key.code {
                KeyCode::Right => {
                    self.value += 1;
                    Outcome::Redraw
                }
                KeyCode::Enter => Outcome::Confirm,
                _ => Outcome::Continue,
            }
        }
        fn frame(&self, base: &RgbaImage) -> RgbaImage {
            base.clone()
        }
        fn controls(&self) -> Vec<Line<'static>> {
            vec![Line::from(self.value.to_string())]
        }
        fn hints(&self) -> Vec<(&'static str, &'static str)> {
            vec![]
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_apply_keys_coalesces_into_one_redraw() {
        let mut s = Counter::default();
        let out = apply_keys(&mut s, [key(KeyCode::Right); 3]);
        assert_eq!(out, Outcome::Redraw);
        assert_eq!(s.value, 3);
    }

    #[test]
    fn test_apply_keys_ignored_keys_continue() {
        let mut s = Counter::default();
        assert_eq!(
            apply_keys(&mut s, [key(KeyCode::Char('x'))]),
            Outcome::Continue
        );
    }

    #[test]
    fn test_apply_keys_esc_cancels() {
        let mut s = Counter::default();
        assert_eq!(apply_keys(&mut s, [key(KeyCode::Esc)]), Outcome::Cancel);
    }

    #[test]
    fn test_apply_keys_ctrl_c_cancels_without_reaching_screen() {
        let mut s = Counter::default();
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let out = apply_keys(&mut s, [ctrl_c, key(KeyCode::Right)]);
        assert_eq!(out, Outcome::Cancel);
        assert_eq!(s.value, 0);
    }

    #[test]
    fn test_apply_keys_stops_at_confirm() {
        let mut s = Counter::default();
        let out = apply_keys(
            &mut s,
            [
                key(KeyCode::Right),
                key(KeyCode::Enter),
                key(KeyCode::Right),
            ],
        );
        assert_eq!(out, Outcome::Confirm);
        assert_eq!(s.value, 1);
    }
}
