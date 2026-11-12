//! Autocrack mode: pick a victim cipher, watch the attack run live.
//!
//! Caesar streams all 26 shift candidates, scored by chi-squared against
//! English. Vigenère first streams the index-of-coincidence key-length scan,
//! then solves each key column by per-column chi-squared, assembling the key
//! column by column. SWEET32 streams birthday-attack traffic stats until an
//! own/victim block collision leaks the cookie. One row lands per tick;
//! `advance` is pure, so the runner is fully testable without a terminal.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use crate::cipher::caesar;
use crate::cipher::freq::{ENGLISH_SAMPLE, chi_squared};
use crate::cipher::sweet32::{
    DEMO_BLOCK_BITS, Simulator, TICK_STRIDE, expected_collisions, first_collision_expectation,
};
use crate::cipher::vigenere;
use crate::cipher::{dlog, rsa};

/// Rho iterations per tick while FREAK runs.
const RHO_STRIDE: u64 = 4096;
/// BSGS steps per tick while Logjam runs.
const BSGS_STRIDE: u64 = 65_536;

/// The victim ciphers on offer, in list order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Victim {
    Caesar,
    Vigenere,
    Sweet32,
    Freak,
    Logjam,
}

pub const VICTIMS: [Victim; 5] = [
    Victim::Caesar,
    Victim::Vigenere,
    Victim::Sweet32,
    Victim::Freak,
    Victim::Logjam,
];

impl Victim {
    pub fn name(self) -> &'static str {
        match self {
            Victim::Caesar => "Caesar (shift 11)",
            Victim::Vigenere => "Vigenère (key \"lemon\")",
            Victim::Sweet32 => "SWEET32 (birthday attack)",
            Victim::Freak => "FREAK (export RSA)",
            Victim::Logjam => "Logjam (export DHE)",
        }
    }

    /// One-line explanation shown under the victim's name.
    pub fn blurb(self) -> &'static str {
        match self {
            Victim::Caesar => "monoalphabetic — dies to frequency analysis",
            Victim::Vigenere => "polyalphabetic — dies to IoC + per-column analysis",
            Victim::Sweet32 => "small block space — dies to the birthday problem",
            Victim::Freak => "export-grade modulus — dies to factoring",
            Victim::Logjam => "export-grade group — dies to discrete log",
        }
    }

    /// The sample ciphertext this victim carries, derived from the shared
    /// English sample so every mode tells the same story. The attack demos
    /// show their parameters instead of text.
    pub fn ciphertext(self) -> String {
        match self {
            Victim::Caesar => caesar::encrypt(ENGLISH_SAMPLE, 11),
            Victim::Vigenere => vigenere::encrypt(ENGLISH_SAMPLE, "lemon"),
            Victim::Sweet32 => Simulator::demo()
                .preview_blocks(4)
                .iter()
                .map(|v| format!("{v:05x}"))
                .collect::<Vec<_>>()
                .join(" "),
            Victim::Freak => format!("N = {} (toy 62-bit export)", rsa::N),
            Victim::Logjam => format!("p = {}, g = 5 (toy ~2^40 export)", dlog::P),
        }
    }
}

/// Which screen the mode is on.
#[derive(Debug, PartialEq)]
pub enum Phase {
    /// Choosing a victim; the index selects into [`VICTIMS`].
    Select { selection: usize },
    /// An attack is running or finished (`run.done` distinguishes them).
    Running(Box<Run>),
}

/// One visible step of an attack: what was tried, how English it looked.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub label: String,
    pub score: f64,
    pub note: String,
}

