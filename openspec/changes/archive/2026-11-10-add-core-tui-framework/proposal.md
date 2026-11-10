# Proposal

## Why

The repo is a hello-world skeleton with all dependencies declared but no application. Every planned feature — SWEET32, FREAK, and Logjam demos, the traffic simulator, cryptopals-style lessons, math views — needs the same substrate to live in: a running ratatui event loop, navigation between mode screens, and safe terminal lifecycle. Building that shell first unblocks every later change instead of having each demo reinvent it. A justfile lands alongside it so the common dev loop (`just run`, `just check`) stops depending on remembering cargo incantations.

## What Changes

- Replace the hello-world `src/main.rs` with a ratatui/crossterm application loop that owns terminal setup and teardown.
- Navigate between placeholder screens for the four README modes — Autocrack, Traffic Simulator, Lessons, Math Views — via tab/shift-tab (and number keys 1-4), with the active mode highlighted in a persistent tab bar.
- Show a mode-specific help/footer bar with global keybindings (quit, next/prev tab).
- Restore the terminal on exit and on panic (hook) so a crash never leaves the user's shell broken.
- Add a `justfile` with recipes for the common dev commands (`run`, `build`, `check`, `fmt`, `lint`).
- Update the README quickstart to mention `just run` alongside `cargo run`.

No cipher logic, attack demos, or math visualizations are included — screens are placeholders that the future changes fill in.

## Capabilities

### New Capabilities

- `tui-shell`: The terminal application shell — event loop, terminal lifecycle (setup, teardown, panic-safe restore), navigation between mode screens, tab bar, and footer with keybindings. Owns what every future mode plugs into.

### Modified Capabilities

(none — no existing specs)

## Impact

- **Code**: `src/main.rs` rewritten; new modules under `src/` for app state, event handling, and UI rendering.
- **Dependencies**: none added — `ratatui`, `crossterm`, and `aes` are already declared in `Cargo.toml`.
- **Tooling**: new `justfile` (requires `just` ≥ 1.0; 1.58.0 verified on this machine).
- **Docs**: README quickstart gains `just run`.
