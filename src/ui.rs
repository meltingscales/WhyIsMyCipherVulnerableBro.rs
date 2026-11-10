//! Shell chrome: tab bar and footer. Mode bodies render themselves via the
//! per-mode dispatch in [`render_body`].

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Tabs};

use crate::app::{ALL_MODES, App, Mode};

/// Global keybindings shown in the footer on every screen.
pub const FOOTER_TEXT: &str = "Tab/Shift+Tab switch · 1-4 select mode · q or Ctrl-C quits";

pub fn render(frame: &mut Frame, app: &App) {
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

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
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
}

/// Dispatch the body to the active mode's renderer.
fn render_body(frame: &mut Frame, area: Rect, app: &App) {
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
    fn render_app(app: &App) -> ratatui::buffer::Buffer {
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
        let buffer = render_app(&app);
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
            let buffer = render_app(&app);
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
        let text = rows(&render_app(&app)).join("\n");
        assert!(text.contains("Up/Dn session"), "active hints shown");
        assert!(!text.contains("victim"), "other modes' hints hidden");

        app.select(Mode::MathViews.index());
        let text = rows(&render_app(&app)).join("\n");
        assert!(text.contains("Left/Right sample"), "active hints shown");
        assert!(!text.contains("lesson"), "other modes' hints hidden");
    }

    #[test]
    fn body_dispatches_to_the_active_modes_renderer() {
        let mut app = App::new();
        app.select(Mode::MathViews.index());
        let text = rows(&render_app(&app)).join("\n");
        assert!(text.contains("Math Views"), "active mode's body renders");
    }
}
