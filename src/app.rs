//! Application state: the four demo modes, per-mode screens, and the
//! dispatch that routes keys and ticks to the active mode only.

use ratatui::crossterm::event::KeyEvent;

use crate::modes;

/// The four demo modes; order defines tab order and the `1`-`4` selection keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Autocrack,
    TrafficSimulator,
    Lessons,
    MathViews,
}

/// All modes in tab order.
pub const ALL_MODES: [Mode; 4] = [
    Mode::Autocrack,
    Mode::TrafficSimulator,
    Mode::Lessons,
    Mode::MathViews,
];

impl Mode {
    /// Human-readable tab title.
    pub fn title(self) -> &'static str {
        match self {
            Mode::Autocrack => "Autocrack",
            Mode::TrafficSimulator => "Traffic Simulator",
            Mode::Lessons => "Lessons",
            Mode::MathViews => "Math Views",
        }
    }

    /// Position in [`ALL_MODES`]; also the mode's selection key minus one.
    pub fn index(self) -> usize {
        ALL_MODES
            .iter()
            .position(|&mode| mode == self)
            .expect("every mode is in ALL_MODES")
    }

    /// Mode at a position, wrapping past the ends so every input lands somewhere.
    fn at(index: usize) -> Self {
        ALL_MODES[index % ALL_MODES.len()]
    }
}

/// Mutable shell state: the active mode, global flags, and every mode's own
/// screen state (inactive modes keep their state untouched).
pub struct App {
    pub mode: Mode,
    pub should_quit: bool,
    pub autocrack: modes::autocrack::State,
    pub traffic: modes::traffic::State,
    pub lessons: modes::lessons::State,
    pub mathviews: modes::mathviews::State,
    /// Clickable tab-bar rects, recorded by the last `ui::render` frame.
    pub(crate) tab_hits: Vec<(ratatui::layout::Rect, usize)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Autocrack,
            should_quit: false,
            autocrack: modes::autocrack::State::new(),
            traffic: modes::traffic::State::new(),
            lessons: modes::lessons::State::new(),
            mathviews: modes::mathviews::State::new(),
            tab_hits: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Switch to the next mode, wrapping from the last to the first.
    pub fn next(&mut self) {
        self.mode = Mode::at(self.mode.index() + 1);
    }

    /// Switch to the previous mode, wrapping from the first to the last.
    pub fn prev(&mut self) {
        self.mode = Mode::at(self.mode.index() + ALL_MODES.len() - 1);
    }

