use image::{DynamicImage, RgbaImage};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;

use super::select_lines;
use crate::tui::{Outcome, Screen, is_cancel_key};

const DEGREES: [u16; 3] = [90, 180, 270];
const LABELS: [&str; 3] = ["90° clockwise", "180°", "90° counter-clockwise"];

/// Pick a rotation from a list while the preview shows it applied.
#[derive(Debug, Default)]
pub(crate) struct RotateScreen {
    cursor: usize,
}

impl RotateScreen {
    pub(crate) fn degrees(&self) -> u16 {
        DEGREES[self.cursor]
    }
}

impl Screen for RotateScreen {
    fn title(&self) -> String {
        "rotate".into()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if is_cancel_key(&key) {
            return Outcome::Cancel;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if self.cursor > 0 => {
                self.cursor -= 1;
                Outcome::Redraw
            }
            KeyCode::Down | KeyCode::Char('j') if self.cursor < DEGREES.len() - 1 => {
                self.cursor += 1;
                Outcome::Redraw
            }
            KeyCode::Enter => Outcome::Confirm,
            _ => Outcome::Continue,
        }
    }

    fn frame(&self, base: &RgbaImage) -> RgbaImage {
        let img = DynamicImage::ImageRgba8(base.clone());
        match self.degrees() {
            90 => img.rotate90(),
            180 => img.rotate180(),
            _ => img.rotate270(),
        }
        .into_rgba8()
    }

    fn controls(&self) -> Vec<Line<'static>> {
        select_lines("Rotation", &LABELS, self.cursor)
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![("↑↓", "choose"), ("enter", "save"), ("esc", "cancel")]
    }

    fn result_size(&self, (w, h): (u32, u32)) -> (u32, u32) {
        if self.degrees() == 180 {
            (w, h)
        } else {
            (h, w)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_rotate_screen_stays_in_bounds() {
        let mut s = RotateScreen::default();
        assert_eq!(s.handle_key(key(KeyCode::Up)), Outcome::Continue);
        assert_eq!(s.degrees(), 90);
        s.handle_key(key(KeyCode::Down));
        s.handle_key(key(KeyCode::Char('j')));
        assert_eq!(s.degrees(), 270);
        assert_eq!(s.handle_key(key(KeyCode::Down)), Outcome::Continue);
        assert_eq!(s.degrees(), 270);
        assert_eq!(s.handle_key(key(KeyCode::Char('k'))), Outcome::Redraw);
        assert_eq!(s.degrees(), 180);
    }

    #[test]
    fn test_rotate_screen_enter_confirms_and_esc_cancels() {
        let mut s = RotateScreen::default();
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
        assert_eq!(s.handle_key(key(KeyCode::Esc)), Outcome::Cancel);
    }

    #[test]
    fn test_rotate_screen_frame_swaps_dimensions() {
        let base = RgbaImage::new(4, 2);
        let mut s = RotateScreen::default();
        assert_eq!(s.frame(&base).dimensions(), (2, 4));
        s.handle_key(key(KeyCode::Down));
        assert_eq!(s.frame(&base).dimensions(), (4, 2));
    }

    #[test]
    fn test_rotate_screen_result_size() {
        let mut s = RotateScreen::default();
        assert_eq!(s.result_size((800, 600)), (600, 800));
        s.handle_key(key(KeyCode::Down));
        assert_eq!(s.result_size((800, 600)), (800, 600));
    }
}
