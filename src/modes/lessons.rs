//! Lessons mode: cryptopals-style walk-throughs of why the demo ciphers die.
//! Pick a lesson, step through it; every number the lessons cite is computed
//! by the same `cipher` core the demos run on.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};

/// One guided lesson: a title and ordered steps of prose.
pub struct Lesson {
    pub title: &'static str,
    pub steps: &'static [&'static str],
}

/// The shipped starter lessons, matching the attacks the app can run.
pub const LESSONS: [Lesson; 2] = [
    Lesson {
        title: "Why Caesar dies — frequency analysis",
        steps: &[
            "The Caesar cipher shifts every letter by the same amount. That \
             single mistake is fatal: whatever it does to 'e', it does to \
             every 'e' in the message.",
            "English is lumpy. 'e' is about 12.7% of letters, 't' 9.1%, \
             'a' 8.2% — while 'z' is 0.07%. Count a few hundred letters of \
             English and the counts are a fingerprint of the language itself.",
            "A Caesar shift slides that fingerprint along the alphabet \
             without changing its shape. Count the ciphertext letters and \
             you see the same lumps, just rotated.",
            "The attack: try all 26 shifts, score each decryption by \
             chi-squared distance from English frequencies, and pick the \
             best. Chi-squared sums (observed - expected)^2 / expected over \
             all 26 letters — low means 'looks like English'.",
            "Watch the Autocrack tab run this live: 26 candidates stream in, \
             each with its score, and the true shift wins because only it \
             turns the lumps back into English's lumps.",
            "The lesson underneath: any cipher where one plaintext letter \
             always maps to one ciphertext letter preserves the fingerprint. \
             Substitute anything, keep the mapping fixed, and you lose.",
        ],
    },
    Lesson {
        title: "Breaking repeating-XOR",
        steps: &[
            "XOR with a repeating key is a classic 'clever enough to feel \
             safe, weak enough to teach a lesson' construction. Byte k of \
             the key XORs every k-th plaintext byte.",
            "That periodicity is the flaw: positions sharing a key byte form \
             a column, and each column is just a single-byte XOR — the \
             easiest cipher there is.",
            "Attack step 1 — find the key length. Try each candidate length: \
             crack the columns and score the whole decryption as English. \
             The length whose break reads most like English wins. (The \
             textbook alternative, Hamming distance between blocks, needs \
             far more ciphertext to separate signal from noise.)",
            "Attack step 2 — solve each column. For a column, XOR it with \
             all 256 possible byte values and score each result by letter \
             and space frequencies. English is ~17% spaces; a wrong key byte \
             scatters them, so the right byte stands out.",
            "Stitch the winning bytes together and you have the key. Open \
             the Traffic Simulator tab, pick a session, press Enter, and \
             watch the plaintext reveal byte by byte.",
            "The lesson underneath: reuse is the enemy. A one-time pad is \
             unbreakable; repeat the pad and every repetition leaks \
             structure an attacker can line up and solve.",
        ],
    },
];

/// Lessons screen state: the open lesson and the current step, if any.
#[derive(Debug)]
pub struct State {
    /// `(lesson index, step)` while a lesson is open.
    pub open: Option<(usize, usize)>,
    pub selection: usize,
    /// Test seam: the last key routed here (only the active mode receives keys).
    pub(crate) last_key: Option<KeyEvent>,
    /// Test seam: ticks received while this mode is active.
    pub(crate) tick_count: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            open: None,
            selection: 0,
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
            KeyCode::Esc | KeyCode::Char('r') => self.open = None,
            KeyCode::Up if self.open.is_none() => self.selection = self.selection.saturating_sub(1),
            KeyCode::Down if self.open.is_none() => {
                if self.selection + 1 < LESSONS.len() {
                    self.selection += 1;
                }
            }
            KeyCode::Enter if self.open.is_none() => self.open = Some((self.selection, 0)),
            KeyCode::Right | KeyCode::Char('n') => {
                if let Some((lesson, step)) = &mut self.open {
                    let total = LESSONS[*lesson].steps.len();
                    if *step + 1 < total {
                        *step += 1;
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('p') => {
                if let Some((_, step)) = &mut self.open {
                    *step = step.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn key_hints(&self) -> &'static str {
        "Up/Dn lesson · Left/Right step · Esc list"
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self.open {
            Some((lesson, step)) => self.render_lesson(frame, area, lesson, step),
            None => self.render_list(frame, area),
        }
    }

    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = LESSONS
            .iter()
            .map(|lesson| {
                ListItem::new(Line::from(format!(
                    "{} ({} steps)",
                    lesson.title,
                    lesson.steps.len()
                )))
            })
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(self.selection));
        let list = List::new(items)
            .block(Block::bordered().title(" Lessons "))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut { list_state });
    }

    fn render_lesson(&self, frame: &mut Frame, area: Rect, lesson: usize, step: usize) {
        let [header_area, body_area] =
            Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(area);
        let title = &LESSONS[lesson];
        let header = Paragraph::new(Text::from(vec![
            Line::from(title.title),
            Line::from(format!("step {} of {}", step + 1, title.steps.len())),
        ]))
        .block(Block::bordered())
        .alignment(Alignment::Left);
        frame.render_widget(header, header_area);

        let body = Paragraph::new(title.steps[step])
            .block(Block::bordered().title(" Step "))
            .wrap(Wrap { trim: true });
        frame.render_widget(body, body_area);
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
    fn list_shows_both_starter_lessons() {
        let state = State::new();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("Why Caesar dies — frequency analysis"));
        assert!(text.contains("Breaking repeating-XOR"));
        assert!(text.contains("steps)"), "step counts shown");
    }

    #[test]
    fn enter_opens_the_selected_lesson_at_step_one() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((1, 0)));
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("step 1 of"), "counter starts at 1 of N");
        assert!(text.contains("XOR with a repeating key"), "first step body");
    }

    #[test]
    fn steps_navigate_forward_and_back_clamped() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((0, 0)));
        state.handle_key(press(KeyCode::Left)); // clamp at first
        assert_eq!(state.open, Some((0, 0)));
        state.handle_key(press(KeyCode::Right));
        state.handle_key(press(KeyCode::Char('n')));
        assert_eq!(state.open, Some((0, 2)));
        state.handle_key(press(KeyCode::Char('p')));
        assert_eq!(state.open, Some((0, 1)));
        let last = LESSONS[0].steps.len() - 1;
        state.open = Some((0, last));
        state.handle_key(press(KeyCode::Right)); // clamp at last
        assert_eq!(state.open, Some((0, last)));
    }

    #[test]
    fn escape_returns_to_the_list() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        state.handle_key(press(KeyCode::Right));
        assert!(state.open.is_some());
        state.handle_key(press(KeyCode::Esc));
        assert_eq!(state.open, None);
        // 'r' also closes from inside a lesson.
        state.handle_key(press(KeyCode::Enter));
        state.handle_key(press(KeyCode::Char('r')));
        assert_eq!(state.open, None);
    }

    #[test]
    fn open_lesson_renders_counter_and_step_body() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        state.handle_key(press(KeyCode::Right));
        state.handle_key(press(KeyCode::Right));
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("step 3 of 6"), "counter shows current step");
        // Single words: wrapped lines break between words, so multi-word
        // phrases can straddle a row boundary.
        assert!(
            text.contains("fingerprint") && text.contains("shape"),
            "third step's content"
        );

        state.handle_key(press(KeyCode::Right));
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("step 4 of 6"));
        assert!(text.contains("chi-squared"), "fourth step's content");
    }
}
