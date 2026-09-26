use image::{DynamicImage, RgbaImage};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;

use super::select_lines;
use crate::commands::transforms::FlipAxis;
use crate::tui::{Outcome, Screen, is_cancel_key};

const DIRECTIONS: [FlipAxis; 2] = [FlipAxis::Horizontal, FlipAxis::Vertical];
const LABELS: [&str; 2] = ["Horizontal (left to right)", "Vertical (top to bottom)"];

/// Pick a flip direction from a list while the preview shows it applied.
#[derive(Debug, Default)]
pub(crate) struct FlipScreen {
    cursor: usize,
}

impl FlipScreen {
    pub(crate) fn direction(&self) -> FlipAxis {
        DIRECTIONS[self.cursor]
    }
}

impl Screen for FlipScreen {
    fn title(&self) -> String {
        "flip".into()
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
            KeyCode::Down | KeyCode::Char('j') if self.cursor < DIRECTIONS.len() - 1 => {
                self.cursor += 1;
                Outcome::Redraw
            }
            KeyCode::Enter => Outcome::Confirm,
            _ => Outcome::Continue,
        }
    }

    fn frame(&self, base: &RgbaImage) -> RgbaImage {
        let img = DynamicImage::ImageRgba8(base.clone());
        match self.direction() {
            FlipAxis::Vertical => img.flipv(),
            FlipAxis::Horizontal => img.fliph(),
        }
        .into_rgba8()
    }

    fn controls(&self) -> Vec<Line<'static>> {
        select_lines("Flip direction", &LABELS, self.cursor)
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![("↑↓", "choose"), ("enter", "save"), ("esc", "cancel")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_flip_screen_stays_in_bounds() {
        let mut s = FlipScreen::default();
        assert_eq!(s.handle_key(key(KeyCode::Up)), Outcome::Continue);
        assert_eq!(s.direction(), FlipAxis::Horizontal);
        assert_eq!(s.handle_key(key(KeyCode::Char('j'))), Outcome::Redraw);
        assert_eq!(s.direction(), FlipAxis::Vertical);
        assert_eq!(s.handle_key(key(KeyCode::Down)), Outcome::Continue);
        assert_eq!(s.handle_key(key(KeyCode::Char('k'))), Outcome::Redraw);
        assert_eq!(s.direction(), FlipAxis::Horizontal);
    }

    #[test]
    fn test_flip_screen_enter_confirms_and_esc_cancels() {
        let mut s = FlipScreen::default();
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
        assert_eq!(s.handle_key(key(KeyCode::Esc)), Outcome::Cancel);
    }

    #[test]
    fn test_flip_screen_frame_flips_in_chosen_direction() {
        let a = Rgba([255, 0, 0, 255]);
        let b = Rgba([0, 0, 255, 255]);
        let row = RgbaImage::from_fn(2, 1, |x, _| if x == 0 { a } else { b });
        let col = RgbaImage::from_fn(1, 2, |_, y| if y == 0 { a } else { b });
        let mut s = FlipScreen::default();
        assert_eq!(*s.frame(&row).get_pixel(0, 0), b);
        assert_eq!(*s.frame(&col).get_pixel(0, 0), a);
        s.handle_key(key(KeyCode::Down));
        assert_eq!(*s.frame(&col).get_pixel(0, 0), b);
        assert_eq!(*s.frame(&row).get_pixel(0, 0), a);
    }
}
