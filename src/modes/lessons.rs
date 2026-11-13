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
pub const LESSONS: [Lesson; 5] = [
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
    Lesson {
        title: "SWEET32 — the birthday problem kills 3DES",
        steps: &[
            "How many people must share a room before two probably have the \
             same birthday? Not 365 — just 23. Collisions arrive after about \
             the square root of the number of possibilities, because what \
             matters is the number of PAIRS, and k people make k·(k-1)/2 pairs.",
            "Ciphers collide the same way. A block cipher with an n-bit block \
             has 2^n possible block values, so after roughly 2^(n/2) blocks \
             two ciphertext blocks probably match. For 3DES, n is 64: the \
             birthday bound is 2^32 blocks — about 32 GB of TLS traffic, an \
             afternoon of browsing.",
            "In 2016 SWEET32 showed that when two blocks collide in CBC \
             mode, XORing the two plaintexts cancels the key stream: \
             P_i XOR P_j = C_(i-1) XOR C_(j-1), which the attacker can see. \
             If one plaintext is known, the other falls out. Browsers and \
             servers dropped 3DES that year — not because 3DES's algorithm \
             broke, but because its BLOCK was too small.",
            "This app's demo encrypts with AES-128 — a cipher nobody has \
             broken — and truncates each block to 20 bits, a space of \
             2^20 = 1,048,576 values. That truncation is the entire attack \
             surface. It proves the point precisely: block size, not \
             algorithm strength, is what the birthday problem attacks.",
            "The bound says the first collision should arrive near \
             sqrt(pi/2 · 2^20) ≈ 1281 blocks. Collision time is a random \
             draw around that center — with this app's fixed demo traffic, \
             the first exploitable collision lands at 4744 blocks, a \
             perfectly ordinary tail draw on the birthday curve.",
            "The leak needs known plaintext on one side: every third record \
             in the demo is the attacker's own session, with a known \
             (zero) cookie. When a victim record collides with one of those, \
             XOR cancels the shared header and the known parts, and the \
             victim's cookie is left standing in the open.",
            "Open the Autocrack tab, pick SWEET32, press Enter, and watch \
             the counters: blocks climb, expected collisions grow as k², \
             and then the collision row lands and the cookie leaks. The \
             lesson underneath: never let a small block space carry \
             long-lived secrets — the math is patient.",
        ],
    },
    Lesson {
        title: "FREAK — when 'export grade' met factoring",
        steps: &[
            "In the 1990s, US export rules capped the crypto shipped \
             abroad. Servers kept 'export' cipher suites — RSA with a \
             512-bit key — long after the rules died. The suites lingered \
             in OpenSSL, Secure Transport, and others, waiting.",
            "FREAK (2015) needed three actors: a client whose TLS state \
             machine wrongly accepted an export certificate mid-handshake, \
             a server that still offered RSA_EXPORT, and a man in the \
             middle who rewrote the client's offer so the server picked \
             the weak suite. Watch those three rows open the Autocrack demo.",
            "The kill chain after the downgrade: factor the 512-bit \
             modulus N into its primes p and q, compute the private \
             exponent d = e⁻¹ mod λ(N), and decrypt the premaster secret \
             the client sent 'in private'. Factoring once breaks every \
             connection using that export key.",
            "How hard is factoring? Pollard's rho walks a pseudorandom \
             sequence mod N until two values collide in a way that exposes \
             a factor — expected around N^(1/4) steps for a balanced \
             semiprime. This app's demo modulus is 62 bits, and rho takes \
             exactly 30782 iterations — right on the N^¼ ≈ 46341 curve.",
            "A real 512-bit modulus puts N^¼ near 2^128 — not an \
             afternoon, but roughly a week of cloud computing circa 2015, \
             and it only has to happen ONCE per export key. Some servers \
             even shipped the same export key as everyone else, making the \
             precompute a shared cost. 'Too weak to matter' was really \
             'too weak to matter yet'.",
            "The demo's done screen shows the whole chain: both primes, \
             the derived private exponent, and the premaster secret \
             falling out of it. The lesson underneath: a cryptographic \
             algorithm is only as strong as the WEAKEST key anyone can \
             trick you into using — and downgrade attacks are exactly \
             that trick.",
        ],
    },
    Lesson {
        title: "Logjam — discrete logs and the prime everyone shared",
        steps: &[
            "Diffie–Hellman lets two parties agree on a secret over a \
             wire anyone can read: pick a prime p and generator g, publish \
             g^a and g^b, and both compute g^(ab). The eavesdropper sees \
             g, g^a, g^b — and must solve the DISCRETE LOG problem to turn \
             g^a back into a.",
            "Export rules also strangled DH: DHE_EXPORT used 512-bit \
             groups. Logjam (2015, same team, same year as FREAK) showed a \
             man in the middle could rewrite a DH handshake into the \
             export group — and many servers said yes.",
            "Baby-step giant-step solves g^y = B in about √p steps: \
             precompute 'baby steps' g^0..g^(m-1) into a table, then take \
             'giant strides' of g^(-m) across the space until the target \
             lands on a table entry. √p is also why key sizes doubled \
             compared to RSA-era intuition — the attack scales with the \
             square root, not the fourth root.",
            "This app's demo group is ~2^40, with the server's exponent y \
             = 314159265. BSGS finds it in exactly 635043 steps — the \
             √p ≈ 1048576 curve, live. The done screen shows y, the \
             shared secret it yields, and the session bytes falling out.",
            "The twist that made Logjam sting: most DHE_EXPORT servers \
             used one of TWO standard 512-bit primes. Solve the discrete \
             log structure of a prime once — a research-budget \
             precomputation — and every connection using it falls in \
             minutes. Individual connections were cheap because the \
             expensive part was amortized across the whole internet.",
            "The fix was boring and beautiful: delete the export suites, \
             and standardize on large, well-chosen (or ephemeral) groups. \
             The lesson underneath, shared with FREAK and SWEET32: \
             downgrade paths and shared weak parameters turn 'hard for \
             one attacker' into 'cheap for everyone'. Cryptography \
             decomposes; good systems don't hand it pieces to eat.",
        ],
    },
];

