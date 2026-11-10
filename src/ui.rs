//! Rendering: tab bar, mode placeholder body, footer with keybindings.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Tabs};

use crate::app::{ALL_MODES, App, Mode};

/// Footer keybinding help, asserted by tests and shown on every screen.
pub const FOOTER_TEXT: &str = " Tab/Shift+Tab switch | 1-4 select mode | q or Ctrl-C quits ";

pub fn render(frame: &mut Frame, app: &App) {
    let [tabs_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    render_tabs(frame, tabs_area, app);
    render_body(frame, body_area, app);
    render_footer(frame, footer_area);
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

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let mode: Mode = app.mode;
    let body = Text::from(vec![
        Line::from(mode.title()),
        Line::from(""),
        Line::from(mode.description()),
        Line::from(""),
        Line::from("[ placeholder — this mode is not implemented yet ]"),
    ]);
    let paragraph = Paragraph::new(body)
        .block(Block::bordered().title(format!(" {} ", mode.title())))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(FOOTER_TEXT).centered());
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Render `app` into a test terminal and return the buffer contents as
    /// one row-per-line string.
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
    fn every_mode_renders_its_name_and_placeholder() {
        for mode in ALL_MODES {
            let mut app = App::new();
            app.select(mode.index());
            let buffer = render_app(&app);
            let text = rows(&buffer).join("\n");
            assert!(
                text.contains(mode.title()),
                "{} title renders",
                mode.title()
            );
            assert!(
                text.contains(mode.description()),
                "{} description renders",
                mode.title()
            );
            assert!(
                text.contains("[ placeholder — this mode is not implemented yet ]"),
                "{} placeholder renders",
                mode.title()
            );
        }
    }

    #[test]
    fn footer_lists_essential_bindings_on_every_mode() {
        for mode in ALL_MODES {
            let mut app = App::new();
            app.select(mode.index());
            let buffer = render_app(&app);
            let text = rows(&buffer).join("\n");
            assert!(
                text.contains(FOOTER_TEXT),
                "footer visible on {}",
                mode.title()
            );
            assert!(text.contains("Tab"), "footer mentions tab switching");
            assert!(text.contains("q"), "footer mentions quit key");
        }
    }
}
