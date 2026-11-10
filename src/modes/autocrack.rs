//! Autocrack mode: pick a victim cipher, watch the attack run live.
//!
//! Caesar streams all 26 shift candidates, scored by chi-squared against
//! English. Vigenère first streams the index-of-coincidence key-length scan,
//! then solves each key column by per-column chi-squared, assembling the key
//! column by column. One row lands per tick; `advance` is pure, so the runner
//! is fully testable without a terminal.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use crate::cipher::caesar;
use crate::cipher::freq::{ENGLISH_SAMPLE, chi_squared};
use crate::cipher::vigenere;

/// The victim ciphers on offer, in list order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Victim {
    Caesar,
    Vigenere,
}

pub const VICTIMS: [Victim; 2] = [Victim::Caesar, Victim::Vigenere];

impl Victim {
    pub fn name(self) -> &'static str {
        match self {
            Victim::Caesar => "Caesar (shift 11)",
            Victim::Vigenere => "Vigenère (key \"lemon\")",
        }
    }

    /// One-line explanation shown under the victim's name.
    pub fn blurb(self) -> &'static str {
        match self {
            Victim::Caesar => "monoalphabetic — dies to frequency analysis",
            Victim::Vigenere => "polyalphabetic — dies to IoC + per-column analysis",
        }
    }

    /// The sample ciphertext this victim carries, derived from the shared
    /// English sample so every mode tells the same story.
    pub fn ciphertext(self) -> String {
        match self {
            Victim::Caesar => caesar::encrypt(ENGLISH_SAMPLE, 11),
            Victim::Vigenere => vigenere::encrypt(ENGLISH_SAMPLE, "lemon"),
        }
    }
}

/// Which screen the mode is on.
#[derive(Debug, PartialEq)]
pub enum Phase {
    /// Choosing a victim; the index selects into [`VICTIMS`].
    Select { selection: usize },
    /// An attack is running or finished (`run.done` distinguishes them).
    Running(Run),
}

/// One visible step of an attack: what was tried, how English it looked.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub label: String,
    pub score: f64,
    pub note: String,
}

/// A live attack. `advance` pushes the next row; when the plan is exhausted
/// the run is `done` and carries the recovered key and plaintext.
#[derive(Debug, PartialEq)]
pub struct Run {
    pub victim: Victim,
    pub ciphertext: String,
    pub rows: Vec<Row>,
    /// Index into `rows` of the best-scoring candidate so far (min score).
    pub best_row: Option<usize>,
    /// Steps already taken; the next step reads its inputs from this.
    step: usize,
    /// Vigenère only: the precomputed key-length scan, streamed one row per tick.
    scan: Vec<vigenere::KeyLengthScore>,
    pub done: bool,
    pub recovered_key: String,
    pub recovered_plaintext: String,
}

/// Key lengths the Vigenère scan streams, 1..=12 like the core tests.
const VIGENERE_MAX_LEN: usize = 12;

impl Run {
    pub fn new(victim: Victim) -> Self {
        let ciphertext = victim.ciphertext();
        let scan = match victim {
            Victim::Vigenere => vigenere::key_length_scan(&ciphertext, 1, VIGENERE_MAX_LEN),
            Victim::Caesar => Vec::new(),
        };
        Self {
            victim,
            ciphertext,
            rows: Vec::new(),
            best_row: None,
            step: 0,
            scan,
            done: false,
            recovered_key: String::new(),
            recovered_plaintext: String::new(),
        }
    }

    /// Total steps this run will take (rows it will stream).
    pub fn total_steps(&self) -> usize {
        match self.victim {
            Victim::Caesar => 26,
            // 12 scan rows, then one solve row per key column of the chosen length.
            Victim::Vigenere => VIGENERE_MAX_LEN + VIGENERE_MAX_LEN,
        }
    }

