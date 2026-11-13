//! Shell chrome: tab bar and footer. Mode bodies render themselves via the
//! per-mode dispatch in [`render_body`].

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Tabs};

use crate::app::{ALL_MODES, App, Mode};

/// Global keybindings shown in the footer on every screen.
pub const FOOTER_TEXT: &str = "Tab/1-4 switch · q quits · mouse works";

pub fn render(frame: &mut Frame, app: &mut App) {
    let [tabs_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    render_tabs(frame, tabs_area, app);
    render_body(frame, body_area, app);
    render_footer(frame, footer_area, app);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &mut App) {
    let titles: Vec<Line> = ALL_MODES
        .iter()
        .enumerate()
        .map(|(i, mode)| Line::from(format!("{}: {}", i + 1, mode.title())))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::bordered().title(" WhyIsMyCipherVulnerableBro.rs "))
        .select(app.mode.index())
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD));
    frame.render_widget(tabs, area);

    // Record each tab's clickable rect from the buffer that was actually
    // drawn — the Tabs widget's padding and divider widths are its business,
    // not ours to re-derive. Titles are unique ASCII, so a row scan finds
    // each one exactly where it rendered.
    app.tab_hits.clear();
    let inner = {
        let mut inner = area;
        inner.x += 1;
        inner.y += 1;
        inner.width = inner.width.saturating_sub(2);
        inner.height = inner.height.saturating_sub(2);
        inner
    };
    let buffer = frame.buffer_mut();
    let row: String = (inner.x..inner.x + inner.width)
        .map(|x| buffer[(x, inner.y)].symbol().chars().next().unwrap_or(' '))
        .collect();
    for (i, mode) in ALL_MODES.iter().enumerate() {
        let title = format!("{}: {}", i + 1, mode.title());
        if let Some(byte_index) = row.find(&title) {
            let char_index = row[..byte_index].chars().count() as u16;
            let rect = Rect {
                x: inner.x + char_index,
                y: inner.y,
                width: title.chars().count() as u16,
                height: 1,
            };
            app.tab_hits.push((rect, i));
        }
    }
}

/// Dispatch the body to the active mode's renderer.
fn render_body(frame: &mut Frame, area: Rect, app: &mut App) {
    match app.mode {
        Mode::Autocrack => app.autocrack.render(frame, area),
        Mode::TrafficSimulator => app.traffic.render(frame, area),
        Mode::Lessons => app.lessons.render(frame, area),
        Mode::MathViews => app.mathviews.render(frame, area),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mode_hints = match app.mode {
        Mode::Autocrack => app.autocrack.key_hints(),
        Mode::TrafficSimulator => app.traffic.key_hints(),
        Mode::Lessons => app.lessons.key_hints(),
        Mode::MathViews => app.mathviews.key_hints(),
    };
    let footer = Paragraph::new(Line::from(format!("{FOOTER_TEXT} | {mode_hints}")).centered())
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Render `app` into a test terminal and return the buffer.
    fn render_app(app: &mut App) -> ratatui::buffer::Buffer {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn rows(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn tab_bar_lists_all_modes_with_active_one_highlighted() {
        let mut app = App::new();
        app.select(2); // Lessons
        let buffer = render_app(&mut app);
        let tab_row = rows(&buffer)
            .into_iter()
            .find(|row| row.contains("Autocrack") && row.contains("Math Views"))
            .expect("tab bar row lists every mode");
        assert!(tab_row.contains("Lessons"), "every tab title present");

        // The active tab's title cells carry the highlight modifier; the
        // inactive tabs' cells do not.
        let y = buffer
            .content
            .chunks(buffer.area.width as usize)
            .position(|row| {
                row.iter()
                    .map(|c| c.symbol())
                    .collect::<String>()
                    .contains("Lessons")
            })
            .expect("lessons row") as u16;
        let row_text: String = (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol().to_string())
            .collect();
        let active_x = row_text.find("Lessons").expect("active title") as u16;
        let inactive_x = row_text.find("Autocrack").expect("inactive title") as u16;

        assert!(buffer[(active_x, y)].modifier.contains(Modifier::REVERSED));
        assert!(
            !buffer[(inactive_x, y)]
                .modifier
                .contains(Modifier::REVERSED)
        );
    }

    #[test]
    fn footer_shows_global_bindings_on_every_mode() {
        for mode in ALL_MODES {
            let mut app = App::new();
            app.select(mode.index());
            let buffer = render_app(&mut app);
            let text = rows(&buffer).join("\n");
            assert!(
                text.contains(FOOTER_TEXT),
                "global footer on {}",
                mode.title()
            );
        }
    }

    #[test]
    fn footer_shows_the_active_modes_hints_only() {
        let mut app = App::new();
        app.select(Mode::TrafficSimulator.index());
        let text = rows(&render_app(&mut app)).join("\n");
        assert!(text.contains("Up/Dn session"), "active hints shown");
        assert!(!text.contains("victim"), "other modes' hints hidden");

        app.select(Mode::MathViews.index());
        let text = rows(&render_app(&mut app)).join("\n");
        assert!(text.contains("Left/Right sample"), "active hints shown");
        assert!(!text.contains("lesson"), "other modes' hints hidden");
    }

    #[test]
    fn body_dispatches_to_the_active_modes_renderer() {
        let mut app = App::new();
        app.select(Mode::MathViews.index());
        let text = rows(&render_app(&mut app)).join("\n");
        assert!(text.contains("Math Views"), "active mode's body renders");
    }
}
