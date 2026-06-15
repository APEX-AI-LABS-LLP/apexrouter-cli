use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

use crate::tui::theme::Theme;

const BANNER_ART: [&str; 6] = [
    "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
    "┃                                                                   ┃",
    "┃                          APEXROUTER CLI                           ┃",
    "┃          Orchestrating Models, Agents & Developer Tools           ┃",
    "┃                                                                   ┃",
    "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
];

const BANNER_ROWS: u16 = 6;
const BANNER_WIDTH: u16 = 69;

pub fn apexrouter_banner(f: &mut Frame, area: Rect, t: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    let fits_box = area.width >= BANNER_WIDTH && area.height >= BANNER_ROWS;
    if fits_box {
        let dim = Style::default().fg(t.text_dim);
        let accent = Style::default().fg(t.orange).add_modifier(Modifier::BOLD);

        lines.push(Line::from(vec![Span::styled(BANNER_ART[0], dim)]));
        lines.push(Line::from(vec![Span::styled(BANNER_ART[1], dim)]));
        lines.push(Line::from(vec![
            Span::styled("┃                          ", dim),
            Span::styled("🚀", accent),
            Span::styled(" APEXROUTER CLI ", accent),
            Span::styled("🚀", accent),
            Span::styled("                          ┃", dim),
        ]));
        lines.push(Line::from(vec![
            Span::styled("┃          ", dim),
            Span::styled("Orchestrating Models, Agents & Developer Tools", Style::default().fg(t.text_dim)),
            Span::styled("          ┃", dim),
        ]));
        lines.push(Line::from(vec![Span::styled(BANNER_ART[4], dim)]));
        lines.push(Line::from(vec![Span::styled(BANNER_ART[5], dim)]));
    } else {
        lines.push(Line::from(Span::styled(
            "APEXROUTER CLI",
            Style::default().fg(t.orange).add_modifier(Modifier::BOLD),
        )));
    }

    let content_rows = lines.len() as u16;
    let top_pad = area.height.saturating_sub(content_rows) / 2;
    let mut padded: Vec<Line> = Vec::with_capacity(lines.len() + top_pad as usize);
    for _ in 0..top_pad {
        padded.push(Line::from(""));
    }
    padded.extend(lines);

    let para = Paragraph::new(Text::from(padded)).alignment(Alignment::Center);
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::tui::theme::Theme;

    fn render(w: u16, h: u16) -> String {
        let t = Theme::hearth();
        let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("test terminal");
        terminal
            .draw(|f| apexrouter_banner(f, f.area(), &t))
            .expect("render banner");
        let buf = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn banner_renders_the_box_when_it_fits() {
        let out = render(80, 10);
        assert!(out.contains("APEXROUTER CLI"), "box wordmark missing:\n{out}");
        assert!(
            !out.contains("type / for commands"),
            "hint leaked into banner:\n{out}"
        );
    }

    #[test]
    fn banner_degrades_on_a_narrow_area() {
        let out = render(30, 6);
        assert!(out.contains("APEXROUTER CLI"), "degraded wordmark missing:\n{out}");
        assert!(
            !out.contains("━━━━━━━"),
            "box art rendered in too-narrow area:\n{out}"
        );
    }

    #[test]
    fn banner_rows_have_uniform_width() {
        for (i, row) in BANNER_ART.iter().enumerate() {
            assert_eq!(
                row.chars().count() as u16,
                BANNER_WIDTH,
                "banner row {i} has width {} expected {BANNER_WIDTH}",
                row.chars().count()
            );
        }
    }

    #[test]
    fn banner_does_not_panic_on_a_tiny_area() {
        let _ = render(1, 1);
        let _ = render(10, 2);
    }

    #[test]
    fn banner_renders_with_the_no_color_theme() {
        let t = Theme::no_color();
        let mut terminal = Terminal::new(TestBackend::new(80, 10)).expect("test terminal");
        terminal
            .draw(|f| apexrouter_banner(f, f.area(), &t))
            .expect("render banner uncolored");
        let buf = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..10 {
            for x in 0..80 {
                out.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(out.contains("APEXROUTER CLI"));
    }
}
