# Design

## Context

See `proposal.md` — Why. `event::next_app_event` currently ignores all non-key events; `ui::render` and every mode `render` take `&self`/`&App`; ratatui 0.30's `init`/`restore` know nothing about mouse capture (verified: no mouse handling in `init.rs`), so the capture lifecycle is ours, including the panic path.

## Goals / Non-Goals

**Goals:**

- Left-click everywhere it makes sense, with the same behavior from the MCP harness.
- Click behavior defined by what the last frame actually drew — no duplicated layout math.
- Mouse capture never leaks past process exit, panic included.

**Non-Goals:**

- Scroll wheel, drag, hover, right/middle click, modifiers; configurable bindings.
- Click-driven stepping inside lessons (keys stay) — wheel is the natural follow-up.

## Decisions

**1. Immediate-mode hit testing: render records, click consults.**
Each mode's `render` becomes `render(&mut self, frame, area)` and clears+fills `self.hit: Vec<(Rect, Hit)>`, where `Hit` is a small enum of that mode's click intents (e.g. `Victim(usize)`, `Session(usize)`, `Lesson(usize)`). `ui::render` likewise records the four tab rects on `App`. `App::mouse_click(pos)` first hit-tests the tab rects (global concern — tab bar is shell chrome), then forwards to the active mode's `click(pos)` which walks its own `hit` list. Inactive modes' areas go stale but are never read (only the active mode renders and receives clicks). Alternative rejected: recomputing rects in click handlers — duplicates layout and drifts.

**2. Rect membership via ratatui's `Rect::contains(Position)`.**
No custom geometry. Rows/columns arrive as `u16` from crossterm; crossterm's (column, row) maps directly to ratatui's (x, y).

**3. Click semantics = select + activate, consistently.**
One click does what select + Enter does on the keyboard: victim click starts the attack, session click decrypts, lesson click opens. While a run/lesson is open, clicks are inert (spec-pinned). Math views halves cycle samples. This keeps one mental model across modes.

**4. Capture lifecycle: enable after init, disable before restore, panic hook chains.**
`run_tui()`: `ratatui::init()` → `execute!(stdout, EnableMouseCapture)` → loop → `restore_terminal()` = `DisableMouseCapture` + `ratatui::restore()`. For panics: ratatui's hook calls its restore (raw mode + alt screen) but not mouse disable, so we install our own hook after `init()` that sends `DisableMouseCapture` then delegates to the previous (ratatui's) hook — capture dies before the message prints. The `--mcp` path never enables capture (headless, untouched).

**5. Event translation: only left-press; everything else ignored.**
`next_app_event` maps `Event::Mouse(MouseEvent { kind: MouseButton(Left) Press, column, row, .. })` → `AppEvent::MouseClick { column, row }`; moves, releases, drag, wheel, other buttons → `None`. No double-click state machine.

**6. Harness: `send_click { row, column }` reuses `App::mouse_click`.**
Bounds: `row < state.rows && column < state.cols`, else JSON-RPC error with state unchanged (spec). The tool result reports what the click hit (tab/mode/inert) by having `mouse_click` return a small routing string, like `send_key`.

**7. Footer wording.**
Global footer gains "· mouse works too" — the spec's footer requirement lists keybindings; this keeps it truthful without enumerating click zones.

## Risks / Trade-offs

- [`render(&mut self)` refactor touches every mode and both render call sites (main loop, harness snapshot)] → Mechanical; TestBackend tests catch signature fallout immediately.
- [Terminal emulators vary in SGR mouse encoding] → We only consume what crossterm decodes; PTY verification in the final pass sends a real SGR sequence (`ESC [ < 0 ; col ; row M`) to prove the end-to-end path.
- [Stale hit areas after resize] → Areas refresh every frame (render runs each loop iteration); a click between resize and next draw reads one stale frame at worst — acceptable.
- [Panics before hook installation] → The window between process start and hook install does not have capture enabled yet, so nothing leaks.

## Migration Plan

Additive interaction layer; no keyboard behavior changes (specs untouched except additive rows). One commit series; rollback is `git revert`.
