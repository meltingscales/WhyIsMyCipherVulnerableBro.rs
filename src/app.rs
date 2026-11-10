//! Application state: the four demo modes and transitions between them.

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

    /// Placeholder copy describing the mode's planned purpose.
    pub fn description(self) -> &'static str {
        match self {
            Mode::Autocrack => "Pick a victim cipher and watch the attack run with cool visuals.",
            Mode::TrafficSimulator => "Pretend to intercept fake web traffic, then decrypt it.",
            Mode::Lessons => {
                "Cryptopals-style, step-by-step walk-throughs of why each cipher dies."
            }
            Mode::MathViews => "Graphs and matrices directly tied to the underlying math.",
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

/// Mutable shell state shared by all modes.
pub struct App {
    pub mode: Mode,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Autocrack,
            should_quit: false,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
