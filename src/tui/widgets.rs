use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};

/// Terminal width at which the controls panel moves beside the preview instead of under it.
pub(crate) const SIDE_PANEL_MIN_WIDTH: u16 = 100;
const SIDE_PANEL_WIDTH: u16 = 34;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Areas {
    pub(crate) header: Rect,
    pub(crate) preview: Rect,
    pub(crate) controls: Rect,
    pub(crate) footer: Rect,
}

/// Split the screen into header, preview, controls and footer.
///
/// `control_lines` is the number of lines the controls panel needs, excluding its border.
pub(crate) fn layout(area: Rect, control_lines: u16) -> Areas {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    let (preview, controls) = if area.width >= SIDE_PANEL_MIN_WIDTH {
        let [preview, controls] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(SIDE_PANEL_WIDTH)])
                .areas(body);
        (preview, controls)
    } else {
        let [preview, controls] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(control_lines + 2)])
                .areas(body);
        (preview, controls)
    };
    Areas {
        header,
        preview,
        controls,
        footer,
    }
}

/// `title · file · 800×600 → 600×800`, with the arrow part only when the size changes.
pub(crate) fn render_header(
    f: &mut Frame,
    area: Rect,
    title: &str,
    file: &str,
    size: (u32, u32),
    result: (u32, u32),
) {
    let dim = Style::new().add_modifier(Modifier::DIM);
    let mut spans = vec![
        Span::from(format!(" {title}")).bold().fg(Color::Cyan),
        Span::styled(" · ", dim),
        Span::from(file.to_string()),
        Span::styled(" · ", dim),
        Span::from(format!("{}×{}", size.0, size.1)),
    ];
    if result != size {
        spans.push(Span::styled(" → ", dim));
        spans.push(Span::from(format!("{}×{}", result.0, result.1)).bold());
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Key hints such as `enter save · esc cancel`, keys highlighted and actions dimmed.
pub(crate) fn render_footer(f: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let mut spans = vec![Span::from(" ")];
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::from("  ·  ").add_modifier(Modifier::DIM));
        }
        spans.push(Span::from(key.to_string()).bold().fg(Color::Cyan));
        spans.push(Span::from(format!(" {action}")).add_modifier(Modifier::DIM));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(crate) fn render_controls(f: &mut Frame, area: Rect, lines: Vec<Line<'static>>) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Draw the preview scaled to fit `area` and centred in it.
pub(crate) fn render_preview(f: &mut Frame, area: Rect, state: &mut StatefulProtocol) {
    let resize = Resize::Scale(Some(image::imageops::FilterType::Triangle));
    let size = state.size_for(resize.clone(), area.as_size());
    let target = center(area, size.width, size.height);
    f.render_stateful_widget(StatefulImage::default().resize(resize), target, state);
}

fn center(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn row_text(terminal: &Terminal<TestBackend>, y: u16) -> String {
        let buf = terminal.backend().buffer();
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect::<String>()
    }

    #[test]
    fn test_layout_wide_puts_controls_beside_preview() {
        let a = layout(Rect::new(0, 0, 120, 40), 3);
        assert_eq!(a.header.height, 1);
        assert_eq!(a.footer.y, 39);
        assert_eq!(a.controls.width, SIDE_PANEL_WIDTH);
        assert_eq!(a.controls.y, a.preview.y);
        assert!(a.controls.x > a.preview.x);
    }

    #[test]
    fn test_layout_narrow_puts_controls_below_preview() {
        let a = layout(Rect::new(0, 0, 80, 40), 3);
        assert_eq!(a.controls.width, 80);
        assert_eq!(a.controls.height, 5);
        assert!(a.controls.y > a.preview.y);
    }

    #[test]
    fn test_center_is_centred_and_clamped() {
        let area = Rect::new(0, 0, 10, 10);
        assert_eq!(center(area, 4, 2), Rect::new(3, 4, 4, 2));
        assert_eq!(center(area, 20, 20), area);
    }

    #[test]
    fn test_header_shows_size_change() {
        let mut t = Terminal::new(TestBackend::new(60, 1)).unwrap();
        t.draw(|f| render_header(f, f.area(), "rotate", "a.png", (800, 600), (600, 800)))
            .unwrap();
        assert!(row_text(&t, 0).contains("rotate · a.png · 800×600 → 600×800"));
    }

    #[test]
    fn test_header_omits_arrow_when_size_unchanged() {
        let mut t = Terminal::new(TestBackend::new(60, 1)).unwrap();
        t.draw(|f| render_header(f, f.area(), "flip", "a.png", (8, 6), (8, 6)))
            .unwrap();
        assert!(!row_text(&t, 0).contains('→'));
    }

    #[test]
    fn test_footer_lists_hints() {
        let mut t = Terminal::new(TestBackend::new(60, 1)).unwrap();
        t.draw(|f| render_footer(f, f.area(), &[("enter", "save"), ("esc", "cancel")]))
            .unwrap();
        assert!(row_text(&t, 0).contains("enter save  ·  esc cancel"));
    }
}
