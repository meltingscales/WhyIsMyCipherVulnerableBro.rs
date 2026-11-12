//! Traffic Simulator mode: browse fake intercepted sessions, inspect their
//! bytes as hex, then run the repeating-XOR break and watch the plaintext
//! reveal. All traffic is simulated locally — nothing touches a network.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};

use crate::cipher::xor;

/// How many plaintext bytes each tick reveals.
const REVEAL_CHUNK: usize = 16;

/// A template the mode builds its simulated sessions from.
struct Template {
    hostname: &'static str,
    cipher_label: &'static str,
    key: &'static [u8],
    plaintext: &'static [u8],
}

/// The built-in fake capture: three "intercepted" requests, each protected by
/// a repeating XOR key of a different length. Plaintexts are rich enough in
/// English for per-column frequency scoring to recover the keys.
const TEMPLATES: [Template; 3] = [
    Template {
        hostname: "login.example.com",
        cipher_label: "XOR-2B",
        key: b"k?",
        plaintext: b"POST /login HTTP/1.1\r\nHost: login.example.com\r\nContent-Type: \
            application/x-www-form-urlencoded\r\nusername=alice&password=hunter2&submit=login\r\n",
    },
    Template {
        hostname: "api.example.com",
        cipher_label: "XOR-3B",
        key: b"abc",
        plaintext: b"GET /api/v1/messages HTTP/1.1\r\nHost: api.example.com\r\nAccept: \
            application/json\r\nAuthorization: Basic dXNlcjpwYXNz\r\nthe quick brown fox jumps \
            over the lazy dog while statistics accumulate happily\r\n",
    },
    Template {
        hostname: "static.example.com",
        cipher_label: "XOR-5B",
        key: b"watch",
        plaintext: b"GET /assets/app.js HTTP/1.1\r\nHost: static.example.com\r\nUser-Agent: \
            curl/8.0\r\nAccept: */*\r\nenglish text fills the capture because every key byte \
            needs a column of letters to vote on, and short columns vote badly\r\n",
    },
];

/// One simulated session and its decrypt state.
#[derive(Debug, PartialEq)]
pub struct Session {
    pub hostname: String,
    pub cipher_label: String,
    pub ciphertext: Vec<u8>,
    pub key_len: usize,
    /// Bytes of recovered plaintext revealed so far (counts up on tick).
    pub revealed: usize,
    /// None until Enter runs the break; then the recovered key bytes.
    pub recovered_key: Option<Vec<u8>>,
}

impl Session {
    fn from_template(t: &Template) -> Self {
        Self {
            hostname: t.hostname.to_string(),
            cipher_label: t.cipher_label.to_string(),
            ciphertext: xor::apply(t.plaintext, t.key),
            key_len: t.key.len(),
            revealed: 0,
            recovered_key: None,
        }
    }

    pub fn decrypted(&self) -> bool {
        self.recovered_key.is_some()
    }

    /// The full recovered plaintext under the recovered key (empty if none).
    pub fn recovered_plaintext(&self) -> Vec<u8> {
        match &self.recovered_key {
            Some(key) => xor::apply(&self.ciphertext, key),
            None => Vec::new(),
        }
    }
}

