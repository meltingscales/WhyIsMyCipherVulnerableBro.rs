# Design

## Context

See `proposal.md` — Why. The repo today is a hello-world `main.rs` plus already-declared deps (`ratatui` 0.30, `crossterm` 0.29, `aes` 0.9); there is no application to extend. This design establishes the shell every future mode change builds on.

## Goals / Non-Goals

**Goals:**

- A running app loop that future mode changes extend without restructuring the loop or lifecycle code.
- Panic- and exit-safe terminal handling, so a crash never wrecks the user's shell.
- Placeholder rendering good enough to verify navigation behavior from the `tui-shell` spec.

**Non-Goals:**

- Any cipher/attack logic or real screen content (later changes).
- A `Screen` trait or plugin architecture — placeholder screens are plain render functions; the trait arrives with the first real mode.
- Theming, config, persistence, mouse support.

## Decisions

**1. Module layout — split state, events, rendering.**
`main.rs` stays a thin entry point; `app.rs` owns state (`Mode` enum, `ALL_MODES`, `next()/prev()`); `event.rs` maps crossterm events to app actions; `ui.rs` renders tab bar / body / footer. Rationale: every future mode touches state + rendering, not the loop. Alternative: one `main.rs` file — smaller today, but each demo would pile into it.

**2. Event loop — `poll()` with a tick, not blocking `read()`.**
A ~4 Hz tick event alongside key events. Rationale: autocrack visuals and the traffic simulator are the point of this app; they need a heartbeat to animate against. Blocking `read()` would be simpler but every animated demo would force a loop rewrite. Idle cost is negligible.

**3. Terminal lifecycle — ratatui's built-in helpers.**
Use `ratatui::init()` / `ratatui::restore()` (which set up the alternate screen, raw mode, and a panic hook that restores the terminal), plus an explicit restore on the normal-exit path. Rationale: hand-rolled crossterm setup + `std::panic::set_hook` duplicates what the crate maintains upstream. Alternative: manual setup with a custom hook — fallback if the 0.30 API differs from this write-up (verify against the locked version while implementing).

**4. Placeholder screens — functions, not objects.**
Each mode's body is a `render_*` function in `ui.rs` showing the mode name and description. Rationale: placeholders carry no state or behavior; a trait here would be speculative. The first real mode change introduces the abstraction with one real consumer to shape it.

**5. justfile — thin aliases, no logic.**
`run` (default), `build`, `check`, `fmt`, `lint` (clippy with `-D warnings`). Rationale: the recipes only exist to save typing; anything cleverer belongs in code or CI later.

## Risks / Trade-offs

- [ratatui 0.30 API drift vs. this design's assumptions] → Verify `init`/`restore` against the locked crate docs during implementation; the placeholder surface is small, so switching to manual crossterm setup is a contained change.
- [Tick loop wakes the CPU when idle] → Negligible at 4 Hz; acceptable for a TUI whose purpose is animated demos.
- [Panic hook cannot run under `panic = "abort"`] → Default dev/release profiles unwind; documented here rather than defended against.
- [Function-per-screen won't scale past placeholders] → Deliberate (Non-Goal); the first real mode change replaces it, and `app.rs`/`ui.rs` seams are where that lands.

## Migration Plan

Single commit replacing hello-world `main.rs` and adding modules + `justfile`. Rollback is `git revert` — nothing persistent, no external systems.