    /// Advance the attack by one visible step. Pure with respect to wall
    /// clock: callers drive it from ticks, tests drive it directly.
    pub fn advance(&mut self) {
        if self.done {
            return;
        }
        match self.victim {
            Victim::Caesar => self.advance_caesar(),
            Victim::Vigenere => self.advance_vigenere(),
        }
        self.step += 1;
        self.best_row = self
            .rows
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.score
                    .partial_cmp(&b.score)
                    .expect("row scores are finite")
            })
            .map(|(i, _)| i);
    }

    fn push_row(&mut self, label: String, score: f64, note: String) {
        self.rows.push(Row { label, score, note });
    }

    fn advance_caesar(&mut self) {
        let shift = self.step as u8;
        let plaintext = caesar::decrypt(&self.ciphertext, shift);
        let score = chi_squared(&plaintext);
        let preview: String = plaintext.chars().take(30).collect();
        self.push_row(format!("shift {shift:2}"), score, preview);
        if self.step == 25 {
            // All shifts tried: settle on the cipher core's own verdict.
            let best = caesar::crack(&self.ciphertext).remove(0);
            self.recovered_key = format!("shift {}", best.shift);
            self.recovered_plaintext = best.plaintext;
            self.done = true;
        }
    }

    fn advance_vigenere(&mut self) {
        if self.step < VIGENERE_MAX_LEN {
            // Phase 1: stream the precomputed key-length scan, in length
            // order rather than the core's ranked order.
            let score = self
                .scan
                .iter()
                .find(|s| s.length == self.step + 1)
                .expect("scan covers 1..=12");
            self.push_row(
                format!("key len {:2}", score.length),
                (0.066 - score.ioc).abs(), // distance from English IoC: lower is better
                format!("IoC {:.4}", score.ioc),
            );
            return;
        }
        // Phase 2: solve columns of the chosen key length.
        let chosen_len = self.chosen_key_length();
        let column = self.step - VIGENERE_MAX_LEN;
        if column >= chosen_len {
            let (key, plaintext) = vigenere::crack(&self.ciphertext, chosen_len);
            self.recovered_key = key;
            self.recovered_plaintext = plaintext;
            self.done = true;
            return;
        }
        let mut candidates = vigenere::crack_column(&self.ciphertext, chosen_len, column);
        let best = candidates.remove(0);
        let runner_up = candidates.first();
        let note = match runner_up {
            Some(next) => format!("runner-up '{}' {:.3}", next.key_letter, next.score),
            None => String::new(),
        };
        self.push_row(
            format!("column {column} → '{}'", best.key_letter),
            best.score,
            note,
        );
    }

    /// The scan's pick: smallest length entering the English IoC band, the
    /// same heuristic the cipher core's tests pin down.
    fn chosen_key_length(&self) -> usize {
        let mut by_length: Vec<&vigenere::KeyLengthScore> = self.scan.iter().collect();
        by_length.sort_by_key(|s| s.length);
        by_length
            .iter()
            .find(|s| s.ioc >= 0.06)
            .map(|s| s.length)
            .unwrap_or(VIGENERE_MAX_LEN)
    }
}

/// Autocrack screen state.
#[derive(Debug)]
pub struct State {
    pub phase: Phase,
    /// Test seam: the last key routed here (only the active mode receives keys).
    pub(crate) last_key: Option<KeyEvent>,
    /// Test seam: ticks received while this mode is active.
    pub(crate) tick_count: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            phase: Phase::Select { selection: 0 },
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
        match (key.code, &mut self.phase) {
            (KeyCode::Char('r'), _) => self.phase = Phase::Select { selection: 0 },
            (KeyCode::Up, Phase::Select { selection }) => {
                *selection = selection.saturating_sub(1);
            }
            (KeyCode::Down, Phase::Select { selection }) => {
                if *selection + 1 < VICTIMS.len() {
                    *selection += 1;
                }
            }
            (KeyCode::Enter, Phase::Select { selection }) => {
                let victim = VICTIMS[*selection];
                self.phase = Phase::Running(Run::new(victim));
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if let Phase::Running(run) = &mut self.phase {
            run.advance();
        }
    }

    pub fn key_hints(&self) -> &'static str {
        "Up/Dn victim · Enter run · r reset"
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match &self.phase {
            Phase::Select { selection } => self.render_select(frame, area, *selection),
            Phase::Running(run) => self.render_run(frame, area, run),
        }
    }