/// Traffic Simulator screen state.
#[derive(Debug)]
pub struct State {
    pub sessions: Vec<Session>,
    pub selection: usize,
    /// Test seam: the last key routed here (only the active mode receives keys).
    pub(crate) last_key: Option<KeyEvent>,
    /// Test seam: ticks received while this mode is active.
    pub(crate) tick_count: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: TEMPLATES.iter().map(Session::from_template).collect(),
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
            KeyCode::Up => self.selection = self.selection.saturating_sub(1),
            KeyCode::Down => {
                if self.selection + 1 < self.sessions.len() {
                    self.selection += 1;
                }
            }
            KeyCode::Char('r') => self.reset_all(),
            KeyCode::Enter => self.decrypt_selected(),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Reveal progresses for any decrypted session that is not done.
        for session in &mut self.sessions {
            if session.decrypted() {
                let total = session.recovered_plaintext().len();
                session.revealed = (session.revealed + REVEAL_CHUNK).min(total);
            }
        }
    }

    /// Run the repeating-XOR break on the selected session: first guess the
    /// key length by crack-and-score, then solve the columns at that length —
    /// the same two-stage attack the lessons describe.
    fn decrypt_selected(&mut self) {
        if let Some(session) = self.sessions.get_mut(self.selection)
            && !session.decrypted()
        {
            let guesses = xor::guess_key_length(&session.ciphertext, 2, 5);
            let key_len = guesses.first().map(|g| g.length).unwrap_or(session.key_len);
            session.recovered_key = Some(xor::crack_key(&session.ciphertext, key_len));
            session.revealed = 0;
        }
    }

    /// Every session back to undecrypted, selection back to the top.
    fn reset_all(&mut self) {
        for session in &mut self.sessions {
            session.recovered_key = None;
            session.revealed = 0;
        }
        self.selection = 0;
    }

    pub fn key_hints(&self) -> &'static str {
        "Up/Dn session · Enter decrypt · r reset"
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(area);
        self.render_list(frame, list_area);
        self.render_detail(frame, detail_area);
    }

    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .map(|s| {
                let status = if s.decrypted() {
                    format!(
                        "  [{}] decrypted {} of {} bytes",
                        s.cipher_label,
                        s.revealed,
                        s.recovered_plaintext().len()
                    )
                } else {
                    format!(
                        "  [{}] {} bytes captured",
                        s.cipher_label,
                        s.ciphertext.len()
                    )
                };
                ListItem::new(Text::from(vec![
                    Line::from(s.hostname.clone()),
                    Line::from(status),
                ]))
            })
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(self.selection));
        let list = List::new(items)
            .block(Block::bordered().title(" Intercepted sessions "))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut { list_state });
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let session = &self.sessions[self.selection];
        let [hex_area, plain_area] =
            Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

        let hex_block = Block::bordered().title(format!(" {} — ciphertext ", session.hostname));
        let hex_text = Text::from(hex_rows(&session.ciphertext, 8));
        frame.render_widget(Paragraph::new(hex_text).block(hex_block), hex_area);

        let (title, body) = if session.decrypted() {
            let pt = session.recovered_plaintext();
            let shown: Vec<u8> = pt[..session.revealed.min(pt.len())].to_vec();
            let shown_text = String::from_utf8_lossy(&shown).replace(['\r', '\n'], " ");
            let key = session
                .recovered_key
                .as_ref()
                .expect("decrypted sessions carry keys");
            let key_hex = key.iter().map(|b| format!("{b:02x}")).collect::<String>();
            (
                format!(" Recovered — key {key_hex} "),
                vec![
                    Line::from(format!("revealed {}/{} bytes", session.revealed, pt.len())),
                    Line::from(shown_text),
                ],
            )
        } else {
            (
                " Plaintext ".to_string(),
                vec![Line::from("Enter runs the repeating-XOR break")],
            )
        };
        frame.render_widget(
            Paragraph::new(Text::from(body))
                .block(Block::bordered().title(title))
                .wrap(Wrap { trim: false }),
            plain_area,
        );
    }
}

