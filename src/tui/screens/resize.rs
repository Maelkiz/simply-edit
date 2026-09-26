use image::RgbaImage;
use image::imageops::{self, FilterType};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};

use crate::commands::transforms::scaled_other_side;
use crate::tui::{Outcome, Screen, is_cancel_key};

/// The focusable rows of the form, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Width,
    Height,
    Aspect,
}

const ROWS: [Row; 3] = [Row::Width, Row::Height, Row::Aspect];

/// Enter a target width and height, optionally locked to the original aspect ratio, while the
/// preview shows the resulting shape.
///
/// A field that just received focus is "fresh": the first digit replaces its value instead of
/// appending, so a prefilled size can be overwritten without backspacing it away first.
#[derive(Debug)]
pub(crate) struct ResizeScreen {
    orig: (u32, u32),
    width: String,
    height: String,
    focus: Row,
    /// The size field edited last; with the aspect locked, the other one follows it.
    anchor: Row,
    lock_aspect: bool,
    fresh: bool,
    error: bool,
}

impl ResizeScreen {
    /// Start from whatever the command line already gave. With nothing given, both fields hold
    /// the original size; with one side given, the other follows the aspect ratio.
    pub(crate) fn new(orig: (u32, u32), width: Option<u32>, height: Option<u32>) -> Self {
        let focus = if width.is_none() && height.is_some() {
            Row::Height
        } else {
            Row::Width
        };
        let mut screen = Self {
            orig,
            width: width.unwrap_or(orig.0).to_string(),
            height: height.unwrap_or(orig.1).to_string(),
            focus,
            anchor: focus,
            lock_aspect: true,
            fresh: true,
            error: false,
        };
        screen.sync();
        screen
    }

    /// The chosen size, or `None` while either field is empty or zero.
    pub(crate) fn dims(&self) -> Option<(u32, u32)> {
        let parse = |s: &str| s.parse::<u32>().ok().filter(|&v| v > 0);
        Some((parse(&self.width)?, parse(&self.height)?))
    }

    fn move_focus(&mut self, forward: bool) {
        let i = ROWS.iter().position(|&r| r == self.focus).unwrap_or(0);
        let n = ROWS.len();
        self.focus = ROWS[if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        }];
        if self.focus != Row::Aspect {
            self.anchor = self.focus;
        }
        self.fresh = true;
    }

    fn toggle_aspect(&mut self) -> Outcome {
        self.lock_aspect = !self.lock_aspect;
        self.sync();
        Outcome::Redraw
    }

    /// With the aspect ratio locked, recompute the other field from the anchor field.
    fn sync(&mut self) {
        if !self.lock_aspect {
            return;
        }
        let (ow, oh) = self.orig;
        match self.anchor {
            Row::Height => {
                if let Some(h) = self.height.parse::<u32>().ok().filter(|&v| v > 0) {
                    self.width = scaled_other_side(h, oh, ow).to_string();
                }
            }
            _ => {
                if let Some(w) = self.width.parse::<u32>().ok().filter(|&v| v > 0) {
                    self.height = scaled_other_side(w, ow, oh).to_string();
                }
            }
        }
    }

    fn field_line(&self, label: &str, row: Row) -> Line<'static> {
        let value = match row {
            Row::Height => &self.height,
            _ => &self.width,
        };
        let focused = row == self.focus;
        let mut spans = vec![
            if focused {
                Span::from(format!(" {label:<8}")).bold()
            } else {
                Span::from(format!(" {label:<8}")).add_modifier(Modifier::DIM)
            },
            if focused {
                Span::from(value.clone()).fg(Color::Cyan).bold()
            } else {
                Span::from(value.clone())
            },
        ];
        if focused {
            spans.push(Span::from("▏").fg(Color::Cyan));
        }
        spans.push(Span::from(" px").add_modifier(Modifier::DIM));
        Line::from(spans)
    }

    fn aspect_line(&self) -> Line<'static> {
        let focused = self.focus == Row::Aspect;
        let mark = if self.lock_aspect { "[x]" } else { "[ ]" };
        let text = "Keep aspect ratio";
        if focused {
            Line::from(vec![
                Span::from(format!(" {mark} ")).fg(Color::Cyan).bold(),
                Span::from(text).bold(),
                Span::from("  space to toggle").add_modifier(Modifier::DIM),
            ])
        } else {
            Line::from(format!(" {mark} {text}")).add_modifier(Modifier::DIM)
        }
    }
}

