# Tasks

## 1. App state and modes

- [ ] 1.1 Create `src/app.rs` with the `Mode` enum (Autocrack, Traffic Simulator, Lessons, Math Views), `ALL_MODES`, and `next()`/`prev()`/`select(index)` state transitions; verify with unit tests covering forward, backward, and wrap-around at both ends plus direct selection (`cargo test`)
- [ ] 1.2 Verify the ratatui 0.30 lifecycle API (`init`/`restore`, panic-hook behavior) against the locked crate docs and record any deviation from `design.md` decision 3 before building on it

## 2. Event handling

- [ ] 2.1 Create `src/event.rs` mapping crossterm key events to app actions (`NextTab`, `PrevTab`, `Select(n)`, `Quit`) plus a ~4 Hz tick via `poll()`; verify with unit tests on the pure key→action mapping (Tab, Shift+Tab, `1`-`4`, `q`, Ctrl-C)

## 3. Terminal lifecycle and main loop

- [ ] 3.1 Rewrite `src/main.rs` as the thin entry point: `ratatui::init()`, loop driving `app` from `event`, restore on clean exit; verify `cargo run` then `q` returns to an intact shell (main screen, cooked mode) and exit code 0
- [ ] 3.2 Verify panic-safe restore from `tui-shell` spec: inject a panic after init in a throwaway local run, confirm the shell is usable afterward, remove the injection

## 4. UI rendering

- [ ] 4.1 Create `src/ui.rs` with tab bar (active mode highlighted), mode-specific placeholder bodies, and footer listing Tab/Shift+Tab/1-4/q/Ctrl-C; verify with `TestBackend` unit tests asserting each mode's name renders, the active tab is distinguished, and the footer text is present (`cargo test`)

## 5. Tooling and docs

- [ ] 5.1 Add `justfile` with `run` (default), `build`, `check`, `fmt`, `lint` (clippy `-D warnings`); verify each recipe executes successfully
- [ ] 5.2 Update the README quickstart to mention `just run` alongside `cargo run`; verify the documented commands work as written

## 6. Integration verification

- [ ] 6.1 Full pass against the `tui-shell` spec scenarios: `cargo test`, `just lint`, `just fmt` clean; run the app and exercise Tab, Shift+Tab (including wrap-around), `1`-`4`, `q`, and Ctrl-C, confirming behavior matches each scenario

## Workflow follow-up

- Archive the change after implementation is verified.
- Verify the archived result.