/// A live attack. `advance` pushes the next row; when the plan is exhausted
/// the run is `done` and carries the recovered key and plaintext (for
/// classical victims) or the structural result and leak (for the attack demos).
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
    /// SWEET32 only: the birthday simulator, stepped `TICK_STRIDE` per tick.
    birthday: Option<Simulator>,
    /// FREAK only: the rho factoring run.
    rho: Option<rsa::RhoSolver>,
    /// Logjam only: the BSGS discrete-log run.
    bsgs: Option<dlog::BsgsSolver>,
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
            _ => Vec::new(),
        };
        let birthday = match victim {
            Victim::Sweet32 => Some(Simulator::demo()),
            _ => None,
        };
        let rho = match victim {
            Victim::Freak => Some(rsa::RhoSolver::demo()),
            _ => None,
        };
        let bsgs = match victim {
            Victim::Logjam => Some(dlog::BsgsSolver::demo()),
            _ => None,
        };
        Self {
            victim,
            ciphertext,
            rows: Vec::new(),
            best_row: None,
            step: 0,
            scan,
            birthday,
            rho,
            bsgs,
            done: false,
            recovered_key: String::new(),
            recovered_plaintext: String::new(),
        }
    }

    /// Total steps this run will take (rows it will stream). Solver run
    /// lengths are draws; report expectations as estimates only.
    pub fn total_steps(&self) -> usize {
        match self.victim {
            Victim::Caesar => 26,
            // 12 scan rows, then one solve row per key column of the chosen length.
            Victim::Vigenere => VIGENERE_MAX_LEN + VIGENERE_MAX_LEN,
            // Expectation scaled by ~2.3: only own/victim pairs exploit, and
            // roughly 4/9 of pairs qualify.
            Victim::Sweet32 => ((first_collision_expectation(DEMO_BLOCK_BITS) * 2.3
                / TICK_STRIDE as f64)
                .ceil() as usize)
                .max(1),
            Victim::Freak => 3 + (rsa::expected_iterations() / RHO_STRIDE as f64).ceil() as usize,
            Victim::Logjam => 3 + (dlog::expected_steps() / BSGS_STRIDE as f64).ceil() as usize,
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
            Victim::Sweet32 => self.advance_birthday(),
            Victim::Freak => self.advance_freak(),
            Victim::Logjam => self.advance_logjam(),
        }
        self.step += 1;
        // "Best candidate" is a cryptanalysis concept; the attack demos'
        // rows have no ordering to highlight.
        if matches!(self.victim, Victim::Caesar | Victim::Vigenere) {
            // "Best candidate" highlight is a cryptanalysis concept; birthday
            // stats rows have no ordering to highlight.
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

    fn advance_birthday(&mut self) {
        // Extract the tick's outcome before touching self's rows: the
        // simulator borrow and row pushes can't coexist.
        let (sent, collisions, exploit) = {
            let sim = self
                .birthday
                .as_mut()
                .expect("SWEET32 runs carry a simulator");
            sim.step(TICK_STRIDE);
            (sim.records_sent, sim.collisions, sim.exploit)
        };
        let expected = expected_collisions(sent, DEMO_BLOCK_BITS);
        self.push_row(
            format!("{sent:6} blocks"),
            expected,
            format!("collisions {collisions} | expected {expected:.2}"),
        );
        if let Some(c) = exploit {
            let cookie_hex: String = c.leaked_cookie.iter().map(|b| format!("{b:02x}")).collect();
            self.push_row(
                format!("COLLISION {:05x}", c.block_value),
                expected,
                format!("record #{} vs own #{}", c.victim_index, c.own_index),
            );
            self.recovered_key = format!(
                "birthday collision after {} blocks (expected ≈{})",
                c.blocks_sent,
                first_collision_expectation(DEMO_BLOCK_BITS).round() as u64
            );
            self.recovered_plaintext =
                format!("leaked cookie: {cookie_hex} (XOR of the colliding records)");
            self.done = true;
        }
    }

    /// One row of the 2015 export-downgrade exchange, indexed by `step`
    /// (0, 1, 2). `strong` and `weak` name the two sides of the strip.
    fn push_downgrade_row(&mut self, index: usize, strong: &str, weak: &str) {
        let (label, note) = match index {
            0 => ("ClientHello", format!("offers strong {strong}")),
            1 => ("MITM", format!("strips the offer down to {weak}")),
            _ => (
                "ServerHello",
                "complies — the export key is signed as if strong".to_string(),
            ),
        };
        self.push_row(label.to_string(), 0.0, note);
    }

    fn advance_freak(&mut self) {
        if self.step < 3 {
            self.push_downgrade_row(
                self.step,
                "RSA-2048",
                "RSA_EXPORT (512-bit, 1990s 'export grade')",
            );
            return;
        }
        let (iterations, factor) = {
            let solver = self.rho.as_mut().expect("FREAK runs carry a rho solver");
            solver.step(RHO_STRIDE);
            (solver.iterations, solver.factor)
        };
        let expected = rsa::expected_iterations();
        self.push_row(
            format!("rho {iterations:6}"),
            expected,
            "gcd(x−y, N) = 1 so far".to_string(),
        );
        if let Some(factor) = factor {
            let other = rsa::N / factor;
            self.push_row(
                "FACTOR FOUND".into(),
                expected,
                "gcd splits N into two primes".to_string(),
            );
            let d = rsa::private_exponent();
            let decrypted = String::from_utf8_lossy(&{
                let m = rsa::decrypt_int(rsa::captured_ciphertext(), d);
                let bytes = m.to_le_bytes();
                bytes[..rsa::SECRET.len()].to_vec()
            })
            .to_string();
            self.recovered_key = format!(
                "N = {factor} × {other} after {iterations} iterations (N^¼ ≈ {expected:.0})"
            );
            self.recovered_plaintext = format!("d = {d} → premaster secret: {decrypted}");
            self.done = true;
        }
    }

    fn advance_logjam(&mut self) {
        if self.step < 3 {
            self.push_downgrade_row(self.step, "DHE-2048", "DHE_EXPORT (512-bit group)");
            return;
        }
        let (steps, built, total_babies, exponent) = {
            let solver = self.bsgs.as_mut().expect("Logjam runs carry a BSGS solver");
            solver.step(BSGS_STRIDE);
            let built = solver.baby_index;
            let total = solver.m;
            (solver.steps, built, total, solver.exponent)
        };
        let expected = dlog::expected_steps();
        let note = if built < total_babies {
            format!("baby steps {built}/{total_babies}")
        } else {
            format!("giant probing (baby steps done: {total_babies})")
        };
        self.push_row(format!("bsgs {steps:7}"), expected, note);
        if let Some(y) = exponent {
            self.push_row(
                "DISCRETE LOG".into(),
                expected,
                "exponent reproduces the server's public value".to_string(),
            );
            let secret = dlog::shared_secret();
            let session = String::from_utf8_lossy(&dlog::decrypt_session(secret)).to_string();
            self.recovered_key = format!("y = {y} after {steps} steps (√p ≈ {expected:.0})");
            self.recovered_plaintext = format!("shared secret {secret} → session: {session}");
            self.done = true;
        }
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
                self.phase = Phase::Running(Box::new(Run::new(victim)));
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

        let status = match (run.victim, run.done) {
            (v, true) if !matches!(v, Victim::Caesar | Victim::Vigenere) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!("Result:   {}", run.recovered_key)),
                Line::from(format!("Leak:     {}", {
                    run.recovered_plaintext.chars().take(72).collect::<String>()
                })),
                Line::from("r — back to victim list"),
            ],
            (_, true) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!("Key:      {}", run.recovered_key)),
                Line::from(format!("Plaintext: {}", {
                    run.recovered_plaintext.chars().take(48).collect::<String>()
                })),
                Line::from("r — back to victim list"),
            ],
            (Victim::Sweet32, false) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!(
                    "Blocks sent {} (expected ≈{} total)",
                    run.birthday
                        .as_ref()
                        .expect("simulator present")
                        .records_sent,
                    run.total_steps() * TICK_STRIDE
                )),
                Line::from("waiting for an own/victim block collision…"),
            ],
            (Victim::Freak, false) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!(
                    "rho iterations {} (N^¼ ≈ {:.0})",
                    run.rho.as_ref().expect("solver present").iterations,
                    rsa::expected_iterations()
                )),
                Line::from("waiting for the export modulus to factor…"),
            ],
            (Victim::Logjam, false) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!(
                    "bsgs steps {} (√p ≈ {:.0})",
                    run.bsgs.as_ref().expect("solver present").steps,
                    dlog::expected_steps()
                )),
                Line::from("waiting for the discrete log to land…"),
            ],
            (_, false) => vec![
                Line::from(format!("Victim:   {}", run.victim.name())),
                Line::from(format!("Step {}/{}", run.rows.len(), run.total_steps())),
                Line::from("▶ marks the best candidate so far (lowest chi-squared)"),
            ],
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
        state.handle_key(press(KeyCode::Down));
        assert_eq!(state.phase, Phase::Select { selection: 2 });
        state.handle_key(press(KeyCode::Down));
        assert_eq!(state.phase, Phase::Select { selection: 3 });
        state.handle_key(press(KeyCode::Down));
        assert_eq!(state.phase, Phase::Select { selection: 4 });
        state.handle_key(press(KeyCode::Down)); // clamps at bottom
        assert_eq!(state.phase, Phase::Select { selection: 4 });
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
    fn sweet32_run_streams_stats_then_collides() {
        let mut run = Run::new(Victim::Sweet32);
        assert!(run.total_steps() >= 1);
        let mut ticks = 0;
        while !run.done && ticks < 60 {
            run.advance();
            ticks += 1;
        }
        assert!(run.done, "collision lands within 60 ticks (≈12k blocks)");
        // Every stats row carries the three counters the spec requires; the
        // final event row is the collision itself.
        let stats = &run.rows[..run.rows.len() - 1];
        assert!(
            stats
                .iter()
                .all(|row| row.note.contains("collisions") && row.note.contains("expected"))
        );
        assert!(run.recovered_key.contains("birthday collision after"));
        assert!(run.recovered_key.contains("expected ≈"));
        assert!(run.recovered_plaintext.contains("leaked cookie: "));
        // No best-candidate highlight on a birthday run.
        assert_eq!(run.best_row, None);
        // The last row is the collision event.
        assert!(run.rows.last().unwrap().label.contains("COLLISION"));
    }

    #[test]
    fn sweet32_selection_reaches_the_third_victim() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.victim, Victim::Sweet32),
            other => panic!("expected running, got {other:?}"),
        }
    }

    #[test]
    fn sweet32_screens_render_counters_and_done_result() {
        let mut state = State::new();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("SWEET32 (birthday attack)"), "victim listed");

        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        state.tick();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("Blocks sent"), "block counter while running");
        assert!(text.contains("collisions"), "stats rows stream");

        for _ in 0..60 {
            if let Phase::Running(run) = &state.phase
                && run.done
            {
                break;
            }
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("Result:   birthday collision after"));
        assert!(text.contains("Leak:     leaked cookie: "));
        assert!(text.contains("COLLISION"));
    }

    #[test]
    fn freak_run_downgrades_then_factors() {
        let mut run = Run::new(Victim::Freak);
        let mut ticks = 0;
        while !run.done && ticks < 60 {
            run.advance();
            ticks += 1;
        }
        assert!(run.done, "rho terminates well within 60 ticks");
        // The downgrade narration comes first.
        let labels: Vec<&str> = run.rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels[..3], ["ClientHello", "MITM", "ServerHello"]);
        // Then rho rows, then the factor event.
        assert!(run.rows[3].label.starts_with("rho"));
        assert!(run.rows.last().unwrap().label.contains("FACTOR"));
        assert!(run.recovered_key.contains("after"), "iteration count cited");
        assert!(run.recovered_key.contains("N^¼ ≈"), "bound cited");
        assert!(
            run.recovered_plaintext
                .contains("premaster secret: PMS-42!")
        );
        assert_eq!(
            run.best_row, None,
            "no candidate highlight on structural attacks"
        );
    }

    #[test]
    fn logjam_run_downgrades_then_solves_the_dlog() {
        let mut run = Run::new(Victim::Logjam);
        let mut ticks = 0;
        while !run.done && ticks < 80 {
            run.advance();
            ticks += 1;
        }
        assert!(run.done, "BSGS terminates well within 80 ticks");
        let labels: Vec<&str> = run.rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels[..3], ["ClientHello", "MITM", "ServerHello"]);
        assert!(run.rows[3].label.starts_with("bsgs"));
        assert!(run.rows.last().unwrap().label.contains("DISCRETE LOG"));
        assert!(run.recovered_key.contains("y = 314159265"), "true exponent");
        assert!(run.recovered_key.contains("√p ≈"), "bound cited");
        assert!(run.recovered_plaintext.contains("GET /export HTTP/1.1"));
        assert_eq!(run.best_row, None);
    }

    #[test]
    fn selection_reaches_the_fourth_and_fifth_victims() {
        let mut state = State::new();
        for _ in 0..3 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.victim, Victim::Freak),
            other => panic!("expected running, got {other:?}"),
        }

        let mut state = State::new();
        for _ in 0..4 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        match &state.phase {
            Phase::Running(run) => assert_eq!(run.victim, Victim::Logjam),
            other => panic!("expected running, got {other:?}"),
        }
    }

    #[test]
    fn export_attacks_render_counters_and_done_chains() {
        let mut state = State::new();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("FREAK (export RSA)"), "victim listed");
        assert!(text.contains("Logjam (export DHE)"), "victim listed");

        // FREAK: downgrade rows then the rho counter, then the full chain.
        for _ in 0..3 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        state.tick();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("ClientHello"), "downgrade narrated");
        for _ in 0..60 {
            if let Phase::Running(run) = &state.phase
                && run.done
            {
                break;
            }
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("rho"), "solver rows streamed");
        assert!(text.contains("Result:   N = "), "factored result");
        assert!(
            text.contains("premaster secret: PMS-42!"),
            "chain completes"
        );

        // Logjam: same shape.
        state.handle_key(press(KeyCode::Char('r')));
        for _ in 0..4 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        for _ in 0..80 {
            if let Phase::Running(run) = &state.phase
                && run.done
            {
                break;
            }
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(
            text.contains("Result:   y = 314159265"),
            "exponent recovered"
        );
        assert!(
            text.contains("session: GET /export HTTP/1.1"),
            "session decrypted"
        );
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
