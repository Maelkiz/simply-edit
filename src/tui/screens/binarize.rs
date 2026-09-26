use image::{DynamicImage, RgbaImage};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};

use crate::commands::transforms::binarize_image;
use crate::tui::{Outcome, Screen, is_cancel_key};

const DEFAULT_THRESHOLD: u8 = 128;
const BIG_STEP: u8 = 10;
const SLIDER_WIDTH: usize = 24;

/// Choose a brightness cutoff with a slider, or by typing a value and pressing Enter.
///
/// Typed digits only take effect on Enter, so the preview never flashes the intermediate values
/// (1, 12, …) on the way to 128.
#[derive(Debug)]
pub(crate) struct BinarizeScreen {
    threshold: u8,
    typed: String,
    show_original: bool,
}

impl Default for BinarizeScreen {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            typed: String::new(),
            show_original: false,
        }
    }
}

impl BinarizeScreen {
    pub(crate) fn threshold(&self) -> u8 {
        self.threshold
    }

    fn set(&mut self, value: u8) -> Outcome {
        self.typed.clear();
        if value == self.threshold {
            return Outcome::Continue;
        }
        self.threshold = value;
        if self.show_original {
            Outcome::Continue
        } else {
            Outcome::Redraw
        }
    }

    fn slider(&self) -> Line<'static> {
        let pos = self.threshold as usize * (SLIDER_WIDTH - 1) / 255;
        Line::from(vec![
            Span::from("━".repeat(pos)).fg(Color::Cyan),
            Span::from("●").fg(Color::Cyan).bold(),
            Span::from("─".repeat(SLIDER_WIDTH - 1 - pos)).add_modifier(Modifier::DIM),
            Span::from(format!(" {:>3}", self.threshold)).bold(),
        ])
    }
}