    /// Switch to the mode at `index` (0-based), wrapping out-of-range input.
    pub fn select(&mut self, index: usize) {
        self.mode = Mode::at(index);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Route a non-global key press to the active mode's handler; inactive
    /// modes never see keys.
    pub fn mode_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Autocrack => self.autocrack.handle_key(key),
            Mode::TrafficSimulator => self.traffic.handle_key(key),
            Mode::Lessons => self.lessons.handle_key(key),
            Mode::MathViews => self.mathviews.handle_key(key),
        }
    }

    /// Drive the active mode's animation for one tick; inactive modes freeze.
    pub fn tick(&mut self) {
        match self.mode {
            Mode::Autocrack => self.autocrack.tick(),
            Mode::TrafficSimulator => self.traffic.tick(),
            Mode::Lessons => self.lessons.tick(),
            Mode::MathViews => self.mathviews.tick(),
        }
    }

    /// Route a left-click: the tab bar is shell chrome and claimed first;
    /// everything else reaches only the active mode's click handler.
    /// Returns where the click went, like `send_key` reports its routing.
    pub fn mouse_click(&mut self, pos: ratatui::layout::Position) -> &'static str {
        if let Some(&(_, index)) = self.tab_hits.iter().find(|(rect, _)| rect.contains(pos)) {
            self.mode = Mode::at(index);
            return "shell:tab";
        }
        match self.mode {
            Mode::Autocrack => self.autocrack.click(pos),
            Mode::TrafficSimulator => self.traffic.click(pos),
            Mode::Lessons => self.lessons.click(pos),
            Mode::MathViews => self.mathviews.click(pos),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn next_cycles_forward() {
        let mut app = App::new();
        assert_eq!(app.mode, Mode::Autocrack);
        app.next();
        assert_eq!(app.mode, Mode::TrafficSimulator);
        app.next();
        assert_eq!(app.mode, Mode::Lessons);
        app.next();
        assert_eq!(app.mode, Mode::MathViews);
    }

    #[test]
    fn next_wraps_from_last_to_first() {
        let mut app = App::new();
        app.select(ALL_MODES.len() - 1);
        app.next();
        assert_eq!(app.mode, Mode::Autocrack);
    }

    #[test]
    fn prev_cycles_backward() {
        let mut app = App::new();
        app.select(1);
        app.prev();
        assert_eq!(app.mode, Mode::Autocrack);
    }

    #[test]
    fn prev_wraps_from_first_to_last() {
        let mut app = App::new();
        assert_eq!(app.mode, Mode::Autocrack);
        app.prev();
        assert_eq!(app.mode, Mode::MathViews);
    }

    #[test]
    fn select_picks_mode_directly() {
        let mut app = App::new();
        app.select(2);
        assert_eq!(app.mode, Mode::Lessons);
        app.select(0);
        assert_eq!(app.mode, Mode::Autocrack);
    }

    #[test]
    fn quit_sets_flag() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    /// Render the shell once (records tab rects), then click a tab.
    fn click_tab(mut app: App, index: usize) -> Mode {
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| crate::ui::render(frame, &mut app))
            .unwrap();
        let (rect, _) = app.tab_hits[index];
        app.mouse_click(ratatui::layout::Position {
            x: rect.x + 1,
            y: rect.y,
        });
        app.mode
    }

    #[test]
    fn tab_clicks_switch_modes_and_body_clicks_stay_in_mode() {
        assert_eq!(click_tab(App::new(), 2), Mode::Lessons);
        assert_eq!(click_tab(App::new(), 3), Mode::MathViews);

        // A body click reaches only the active mode (autocrack starts).
        let mut app = App::new();
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| crate::ui::render(frame, &mut app))
            .unwrap();
        let routed = app.mouse_click(ratatui::layout::Position { x: 10, y: 10 });
        assert_eq!(routed, "autocrack:victim");
        assert!(matches!(
            app.autocrack.phase,
            crate::modes::autocrack::Phase::Running(_)
        ));
    }

    #[test]
    fn mode_keys_reach_only_the_active_mode() {
        let mut app = App::new(); // Autocrack active
        let enter = key(KeyCode::Enter);
        app.mode_key(enter);
        assert_eq!(app.autocrack.last_key, Some(enter));
        assert_eq!(app.traffic.last_key, None);
        assert_eq!(app.lessons.last_key, None);
        assert_eq!(app.mathviews.last_key, None);

        app.select(Mode::Lessons.index());
        let esc = key(KeyCode::Esc);
        app.mode_key(esc);
        assert_eq!(app.lessons.last_key, Some(esc));
        assert_eq!(app.autocrack.last_key, Some(enter), "earlier key untouched");
    }

    #[test]
    fn ticks_reach_only_the_active_mode() {
        let mut app = App::new(); // Autocrack active
        app.tick();
        app.tick();
        assert_eq!(app.autocrack.tick_count, 2);
        assert_eq!(app.traffic.tick_count, 0);
        assert_eq!(app.lessons.tick_count, 0);
        assert_eq!(app.mathviews.tick_count, 0);

        app.select(Mode::MathViews.index());
        app.tick();
        assert_eq!(app.mathviews.tick_count, 1);
        assert_eq!(app.autocrack.tick_count, 2, "frozen while inactive");
    }
}
