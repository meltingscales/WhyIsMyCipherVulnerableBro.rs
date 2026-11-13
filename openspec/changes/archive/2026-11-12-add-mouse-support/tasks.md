# Tasks

## 1. Capture lifecycle and event translation

- [x] 1.1 Wire mouse capture into `main.rs`/`event.rs`: enable `EnableMouseCapture` after `ratatui::init()`, a `restore_terminal()` helper (`DisableMouseCapture` + `ratatui::restore`) on the clean path, and a chained panic hook that disables capture before delegating; translate only left-press mouse events into `AppEvent::MouseClick { column, row }` in `next_app_event`; verify with unit tests on the translation (press/move/release/wheel) and a PTY run confirming capture codes are emitted on start and undone on quit and on panic (inject-then-remove, as with the terminal-restore verification)

## 2. Hit-testing plumbing

- [x] 2.1 Change mode renders to `render(&mut self, frame, area)` recording `self.hit: Vec<(Rect, Hit)>` per mode (victim/session/lesson rows, chart halves), record tab rects on `App` in `ui::render`, and add `App::mouse_click(Position) -> &'static str` routing tab-bar clicks to mode switches and everything else to the active mode's `click(pos)`; update the main loop and harness snapshot to render with `&mut App`; verify with unit tests: a TestBackend render populates hit areas, tab clicks switch modes, body clicks reach only the active mode (inactive mode's click handler untouched)

## 3. Mode click handlers

- [x] 3.1 Implement and test Autocrack's click (victim row → select + start; inert while running) — verify with render-then-click unit tests against recorded row rects
- [x] 3.2 Implement and test Traffic Simulator's click (session row → select + decrypt; inert outside the list) — same verification pattern
- [x] 3.3 Implement and test Lessons' click (lesson row → open at step one; inert while open) — same verification pattern
- [x] 3.4 Implement and test Math Views' click (left/right halves of the chart area → previous/next sample, in every view) — same verification pattern

## 4. Harness and docs

- [x] 4.1 Add `send_click { row, column }` to `src/mcp/dispatch.rs` (bounds-checked against `rows`/`cols`, routed through `App::mouse_click`, routing string in the result) and update the tools listing to five; extend the integration test: click the Lessons tab via `send_click` (mode switches), then click a victim after reset (attack starts), plus the out-of-range error case; verify with `cargo test`
- [x] 4.2 Update the footer hint to mention the mouse and add a README line under Getting started; verify the footer renders in the existing footer tests

## 5. Verification

- [x] 5.1 Full pass: `cargo test`, `just lint`, `cargo fmt --check` clean; PTY session sending real SGR mouse sequences (tab click → mode switches; victim click → attack starts) via `script`, verified with the screen emulator; every `tui-shell`/mode `mcp-harness` mouse scenario re-checked through `--mcp`

## Workflow follow-up

- Archive the change after implementation is verified.
- Natural follow-up: scroll-wheel support (lesson stepping, list selection).
