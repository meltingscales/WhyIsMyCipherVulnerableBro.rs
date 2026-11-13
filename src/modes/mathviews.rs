//! Math Views mode: the statistics behind the breaks, drawn as data. A
//! letter-frequency chart of the selected sample with the chi-squared and
//! index-of-coincidence readouts that justify the attacks.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Paragraph};

use crate::cipher::sweet32::{DEMO_BLOCK_BITS as BIRTHDAY_BITS, Simulator, expected_collisions};

use crate::cipher::caesar;
use crate::cipher::freq::{ENGLISH_SAMPLE, chi_squared, index_of_coincidence, letter_counts};
use crate::cipher::vigenere;

/// One chartable sample. Ciphertexts derive from the shared English sample
/// so the demo tells one coherent story.
struct Sample {
    name: &'static str,
    text: String,
}

fn samples() -> Vec<Sample> {
    vec![
        Sample {
            name: "English plaintext",
            text: ENGLISH_SAMPLE.to_string(),
        },
        Sample {
            name: "Caesar ciphertext (shift 11)",
            text: caesar::encrypt(ENGLISH_SAMPLE, 11),
        },
        Sample {
            name: "Vigenère ciphertext (key \"lemon\")",
            text: vigenere::encrypt(ENGLISH_SAMPLE, "lemon"),
        },
    ]
}

/// Which chart the screen shows; `v` cycles them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Frequency,
    Birthday,
    Cost,
}

impl View {
    fn next(self) -> Self {
        match self {
            View::Frequency => View::Birthday,
            View::Birthday => View::Cost,
            View::Cost => View::Frequency,
        }
    }
}

/// What a click on the last rendered frame meant, per recorded rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    /// The left half of the chart area: click cycles to the previous sample.
    ChartPrev,
    /// The right half: next sample.
    ChartNext,
}

/// Math Views screen state.
#[derive(Debug)]
pub struct State {
    /// Index into the sample set; arrows cycle with wrap-around.
    pub sample: usize,
    /// The active chart.
    pub view: View,
    /// Clickable rects recorded by the last render (immediate-mode hits).
    pub(crate) hit: Vec<(Rect, Hit)>,
    /// Test seam: the last key routed here (only the active mode receives keys).
    pub(crate) last_key: Option<KeyEvent>,
    /// Test seam: ticks received while this mode is active.
    pub(crate) tick_count: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            sample: 0,
            view: View::Frequency,
            hit: Vec::new(),
            last_key: None,
            tick_count: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn handle_key(&mut self, key: KeyEvent) {
        self.last_key = Some(key);
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Left => {
                let len = samples().len();
                self.sample = (self.sample + len - 1) % len;
            }
            KeyCode::Right => {
                self.sample = (self.sample + 1) % samples().len();
            }
            KeyCode::Char('v') => {
                self.view = self.view.next();
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn key_hints(&self) -> &'static str {
        "Left/Right sample · v view"
    }

    /// Handle a routed left-click: the left half of the chart area cycles
    /// to the previous sample, the right half to the next, in every view.
    pub fn click(&mut self, pos: ratatui::layout::Position) -> &'static str {
        if let Some(&(_, hit)) = self.hit.iter().find(|(rect, _)| rect.contains(pos)) {
            let len = samples().len();
            self.sample = match hit {
                Hit::ChartPrev => (self.sample + len - 1) % len,
                Hit::ChartNext => (self.sample + 1) % len,
            };
            "mathviews:sample"
        } else {
            "mathviews:inert"
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.view {
            View::Frequency => self.render_frequency(frame, area),
            View::Birthday => self.render_birthday(frame, area),
            View::Cost => self.render_cost(frame, area),
        }
    }

    /// Record the chart area's halves as this view's clickable rects.
    fn record_chart_halves(&mut self, chart_area: Rect) {
        self.hit.clear();
        let half = chart_area.width / 2;
        self.hit.push((
            Rect {
                x: chart_area.x,
                y: chart_area.y,
                width: half,
                height: chart_area.height,
            },
            Hit::ChartPrev,
        ));
        self.hit.push((
            Rect {
                x: chart_area.x + half,
                y: chart_area.y,
                width: chart_area.width - half,
                height: chart_area.height,
            },
            Hit::ChartNext,
        ));
    }

    fn render_frequency(&mut self, frame: &mut Frame, area: Rect) {
        let samples = samples();
        let sample = &samples[self.sample % samples.len()];
        let [chart_area, stats_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);
        self.record_chart_halves(chart_area);

        let counts = letter_counts(&sample.text);
        let total: usize = counts.iter().sum();
        let bars: Vec<Bar> = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                Bar::default()
                    .label(((b'a' + i as u8) as char).to_string())
                    .value(((count as f64 / total as f64) * 1000.0).round() as u64)
            })
            .collect();
        let chart = BarChart::new(bars)
            .block(Block::bordered().title(format!(" Letter frequency — {} ", sample.name)))
            .bar_width(3)
            .bar_style(Style::default().fg(ratatui::style::Color::Cyan));
        frame.render_widget(chart, chart_area);