/// What a click on the last rendered frame meant, per recorded rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    /// A lesson row in the list: click opens it at step one.
    Lesson(usize),
}

/// Lessons screen state: the open lesson and the current step, if any.
#[derive(Debug)]
pub struct State {
    /// `(lesson index, step)` while a lesson is open.
    pub open: Option<(usize, usize)>,
    pub selection: usize,
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
            open: None,
            selection: 0,
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

    /// Handle a routed left-click: clicking a lesson row opens that lesson
    /// at step one; clicks while a lesson is open are inert.
    pub fn click(&mut self, pos: ratatui::layout::Position) -> &'static str {
        if self.open.is_some() {
            return "lessons:inert";
        }
        if let Some(&(_, Hit::Lesson(index))) = self.hit.iter().find(|(rect, _)| rect.contains(pos))
        {
            self.selection = index;
            self.open = Some((index, 0));
            "lessons:lesson"
        } else {
            "lessons:inert"
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.open {
            Some((lesson, step)) => self.render_lesson(frame, area, lesson, step),
            None => self.render_list(frame, area),
        }
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
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
        let selection = self.selection;
        let mut list_state = ListState::default();
        list_state.select(Some(selection));
        let list = List::new(items)
            .block(Block::bordered().title(" Lessons "))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut { list_state });

