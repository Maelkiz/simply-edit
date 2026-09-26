//! One `Screen` implementation per interactive command.

mod binarize;
mod flip;
mod resize;
mod rotate;

pub(crate) use binarize::BinarizeScreen;
pub(crate) use flip::FlipScreen;
pub(crate) use resize::ResizeScreen;
pub(crate) use rotate::RotateScreen;

use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};

/// Controls for a single-choice list: a dim heading, then `● item` for the cursor and `○ item`
/// for the rest.
pub(super) fn select_lines(heading: &str, items: &[&str], cursor: usize) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(heading.to_string()).add_modifier(Modifier::DIM),
        Line::default(),
    ];
    for (i, item) in items.iter().enumerate() {
        let line = if i == cursor {
            Line::from(vec![
                Span::from(" ● ").fg(Color::Green),
                Span::from(item.to_string()).bold(),
            ])
        } else {
            Line::from(vec![
                Span::from(" ○ ").add_modifier(Modifier::DIM),
                Span::styled(item.to_string(), Style::new().add_modifier(Modifier::DIM)),
            ])
        };
        lines.push(line);
    }
    lines
}