impl Screen for ResizeScreen {
    fn title(&self) -> String {
        "resize".into()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if is_cancel_key(&key) {
            return Outcome::Cancel;
        }
        match key.code {
            KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                self.move_focus(true);
                Outcome::Continue
            }
            KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                self.move_focus(false);
                Outcome::Continue
            }
            KeyCode::Char(' ') | KeyCode::Char('a') => self.toggle_aspect(),
            KeyCode::Enter if self.dims().is_some() => Outcome::Confirm,
            KeyCode::Enter => {
                self.error = true;
                Outcome::Continue
            }
            _ if self.focus == Row::Aspect => Outcome::Continue,
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let field = if self.focus == Row::Height {
                    &mut self.height
                } else {
                    &mut self.width
                };
                let candidate = if self.fresh || field == "0" {
                    c.to_string()
                } else {
                    format!("{field}{c}")
                };
                // Refuse digits that would overflow rather than showing an error.
                if candidate.parse::<u32>().is_err() {
                    return Outcome::Continue;
                }
                *field = candidate;
                self.fresh = false;
                self.error = false;
                self.sync();
                Outcome::Redraw
            }
            KeyCode::Backspace => {
                if self.focus == Row::Height {
                    self.height.pop();
                } else {
                    self.width.pop();
                }
                self.fresh = false;
                self.error = false;
                self.sync();
                Outcome::Redraw
            }
            _ => Outcome::Continue,
        }
    }

    /// The base stretched to the target aspect ratio, fitted in a square as large as the base's
    /// longest side so neither a wide nor a tall result gets clipped.
    fn frame(&self, base: &RgbaImage) -> RgbaImage {
        let Some((w, h)) = self.dims() else {
            return base.clone();
        };
        let side = base.width().max(base.height()) as f64;
        let scale = f64::min(side / w as f64, side / h as f64);
        let fw = ((w as f64 * scale).round() as u32).max(1);
        let fh = ((h as f64 * scale).round() as u32).max(1);
        imageops::resize(base, fw, fh, FilterType::Triangle)
    }

    fn controls(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from("Size").add_modifier(Modifier::DIM),
            Line::default(),
            self.field_line("Width", Row::Width),
            self.field_line("Height", Row::Height),
            Line::default(),
            self.aspect_line(),
        ];
        if self.error {
            lines.push(Line::default());
            lines.push(Line::from("Width and height must be greater than 0").fg(Color::Red));
        }
        lines
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓", "move"),
            ("0-9", "type"),
            ("space", "keep ratio on/off"),
            ("enter", "save"),
            ("esc", "cancel"),
        ]
    }

    fn result_size(&self, size: (u32, u32)) -> (u32, u32) {
        self.dims().unwrap_or(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(s: &mut ResizeScreen, text: &str) {
        for c in text.chars() {
            s.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn test_resize_screen_starts_at_original_size() {
        let s = ResizeScreen::new((800, 600), None, None);
        assert_eq!(s.dims(), Some((800, 600)));
    }

    #[test]
    fn test_resize_screen_prefills_partial_argument_with_aspect() {
        assert_eq!(
            ResizeScreen::new((800, 600), Some(400), None).dims(),
            Some((400, 300))
        );
        assert_eq!(
            ResizeScreen::new((800, 600), None, Some(300)).dims(),
            Some((400, 300))
        );
    }

    #[test]
    fn test_resize_screen_first_digit_replaces_then_appends() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        type_str(&mut s, "40");
        assert_eq!(s.dims(), Some((40, 30)));
        s.handle_key(key(KeyCode::Char('0')));
        assert_eq!(s.dims(), Some((400, 300)));
    }

    #[test]
    fn test_resize_screen_locked_height_recomputes_width() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Tab));
        type_str(&mut s, "150");
        assert_eq!(s.dims(), Some((200, 150)));
    }

    #[test]
    fn test_resize_screen_unlocked_fields_are_independent() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Char(' ')));
        type_str(&mut s, "100");
        assert_eq!(s.dims(), Some((100, 600)));
        // Locking again snaps the other field back to the ratio.
        s.handle_key(key(KeyCode::Char('a')));
        assert_eq!(s.dims(), Some((100, 75)));
    }

    #[test]
    fn test_resize_screen_aspect_row_is_reachable_and_toggles() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Down));
        s.handle_key(key(KeyCode::Down));
        assert_eq!(s.focus, Row::Aspect);
        assert_eq!(s.handle_key(key(KeyCode::Char(' '))), Outcome::Redraw);
        assert!(!s.lock_aspect);
        // Digits do nothing on the toggle row.
        assert_eq!(s.handle_key(key(KeyCode::Char('5'))), Outcome::Continue);
        assert_eq!(s.dims(), Some((800, 600)));
        // Wraps back to the width field.
        s.handle_key(key(KeyCode::Down));
        assert_eq!(s.focus, Row::Width);
        s.handle_key(key(KeyCode::Up));
        assert_eq!(s.focus, Row::Aspect);
    }

    #[test]
    fn test_resize_screen_relock_follows_last_edited_field() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Char(' ')));
        s.handle_key(key(KeyCode::Down));
        type_str(&mut s, "300");
        s.handle_key(key(KeyCode::Down));
        s.handle_key(key(KeyCode::Char(' ')));
        assert_eq!(s.dims(), Some((400, 300)));
    }

    #[test]
    fn test_resize_screen_blocks_enter_on_empty_or_zero() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Backspace));
        s.handle_key(key(KeyCode::Backspace));
        s.handle_key(key(KeyCode::Backspace));
        assert_eq!(s.dims(), None);
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Continue);
        type_str(&mut s, "0");
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Continue);
        type_str(&mut s, "5");
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
    }

    #[test]
    fn test_resize_screen_rejects_overflowing_digits() {
        let mut s = ResizeScreen::new((800, 600), None, None);
        s.handle_key(key(KeyCode::Char(' ')));
        type_str(&mut s, "99999999999");
        assert_eq!(s.dims(), Some((999_999_999, 600)));
    }

    #[test]
    fn test_resize_screen_frame_matches_target_aspect() {
        let base = RgbaImage::new(100, 50);
        let mut s = ResizeScreen::new((100, 50), None, None);
        assert_eq!(s.frame(&base).dimensions(), (100, 50));
        s.handle_key(key(KeyCode::Char(' ')));
        s.handle_key(key(KeyCode::Tab));
        type_str(&mut s, "200");
        assert_eq!(s.frame(&base).dimensions(), (50, 100));
    }
}