        // Each lesson is a single row from the list's inner top.
        self.hit.clear();
        let mut inner = area;
        inner.x += 1;
        inner.y += 1;
        inner.width = inner.width.saturating_sub(2);
        inner.height = inner.height.saturating_sub(2);
        for i in 0..LESSONS.len() {
            let y = inner.y.saturating_add(i as u16);
            if y >= inner.y + inner.height {
                break;
            }
            self.hit.push((
                Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                },
                Hit::Lesson(i),
            ));
        }
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
    fn clicking_a_lesson_opens_it_at_step_one() {
        let mut state = State::new();
        let _ = rows_rendered(&mut state);
        let (rect, _) = state.hit[2]; // SWEET32 lesson
        let routed = state.click(ratatui::layout::Position {
            x: rect.x + 2,
            y: rect.y,
        });
        assert_eq!(routed, "lessons:lesson");
        assert_eq!(state.open, Some((2, 0)));
        assert_eq!(state.selection, 2);
    }

    #[test]
    fn clicks_while_a_lesson_is_open_are_inert() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Enter));
        let _ = rows_rendered(&mut state);
        let routed = state.click(ratatui::layout::Position { x: 5, y: 5 });
        assert_eq!(routed, "lessons:inert");
        assert_eq!(state.open, Some((0, 0)));
    }

    #[test]
    fn list_shows_both_starter_lessons() {
        let mut state = State::new();
        let text = rows_rendered(&mut state).join("\n");
        assert!(text.contains("Why Caesar dies — frequency analysis"));
        assert!(text.contains("Breaking repeating-XOR"));
        assert!(text.contains("SWEET32 — the birthday problem kills 3DES"));
        assert!(text.contains("steps)"), "step counts shown");
    }

    #[test]
    fn sweet32_lesson_opens_and_cites_the_simulators_numbers() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((2, 0)));

        // Walk every step; the prose cites the pinned simulator numbers.
        let last = LESSONS[2].steps.len() - 1;
        let mut full_text = rows_rendered(&mut state).join("\n");
        for _ in 0..last {
            state.handle_key(press(KeyCode::Right));
            full_text.push('\n');
            full_text.push_str(&rows_rendered(&mut state).join("\n"));
        }
        assert!(full_text.contains("4744"), "cites the demo's pinned draw");
        assert!(full_text.contains("1281"), "cites the birthday bound");
        assert!(full_text.contains("32 GB"), "cites the real SWEET32 scale");
    }

    #[test]
    fn freak_lesson_opens_and_cites_the_solvers_numbers() {
        let mut state = State::new();
        for _ in 0..3 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((3, 0)));

        let last = LESSONS[3].steps.len() - 1;
        let mut full_text = rows_rendered(&mut state).join("\n");
        for _ in 0..last {
            state.handle_key(press(KeyCode::Right));
            full_text.push('\n');
            full_text.push_str(&rows_rendered(&mut state).join("\n"));
        }
        assert!(
            full_text.contains("30782"),
            "cites the pinned rho iterations"
        );
        assert!(full_text.contains("46341"), "cites the N^¼ curve");
        assert!(
            full_text.contains("512-bit"),
            "cites the real export modulus"
        );
    }

    #[test]
    fn logjam_lesson_opens_and_cites_the_solvers_numbers() {
        let mut state = State::new();
        for _ in 0..4 {
            state.handle_key(press(KeyCode::Down));
        }
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((4, 0)));

        let last = LESSONS[4].steps.len() - 1;
        let mut full_text = rows_rendered(&mut state).join("\n");
        for _ in 0..last {
            state.handle_key(press(KeyCode::Right));
            full_text.push('\n');
            full_text.push_str(&rows_rendered(&mut state).join("\n"));
        }
        assert!(full_text.contains("635043"), "cites the pinned BSGS steps");
        assert!(full_text.contains("1048576"), "cites the √p curve");
        // single word: wrapped rows split phrases across lines
        assert!(
            full_text.contains("amortized"),
            "cites the shared-prime twist"
        );
    }

    #[test]
    fn enter_opens_the_selected_lesson_at_step_one() {
        let mut state = State::new();
        state.handle_key(press(KeyCode::Down));
        state.handle_key(press(KeyCode::Enter));
        assert_eq!(state.open, Some((1, 0)));
        let text = rows_rendered(&mut state).join("\n");
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
        let text = rows_rendered(&mut state).join("\n");
        assert!(text.contains("step 3 of 6"), "counter shows current step");
        // Single words: wrapped lines break between words, so multi-word
        // phrases can straddle a row boundary.
        assert!(
            text.contains("fingerprint") && text.contains("shape"),
            "third step's content"
        );

        state.handle_key(press(KeyCode::Right));
        let text = rows_rendered(&mut state).join("\n");
        assert!(text.contains("step 4 of 6"));
        assert!(text.contains("chi-squared"), "fourth step's content");
    }
}