        // Bar values are per-mille of total letters; the stats are the real
        // numbers the attacks score with.
        let chi = chi_squared(&sample.text);
        let ioc = index_of_coincidence(&sample.text);
        let stats = Paragraph::new(Line::from(format!(
            "chi-squared vs English: {chi:.4}   ·   index of coincidence: {ioc:.4}   (English ≈ 0.066, flat ≈ 0.038)"
        )))
        .block(Block::bordered());
        frame.render_widget(stats, stats_area);
    }

    /// Observed vs expected collisions at 1k-block checkpoints of the demo
    /// space, computed once and shared by every render.
    fn birthday_checkpoints() -> &'static [(usize, usize, f64)] {
        static CHECKPOINTS: std::sync::OnceLock<Vec<(usize, usize, f64)>> =
            std::sync::OnceLock::new();
        CHECKPOINTS.get_or_init(|| {
            let mut sim = Simulator::demo();
            (1..=8)
                .map(|k| {
                    let blocks = k * 1024;
                    sim.step_past_exploit(blocks - sim.records_sent);
                    let expected = expected_collisions(blocks, BIRTHDAY_BITS);
                    (blocks, sim.collisions, expected)
                })
                .collect()
        })
    }

    fn render_birthday(&mut self, frame: &mut Frame, area: Rect) {
        let [chart_area, stats_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).areas(area);
        self.record_chart_halves(chart_area);

        let observed: Vec<Bar> = Self::birthday_checkpoints()
            .iter()
            .map(|&(blocks, collisions, _)| {
                Bar::default()
                    .label(format!("{}k", blocks / 1024))
                    .value(collisions as u64)
                    .style(Style::default().fg(ratatui::style::Color::Cyan))
            })
            .collect();
        let expected: Vec<Bar> = Self::birthday_checkpoints()
            .iter()
            .map(|&(blocks, _, expected)| {
                Bar::default()
                    .label(format!("{}k", blocks / 1024))
                    .value(expected.ceil() as u64)
                    .style(Style::default().fg(ratatui::style::Color::Magenta))
            })
            .collect();
        let chart = BarChart::new(observed)
            .data(BarGroup::new(expected))
            .block(Block::bordered().title(" Birthday collisions — 2^20 block space "))
            .bar_width(3)
            .bar_gap(1);
        frame.render_widget(chart, chart_area);

        let stats = Paragraph::new(Text::from(vec![
            Line::from("observed (cyan) vs expected k²/2^21 (magenta)"),
            Line::from("checkpoints every 1k blocks, 2^20 space"),
        ]))
        .block(Block::bordered());
        frame.render_widget(stats, stats_area);
    }

    /// Observed vs expected solver costs at several problem sizes, computed
    /// once by actually running the solvers. Log-scaled so four decades of
    /// step counts share one chart.
    fn cost_series() -> &'static [(&'static str, u64, f64)] {
        static SERIES: std::sync::OnceLock<Vec<(&'static str, u64, f64)>> =
            std::sync::OnceLock::new();
        SERIES.get_or_init(|| {
            let mut rho = crate::cipher::rsa::RhoSolver::new(crate::cipher::rsa::N);
            while rho.factor.is_none() {
                rho.step(8192);
            }
            let mut rho2 = crate::cipher::rsa::RhoSolver::new(crate::cipher::rsa::N2);
            while rho2.factor.is_none() {
                rho2.step(8192);
            }
            let run_bsgs = |p: u64, g: u64, target: u64| {
                let mut solver = crate::cipher::dlog::BsgsSolver::new(p, g, target);
                while solver.exponent.is_none() {
                    solver.step(131_072);
                }
                solver.steps
            };
            let steps = run_bsgs(
                crate::cipher::dlog::P,
                crate::cipher::dlog::G,
                crate::cipher::dlog::server_public(),
            );
            let steps2 = run_bsgs(
                crate::cipher::dlog::P2,
                crate::cipher::dlog::G2,
                crate::cipher::dlog::second_target(),
            );
            vec![
                (
                    "rho62",
                    rho.iterations,
                    crate::cipher::rsa::expected_iterations(),
                ),
                (
                    "rho63",
                    rho2.iterations,
                    (crate::cipher::rsa::N2 as f64).powf(0.25),
                ),
                ("bs40", steps, crate::cipher::dlog::expected_steps()),
                (
                    "bs38",
                    steps2,
                    ((crate::cipher::dlog::P2 - 1) as f64).sqrt(),
                ),
            ]
        })
    }

    fn render_cost(&mut self, frame: &mut Frame, area: Rect) {
        let [chart_area, stats_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(4)]).areas(area);
        self.record_chart_halves(chart_area);

        let log10_bar = |steps: f64| (steps.log10() * 10_000.0).round() as u64;
        let observed: Vec<Bar> = Self::cost_series()
            .iter()
            .map(|&(label, steps, _)| {
                Bar::default()
                    .label(label)
                    .value(log10_bar(steps as f64))
                    .style(Style::default().fg(ratatui::style::Color::Cyan))
            })
            .collect();
        let expected: Vec<Bar> = Self::cost_series()
            .iter()
            .map(|&(label, _, expected)| {
                Bar::default()
                    .label(label)
                    .value(log10_bar(expected))
                    .style(Style::default().fg(ratatui::style::Color::Magenta))
            })
            .collect();
        let chart = BarChart::new(observed)
            .data(BarGroup::new(expected))
            .block(Block::bordered().title(" Attack cost — solver steps "))
            .bar_width(5)
            .bar_gap(1);
        frame.render_widget(chart, chart_area);

        let stats = Paragraph::new(Text::from(vec![
            Line::from("log10 steps: observed (cyan) vs expected (magenta)"),
            Line::from("rho ≈ N^¼, bsgs ≈ √p; brute force would be N — far off-chart"),
        ]))
        .block(Block::bordered());
        frame.render_widget(stats, stats_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cipher::freq::chi_squared as chi_of;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    fn rows_rendered(state: &mut State) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| state.render(f, f.area())).unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn arrows_cycle_samples_with_wraparound_both_ways() {
        let mut state = State::new();
        assert_eq!(state.sample, 0);
        state.handle_key(press(KeyCode::Left)); // wraps to last
        assert_eq!(state.sample, 2);
        state.handle_key(press(KeyCode::Right)); // wraps back to first
        assert_eq!(state.sample, 0);
        state.handle_key(press(KeyCode::Right));
        assert_eq!(state.sample, 1);
    }

    #[test]
    fn plaintext_has_the_smallest_chi_squared_of_the_set() {
        let samples = samples();
        let scores: Vec<f64> = samples.iter().map(|s| chi_of(&s.text)).collect();
        assert!(scores[0] < scores[1], "plaintext beats Caesar ct");
        assert!(scores[0] < scores[2], "plaintext beats Vigenère ct");
        // And Vigenère's polyalphabetic spread lifts its IoC away from English.
        let iocs: Vec<f64> = samples
            .iter()
            .map(|s| index_of_coincidence(&s.text))
            .collect();
        assert!(iocs[0] > iocs[2], "English IoC above Vigenère ct");
    }

    #[test]
    fn chart_renders_sample_name_letters_and_statistics() {
        let mut state = State::new();
        let text = rows_rendered(&mut state).join("\n");
        assert!(
            text.contains("Letter frequency — English plaintext"),
            "sample named"
        );
        assert!(text.contains("a"), "letter axis renders");
        assert!(
            text.contains("chi-squared vs English:"),
            "chi readout renders"
        );
        assert!(
            text.contains("index of coincidence:"),
            "IoC readout renders"
        );
    }

    #[test]
    fn chart_halves_cycle_samples_in_every_view() {
        for view in [View::Frequency, View::Birthday, View::Cost] {
            let mut state = State::new();
            state.view = view;
            let _ = rows_rendered(&mut state);
            let (left, _) = state.hit[0];
            let (right, _) = state.hit[1];
            state.click(ratatui::layout::Position {
                x: right.x + 1,
                y: right.y + 1,
            });
            assert_eq!(state.sample, 1, "{view:?}: right half advances");
            state.click(ratatui::layout::Position {
                x: left.x + 1,
                y: left.y + 1,
            });
            assert_eq!(state.sample, 0, "{view:?}: left half goes back");
        }
    }

    #[test]
    fn v_cycles_frequency_birthday_cost_and_back() {
        let mut state = State::new();
        assert_eq!(state.view, View::Frequency);
        state.handle_key(press(KeyCode::Char('v')));
        assert_eq!(state.view, View::Birthday);
        state.handle_key(press(KeyCode::Char('v')));
        assert_eq!(state.view, View::Cost);
        // Arrows keep cycling samples in any view (harmless in the charts).
        state.handle_key(press(KeyCode::Right));
        assert_eq!(state.sample, 1);
        state.handle_key(press(KeyCode::Char('v')));
        assert_eq!(state.view, View::Frequency);
    }

    #[test]
    fn cost_view_renders_both_labeled_series() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Char('v')));
        state.handle_key(press(KeyCode::Char('v')));
        assert_eq!(state.view, View::Cost);
        let text = rows_rendered(&mut state).join("\n");
        assert!(text.contains("Attack cost — solver steps"), "view named");
        assert!(
            text.contains("rho62") && text.contains("rho63"),
            "rho series labeled"
        );
        assert!(
            text.contains("bs40") && text.contains("bs38"),
            "bsgs series labeled"
        );
        assert!(
            text.contains("observed (cyan) vs expected (magenta)"),
            "series labeled"
        );
    }

    #[test]
    fn birthday_view_renders_both_labeled_series() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Char('v')));
        let text = rows_rendered(&mut state).join("\n");
        assert!(
            text.contains("Birthday collisions — 2^20 block space"),
            "view named"
        );
        assert!(text.contains("observed (cyan)"), "observed labeled");
        assert!(
            text.contains("expected k²/2^21 (magenta)"),
            "expected labeled"
        );
        assert!(
            text.contains("checkpoints every 1k blocks"),
            "checkpoints named"
        );
    }

    #[test]
    fn switching_samples_updates_name_and_readouts() {
        let mut state = State::new();
        let plaintext_chi_row = rows_rendered(&mut state)
            .into_iter()
            .find(|r| r.contains("chi-squared"))
            .expect("stats row");
        state.handle_key(press(KeyCode::Right));
        let caesar_text = rows_rendered(&mut state).join("\n");
        assert!(caesar_text.contains("Caesar ciphertext"), "name updates");
        let caesar_chi_row = caesar_text
            .lines()
            .map(|l| l.to_string())
            .find(|r| r.contains("chi-squared"))
            .expect("stats row");
        assert_ne!(
            plaintext_chi_row.trim(),
            caesar_chi_row.trim(),
            "values update"
        );
    }
}