/// Classic hex dump rows: `offset  xx xx xx …  ascii`, 8 bytes per row.
fn hex_rows(bytes: &[u8], per_row: usize) -> Vec<Line<'static>> {
    bytes
        .chunks(per_row)
        .enumerate()
        .map(|(row, chunk)| {
            let offset = row * per_row;
            let hex: String = chunk
                .iter()
                .map(|b| format!("{b:02x} "))
                .collect::<String>();
            let ascii: String = chunk
                .iter()
                .map(|&b| {
                    if (32..=126).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            Line::from(format!(
                "{offset:04x}  {hex:<per_row$} {ascii}",
                per_row = per_row * 3
            ))
        })
        .collect()
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
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
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
    fn sessions_list_with_host_label_and_length() {
        let state = State::new();
        assert_eq!(state.sessions.len(), 3);
        let text = rows_rendered(&state).join("\n");
        for host in ["login.example.com", "api.example.com", "static.example.com"] {
            assert!(text.contains(host), "{host} listed");
        }
        assert!(text.contains("bytes captured"), "lengths shown");
        assert!(text.contains("[XOR-3B]"), "cipher labels shown");
    }

    #[test]
    fn selection_moves_and_hex_view_follows() {
        let mut state = State::new();
        assert_eq!(state.selection, 0);
        assert!(
            rows_rendered(&state)
                .join("\n")
                .contains("login.example.com — ciphertext")
        );
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Down));
        assert_eq!(state.selection, 2);
        // The selected (third) session's ciphertext now feeds the hex pane.
        assert!(
            rows_rendered(&state)
                .join("\n")
                .contains("static.example.com — ciphertext")
        );
        // And moving back up still renders the second session.
        state.handle_key(press(KeyCode::Up));
        assert_eq!(state.selection, 1);
        assert!(
            rows_rendered(&state)
                .join("\n")
                .contains("api.example.com — ciphertext")
        );
    }

    #[test]
    fn enter_breaks_the_key_and_ticks_reveal_progressively() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter)); // session 0, 2-byte key
        {
            let s = &state.sessions[0];
            assert!(s.decrypted(), "break ran immediately");
            assert_eq!(
                s.recovered_key.as_deref(),
                Some(TEMPLATES[0].key),
                "key recovered"
            );
            assert_eq!(s.revealed, 0, "reveal starts at zero");
        }
        state.tick();
        let after_one = state.sessions[0].revealed;
        assert!(after_one > 0 && after_one < state.sessions[0].recovered_plaintext().len());
        // Ticks never overshoot.
        for _ in 0..100 {
            state.tick();
        }
        let s = &state.sessions[0];
        assert_eq!(s.revealed, s.recovered_plaintext().len(), "fully revealed");
        assert_eq!(s.recovered_plaintext(), TEMPLATES[0].plaintext);
    }

    #[test]
    fn all_three_sessions_break_to_their_true_keys() {
        let mut state = State::new();
        for (i, template) in TEMPLATES.iter().enumerate() {
            state.selection = i;
            state.handle_key(press(KeyCode::Enter));
            let s = &state.sessions[i];
            assert_eq!(
                s.recovered_key.as_deref(),
                Some(template.key),
                "session {i}"
            );
            assert_eq!(s.recovered_plaintext(), template.plaintext, "session {i}");
        }
    }

    #[test]
    fn r_clears_every_session_and_resets_selection() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        state.tick();
        state.handle_key(press(KeyCode::Char('r')));
        assert_eq!(state.selection, 0);
        for (i, s) in state.sessions.iter().enumerate() {
            assert!(!s.decrypted(), "session {i} cleared");
            assert_eq!(s.revealed, 0);
        }
    }

    #[test]
    fn decrypt_renders_reveal_and_completed_shows_plaintext() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        state.tick();
        state.tick();
        let text = rows_rendered(&state).join("\n");
        assert!(text.contains("decrypted"), "status shown");
        assert!(text.contains("revealed"), "reveal counter shown");

        for _ in 0..100 {
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        assert!(
            text.contains("Recovered — key 6b3f"),
            "recovered key hex shown"
        );
        assert!(text.contains("POST /login"), "plaintext revealed");
    }

    #[test]
    fn recovered_plaintext_wraps_instead_of_truncating() {
        // Regression: the recovered pane once rendered the plaintext as one
        // unwrapped line, so anything past the pane width vanished. The
        // longest session's prose sits well past the first wrapped row.
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Down)); // static.example.com, longest
        state.handle_key(press(KeyCode::Enter));
        for _ in 0..100 {
            state.tick();
        }
        let text = rows_rendered(&state).join("\n");
        // "english" sits ~90 chars into the plaintext; the pane's first row
        // holds ~56 — it only renders if the paragraph wraps.
        assert!(text.contains("english"), "deep plaintext wraps into view");
        assert!(text.contains("vote"), "tail of the plaintext wraps too");
    }
}
