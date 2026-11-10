# Tasks

## 1. Cipher core (pure, TUI-free)

- [x] 1.1 Create `src/cipher/freq.rs` — English letter profile, chi-squared distance, index of coincidence; verify with unit tests that the English sample scores lower chi-squared than any Caesar rotation of itself and that IoC distinguishes English from polyalphabetic text
- [x] 1.2 Create `src/cipher/caesar.rs` — encrypt/decrypt plus `crack` yielding all 26 shift candidates scored by chi-squared; verify with unit tests for round-trip and that the top-ranked candidate recovers the known key and plaintext
- [x] 1.3 Create `src/cipher/vigenere.rs` — encrypt/decrypt, key-length scan by index of coincidence, per-column chi-squared crack; verify with unit tests that the scan finds the true key length and the crack recovers the key for the built-in sample
- [x] 1.4 Create `src/cipher/xor.rs` — repeating-key XOR apply plus per-column single-byte crack; verify with unit tests for round-trip and that the crack recovers a fixed 3-byte key on a known sample

## 2. Screen architecture

- [x] 2.1 Restructure `App` to hold per-mode state in `src/modes/` modules (state + `handle_key` + `tick` + `render` per mode), route unmapped keys to the active mode after global keys, and dispatch `Tick` to the active mode; verify with unit tests that global keys (Tab/1-4/q/Ctrl-C) are never routed to a mode and that a mode key reaches only the active mode
- [x] 2.2 Replace the placeholder body render and its test with per-mode dispatch, and extend the footer with the active mode's key hints; verify with `TestBackend` tests that the footer shows mode hints per mode and that global chrome (tab bar highlight, global bindings) still renders on every mode

## 3. Autocrack panel

- [x] 3.1 Implement victim selection (Caesar/Vigenère with sample ciphertexts; arrows + Enter — bare `1`-`4` stay reserved for global mode switching per the `tui-shell` spec, which the mode spec's "arrow keys or number keys" permits) over `cipher::caesar`/`cipher::vigenere`; verify with unit tests for selection state and Enter starting the right attack
- [x] 3.2 Implement the tick-driven attack runner — one candidate step per tick via a pure `advance` function, streaming candidate rows with scores, best-so-far highlighted, done state showing key + plaintext, `r` reset; verify with unit tests that stepping `advance` to completion yields the cipher core's top candidate, and with `TestBackend` tests that selection, running, and done states render their required content

## 4. Traffic Simulator panel

- [x] 4.1 Implement the built-in session list (fixed hostnames, cipher labels, lengths; arrows/numbers) with hex dump of the selected session; verify with unit tests that selection moves and the hex view follows it
- [x] 4.2 Implement Enter-to-decrypt — repeating-XOR break via `cipher::xor`, plaintext revealed progressively per tick, `r` reset; verify with unit tests that a fully stepped reveal matches the crack result and reset clears all sessions, and with `TestBackend` tests that list, hex, and revealing states render

## 5. Lessons panel

- [x] 5.1 Author the two starter lessons as const content ("Why Caesar dies" — frequency analysis/chi-squared; "Breaking repeating-XOR") and implement the lesson list plus step reader (Enter opens at step 1, left/right or n/p navigate clamped, Escape/r returns, step counter shown); verify with unit tests for clamped navigation and return-to-list, and with `TestBackend` tests that list, open-lesson, and step states render with the counter

## 6. Math Views panel

- [x] 6.1 Implement the sample set (English plaintext plus Caesar/Vigenère ciphertexts derived by running the cipher core over it) with the letter-frequency `BarChart`, sample name, chi-squared and IoC readouts, and left/right cycling with wrap-around; verify with unit tests that cycling wraps both directions and that the plaintext sample has the smallest chi-squared of the set, and with `TestBackend` tests that the chart, sample name, and both statistics render

## 7. Integration verification

- [x] 7.1 Full pass: `cargo test`, `just lint`, `cargo fmt --check` all clean; then a PTY session per spec scenario — autocrack run to completion showing key/plaintext, traffic session decrypt reveal, lesson open/step/escape, math-views sample cycling, and a mid-attack mode switch proving progress survives — confirming behavior against each `autocrack`/`traffic-simulator`/`lessons`/`math-views` scenario and the footer's mode hints

## Workflow follow-up

- Archive the change after implementation is verified (spec sync will create the four new main specs and apply the tui-shell REMOVED delta).
- Verify the archived result.
