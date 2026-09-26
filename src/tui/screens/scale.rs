use image::RgbaImage;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};

use crate::commands::transforms::scaled_size;
use crate::tui::{Outcome, Screen, is_cancel_key};

const STEP: f64 = 0.1;
const BIG_STEP: f64 = 0.5;
const MIN_FACTOR: f64 = 0.01;
const MAX_INPUT_LEN: usize = 8;

/// Enter a uniform scale factor while the header and controls show the resulting size.
///
/// A uniform scale doesn't change how the image looks, so the preview stays put and only the
/// numbers update. As in the resize form, the first key typed replaces the prefilled value.
#[derive(Debug)]
pub(crate) struct ScaleScreen {
    orig: (u32, u32),
    input: String,
    fresh: bool,
    error: bool,
}

impl ScaleScreen {
    pub(crate) fn new(orig: (u32, u32)) -> Self {
        Self {
            orig,
            input: "1".into(),
            fresh: true,
            error: false,
        }
    }

    /// The typed factor, or `None` unless it is a finite number greater than 0.
    pub(crate) fn factor(&self) -> Option<f32> {
        self.input
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
    }

    fn step(&mut self, delta: f64) {
        let current = self.factor().map_or(1.0, f64::from);
        // Round to hundredths so repeated steps don't accumulate float noise.
        let next = ((current + delta).max(MIN_FACTOR) * 100.0).round() / 100.0;
        self.input = format_factor(next);
        self.fresh = true;
        self.error = false;
    }

    fn type_char(&mut self, c: char) {
        let candidate = if self.fresh {
            if c == '.' { "0.".into() } else { c.to_string() }
        } else {
            format!("{}{c}", self.input)
        };
        if candidate.len() <= MAX_INPUT_LEN && candidate.matches('.').count() <= 1 {
            self.input = candidate;
        }
        self.fresh = false;
        self.error = false;
    }
}

/// `1.50` → `1.5`, `2.00` → `2`.
fn format_factor(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl Screen for ScaleScreen {
    fn title(&self) -> String {
        "scale".into()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if is_cancel_key(&key) {
            return Outcome::Cancel;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Left if shift => self.step(-BIG_STEP),
            KeyCode::Right if shift => self.step(BIG_STEP),
            KeyCode::Char('H') => self.step(-BIG_STEP),
            KeyCode::Char('L') => self.step(BIG_STEP),
            KeyCode::Left | KeyCode::Char('h') => self.step(-STEP),
            KeyCode::Right | KeyCode::Char('l') => self.step(STEP),
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => self.type_char(c),
            KeyCode::Backspace => {
                self.input.pop();
                self.fresh = false;
                self.error = false;
            }
            KeyCode::Enter if self.factor().is_some() => return Outcome::Confirm,
            KeyCode::Enter => self.error = true,
            _ => {}
        }
        // The preview never changes; the header and controls redraw every loop anyway.
        Outcome::Continue
    }

    fn frame(&self, base: &RgbaImage) -> RgbaImage {
        base.clone()
    }

    fn controls(&self) -> Vec<Line<'static>> {
        let result = match self.factor() {
            Some(f) => {
                let (w, h) = scaled_size(self.orig.0, self.orig.1, f, f);
                Span::from(format!("{w} × {h} px")).bold()
            }
            None => Span::from("—").add_modifier(Modifier::DIM),
        };
        let mut lines = vec![
            Line::from("Scale factor").add_modifier(Modifier::DIM),
            Line::default(),
            Line::from(vec![
                Span::from(" × ").add_modifier(Modifier::DIM),
                Span::from(self.input.clone()).fg(Color::Cyan).bold(),
                Span::from("▏").fg(Color::Cyan),
            ]),
            Line::default(),
            Line::from(vec![
                Span::from(" Result  ").add_modifier(Modifier::DIM),
                result,
            ]),
        ];
        if self.error {
            lines.push(Line::default());
            lines.push(Line::from("Enter a number greater than 0").fg(Color::Red));
        }
        lines
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("←→", "±0.1"),
            ("shift", "±0.5"),
            ("0-9 .", "type"),
            ("enter", "save"),
            ("esc", "cancel"),
        ]
    }

    fn result_size(&self, size: (u32, u32)) -> (u32, u32) {
        match self.factor() {
            Some(f) => scaled_size(self.orig.0, self.orig.1, f, f),
            None => size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_str(s: &mut ScaleScreen, text: &str) {
        for c in text.chars() {
            s.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn test_scale_screen_typing_replaces_prefill() {
        let mut s = ScaleScreen::new((800, 600));
        type_str(&mut s, "0.25");
        assert_eq!(s.factor(), Some(0.25));
        let mut s = ScaleScreen::new((800, 600));
        type_str(&mut s, ".5");
        assert_eq!(s.factor(), Some(0.5));
    }

    #[test]
    fn test_scale_screen_rejects_second_dot_and_long_input() {
        let mut s = ScaleScreen::new((800, 600));
        type_str(&mut s, "1.2.3");
        assert_eq!(s.factor(), Some(1.23));
        type_str(&mut s, "456789");
        assert_eq!(s.input.len(), MAX_INPUT_LEN);
    }

    #[test]
    fn test_scale_screen_steps_and_clamps() {
        let mut s = ScaleScreen::new((800, 600));
        s.handle_key(key(KeyCode::Right));
        assert_eq!(s.factor(), Some(1.1));
        s.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
        assert_eq!(s.factor(), Some(1.6));
        for _ in 0..5 {
            s.handle_key(key(KeyCode::Char('H')));
        }
        assert_eq!(s.factor(), Some(0.01));
        s.handle_key(key(KeyCode::Char('l')));
        assert_eq!(s.factor(), Some(0.11));
    }

    #[test]
    fn test_scale_screen_blocks_enter_on_invalid_factor() {
        let mut s = ScaleScreen::new((800, 600));
        type_str(&mut s, "0");
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Continue);
        assert!(s.error);
        s.handle_key(key(KeyCode::Backspace));
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Continue);
        type_str(&mut s, "2");
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
    }

    #[test]
    fn test_scale_screen_result_size_matches_run_scale_rounding() {
        let mut s = ScaleScreen::new((3, 5));
        type_str(&mut s, "0.5");
        // 1.5 → 2 and 2.5 → 3, rounding half away from zero like `run_scale`.
        assert_eq!(s.result_size((3, 5)), (2, 3));
        // Tiny factors never produce a zero-sized image.
        let mut s = ScaleScreen::new((3, 5));
        type_str(&mut s, "0.001");
        assert_eq!(s.result_size((3, 5)), (1, 1));
    }
}
