# Proposal

## Why

Every interaction so far is keyboard-only; the obvious gesture — click the tab you want, click the victim you want — is missing. Mouse support turns the four panels from presentations into direct-manipulation UIs: the tab bar becomes a real tab bar, list items become buttons, and the math chart becomes a scroll-through carousel. The plumbing also forces the hit-testing architecture a growing TUI needs eventually (render records what is clickable), and the MCP harness can expose `send_click` so click behavior is verifiable in CI like everything else.

## What Changes

- Enable crossterm mouse capture on startup and — critically — disable it on every exit path, including panics (ratatui's own restore does not know about mouse capture; a crash must not leave the user's terminal with capture stuck on).
- Left-click translation: a new `AppEvent::MouseClick { row, column }` joins key events; the shell consumes clicks on the tab bar (click a tab = switch to that mode) and routes all other clicks to the active mode, mirroring key routing.
- Hit-testing architecture: rendering becomes the source of truth for what is clickable — each mode's `render` records its clickable rects (list rows, chart halves), and the click handler consults the areas recorded by the last frame. Inactive modes' stale areas are never consulted.
- Click semantics per mode:
  - **Autocrack**: clicking a victim selects it and starts the attack (one gesture = select + Enter).
  - **Traffic Simulator**: clicking a session selects it and runs the decrypt.
  - **Lessons**: clicking a lesson opens it at step one (while a lesson is open, clicks are inert; Esc still returns).
  - **Math Views**: clicking the left half of the chart area cycles to the previous sample, the right half to the next.
- Footer hints gain "· click works too" wording; the README mentions mouse support.
- The MCP harness gains a `send_click { row, column }` tool routed through the same dispatch, so click scenarios are testable end to end.

Not included: scroll-wheel handling (natural follow-up: wheel in lessons to step, wheel in lists to move selection), drag, hover highlighting, click-to-type in any future input fields.

## Capabilities

### New Capabilities

(none — behavior is specified through the existing modes and shell)

### Modified Capabilities

- `tui-shell`: terminal lifecycle additionally covers mouse capture on every exit path; new requirement for tab-bar click routing and click-first dispatch.
- `autocrack`: new requirement — clicking a victim selects and starts the attack.
- `traffic-simulator`: new requirement — clicking a session selects and decrypts it.
- `lessons`: new requirement — clicking a lesson opens it.
- `math-views`: new requirement — chart-half clicks cycle samples.
- `mcp-harness`: tool listing grows to five (`send_click` added); new requirement for its click semantics.

## Impact

- **Code**: `event.rs` (mouse event translation), `main.rs` (capture lifecycle + restore wrapper), `app.rs` (click dispatch), `ui.rs` (tab hit areas, `&mut App` render), all four modes (`render(&mut self, …)` recording hit areas + `click` handlers), `mcp/dispatch.rs` (`send_click`).
- **Dependencies**: none new.
- **Testing**: unit tests for rect hit-testing and each mode's click handler; TestBackend render-then-click tests (click inside a recorded victim row starts the attack); harness integration test clicks a tab and a victim; a PTY session sends a real SGR mouse escape sequence.
- **Docs**: README one-liner; footer hint strings.