impl Screen for BinarizeScreen {
    fn title(&self) -> String {
        "binarize".into()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Esc if !self.typed.is_empty() => {
                self.typed.clear();
                Outcome::Continue
            }
            _ if is_cancel_key(&key) => Outcome::Cancel,
            KeyCode::Left if shift => self.set(self.threshold.saturating_sub(BIG_STEP)),
            KeyCode::Right if shift => self.set(self.threshold.saturating_add(BIG_STEP)),
            KeyCode::Char('H') => self.set(self.threshold.saturating_sub(BIG_STEP)),
            KeyCode::Char('L') => self.set(self.threshold.saturating_add(BIG_STEP)),
            KeyCode::Left | KeyCode::Char('h') => self.set(self.threshold.saturating_sub(1)),
            KeyCode::Right | KeyCode::Char('l') => self.set(self.threshold.saturating_add(1)),
            KeyCode::Home => self.set(0),
            KeyCode::End => self.set(255),
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let candidate = format!("{}{c}", self.typed);
                // Refuse digits that would leave the 0–255 range rather than showing an error.
                if candidate.parse::<u16>().is_ok_and(|v| v <= 255) {
                    self.typed = candidate;
                }
                Outcome::Continue
            }
            KeyCode::Backspace => {
                self.typed.pop();
                Outcome::Continue
            }
            KeyCode::Tab => {
                self.show_original = !self.show_original;
                Outcome::Redraw
            }
            KeyCode::Enter if !self.typed.is_empty() => {
                let value = self.typed.parse().expect("typed is validated on input");
                self.set(value)
            }
            KeyCode::Enter => Outcome::Confirm,
            _ => Outcome::Continue,
        }
    }

    fn frame(&self, base: &RgbaImage) -> RgbaImage {
        if self.show_original {
            return base.clone();
        }
        binarize_image(DynamicImage::ImageRgba8(base.clone()), self.threshold).into_rgba8()
    }

    fn controls(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from("Threshold").add_modifier(Modifier::DIM),
            Line::default(),
            self.slider(),
            Line::default(),
        ];
        if !self.typed.is_empty() {
            lines.push(Line::from(vec![
                Span::from("Set to "),
                Span::from(self.typed.clone()).bold(),
                Span::from("▏").fg(Color::Cyan),
                Span::from("  enter to apply").add_modifier(Modifier::DIM),
            ]));
        } else if self.show_original {
            lines.push(Line::from("Showing original").fg(Color::Yellow));
        }
        lines
    }

    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("←→", "adjust"),
            ("shift", "×10"),
            ("0-9", "type"),
            ("tab", "compare"),
            ("enter", "save"),
            ("esc", "cancel"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    fn press(s: &mut BinarizeScreen, keys: &[KeyEvent]) -> Outcome {
        keys.iter().fold(Outcome::Continue, |_, k| s.handle_key(*k))
    }

    #[test]
    fn test_binarize_screen_arrows_step_and_saturate() {
        let mut s = BinarizeScreen::default();
        assert_eq!(s.handle_key(key(KeyCode::Right)), Outcome::Redraw);
        assert_eq!(s.threshold(), 129);
        s.handle_key(key(KeyCode::Home));
        assert_eq!(s.handle_key(key(KeyCode::Left)), Outcome::Continue);
        assert_eq!(s.threshold(), 0);
        s.handle_key(key(KeyCode::End));
        assert_eq!(s.handle_key(key(KeyCode::Char('l'))), Outcome::Continue);
        assert_eq!(s.threshold(), 255);
    }

    #[test]
    fn test_binarize_screen_shift_steps_by_ten() {
        let mut s = BinarizeScreen::default();
        s.handle_key(shift(KeyCode::Right));
        assert_eq!(s.threshold(), 138);
        s.handle_key(key(KeyCode::Char('H')));
        s.handle_key(key(KeyCode::Char('H')));
        assert_eq!(s.threshold(), 118);
        s.handle_key(key(KeyCode::End));
        s.handle_key(shift(KeyCode::Right));
        assert_eq!(s.threshold(), 255);
    }

    #[test]
    fn test_binarize_screen_typed_value_applies_only_on_enter() {
        let mut s = BinarizeScreen::default();
        s.handle_key(key(KeyCode::Home));
        let digits = [
            key(KeyCode::Char('2')),
            key(KeyCode::Char('0')),
            key(KeyCode::Char('0')),
        ];
        assert_eq!(press(&mut s, &digits), Outcome::Continue);
        assert_eq!(s.threshold(), 0);
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Redraw);
        assert_eq!(s.threshold(), 200);
        // The first Enter applied the value; the next one confirms.
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
    }

    #[test]
    fn test_binarize_screen_rejects_out_of_range_digits() {
        let mut s = BinarizeScreen::default();
        press(
            &mut s,
            &[
                key(KeyCode::Char('3')),
                key(KeyCode::Char('0')),
                key(KeyCode::Char('0')),
            ],
        );
        s.handle_key(key(KeyCode::Enter));
        assert_eq!(s.threshold(), 30);
    }

    #[test]
    fn test_binarize_screen_esc_clears_typing_before_cancelling() {
        let mut s = BinarizeScreen::default();
        s.handle_key(key(KeyCode::Char('5')));
        assert_eq!(s.handle_key(key(KeyCode::Esc)), Outcome::Continue);
        assert_eq!(s.handle_key(key(KeyCode::Enter)), Outcome::Confirm);
        assert_eq!(s.threshold(), 128);
        assert_eq!(s.handle_key(key(KeyCode::Esc)), Outcome::Cancel);
    }

    #[test]
    fn test_binarize_screen_tab_toggles_original() {
        let base = RgbaImage::from_pixel(1, 1, Rgba([100, 100, 100, 255]));
        let mut s = BinarizeScreen::default();
        assert_eq!(*s.frame(&base).get_pixel(0, 0), Rgba([0, 0, 0, 255]));
        assert_eq!(s.handle_key(key(KeyCode::Tab)), Outcome::Redraw);
        assert_eq!(*s.frame(&base).get_pixel(0, 0), Rgba([100, 100, 100, 255]));
        // Adjusting while viewing the original needs no preview rebuild.
        assert_eq!(s.handle_key(key(KeyCode::Right)), Outcome::Continue);
    }
}