    fn render_select(&self, frame: &mut Frame, area: Rect, selection: usize) {
        let items: Vec<ListItem> = VICTIMS
            .iter()
            .map(|&victim| {
                let ct = victim.ciphertext();
                let preview: String = ct.chars().take(26).collect();
                ListItem::new(Text::from(vec![
                    Line::from(victim.name()),
                    Line::from(format!("  {} — {blurb}", preview, blurb = victim.blurb())),
                    Line::from(""),
                ]))
            })
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(selection));
        let list = List::new(items)
            .block(Block::bordered().title(" Pick a victim cipher "))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut { list_state });
    }

    fn render_run(&self, frame: &mut Frame, area: Rect, run: &Run) {
        let [list_area, status_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(5)]).areas(area);

        // Show the tail of the streamed rows so late candidates stay visible.
        let visible = 12;
        let start = run.rows.len().saturating_sub(visible);
        let items: Vec<ListItem> = run
            .rows
            .iter()
            .enumerate()
            .skip(start)
            .map(|(i, row)| {
                let marker = if Some(i) == run.best_row { "▶" } else { " " };
                ListItem::new(Line::from(format!(
                    "{marker} {:<18} {:8.4}  {}",
                    row.label, row.score, row.note
                )))
            })
            .collect();
        let title = if run.done {
            " Attack — done "
        } else {
            " Attack — running "
        };
        let list = List::new(items).block(Block::bordered().title(title));
        frame.render_widget(list, list_area);

        let status = if run.done {
            vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!("Key:      {}", run.recovered_key)),
                Line::from(format!("Plaintext: {}", {
                    run.recovered_plaintext.chars().take(48).collect::<String>()
                })),
                Line::from("r — back to victim list"),
            ]
        } else {
            vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!("Step {}/{}", run.rows.len(), run.total_steps())),
                Line::from("▶ marks the best candidate so far (lowest chi-squared)"),
            ]
        };
        let paragraph = Paragraph::new(Text::from(status)).alignment(Alignment::Left);
        frame.render_widget(paragraph, status_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    fn rows_rendered(state: &State) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(90, 30)).unwrap();
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
    fn selection_moves_and_clamps() {
        let mut state = State::new();
        assert_eq!(state.phase, Phase::Select { selection: 0 });
        state.handle_key(press(KeyCode::Up)); // clamps at top
        assert_eq!(state.phase, Phase::Select { selection: 0 });
        state.handle_key(press(KeyCode::Down));
        assert_eq!(state.phase, Phase::Select { selection: 1 });
        state.handle_key(press(KeyCode::Down)); // clamps at bottom
        assert_eq!(state.phase, Phase::Select { selection: 1 });
    }

    #[test]
    fn enter_starts_the_selected_victim() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.victim, Victim::Caesar),
            other => panic!("expected running, got {other:?}"),
        }

        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.victim, Victim::Vigenere),
            other => panic!("expected running, got {other:?}"),
        }
    }

    #[test]
    fn caesar_run_streams_all_shifts_then_recovers_the_key() {
        let mut run = Run::new(Victim::Caesar);
        assert_eq!(run.total_steps(), 26);
        while !run.done {
            run.advance();
        }
        assert_eq!(run.rows.len(), 26, "one row per shift");
        assert_eq!(run.recovered_key, "shift 11");
        assert_eq!(run.recovered_plaintext, ENGLISH_SAMPLE);
        // Best row is the recovered shift.
        let best = &run.rows[run.best_row.unwrap()];
        assert!(best.label.contains("11"));
    }

    #[test]
    fn vigenere_run_scans_then_recovers_the_key() {
        let mut run = Run::new(Victim::Vigenere);
        while !run.done {
            run.advance();
        }
        assert!(
            run.rows.len() >= VIGENERE_MAX_LEN + 5,
            "scan rows plus column solves"
        );
        assert_eq!(run.recovered_key, "lemon");
        assert_eq!(run.recovered_plaintext, ENGLISH_SAMPLE);
    }

    #[test]
    fn tick_advances_a_running_attack_and_r_resets() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        state.tick();
        state.tick();
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.rows.len(), 2),
            other => panic!("expected running, got {other:?}"),
        }
        state.handle_key(press(KeyCode::Char('r')));
        assert_eq!(state.phase, Phase::Select { selection: 0 });
    }

    #[test]
    fn done_attack_stops_advancing() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        for _ in 0..40 {
            state.tick();
        }
        let rows = match &state.phase {
            Phase::Running(run) => {
                assert!(run.done);
                run.rows.len()
            }
            other => panic!("expected running, got {other:?}"),
        };
        state.tick();
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.rows.len(), rows, "no extra rows after done"),
            other => panic!("expected running, got {other:?}"),
        }
    }

    #[test]
    fn select_screen_lists_victims_with_ciphertext_previews() {
        let state = State::new();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("Caesar (shift 11)"));
        assert!(text.contains("Vigenère (key \"lemon\")"));
        assert!(text.contains("Pick a victim"));
    }

    #[test]
    fn running_screen_shows_rows_and_done_screen_shows_key_and_plaintext() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        for _ in 0..3 {
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("running"));
        assert!(text.contains("shift  0"));

        while let Phase::Running(run) = &state.phase {
            if run.done {
                break;
            }
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("done"));
        assert!(text.contains("Key:      shift 11"));
        assert!(text.contains("Plaintext: Cryptanalysis"));
    }
}
