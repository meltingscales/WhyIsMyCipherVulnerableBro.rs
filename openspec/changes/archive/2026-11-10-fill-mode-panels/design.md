# Design

## Context

See `proposal.md` — Why. The shell (`tui-shell`, archived `2026-11-10-add-core-tui-framework`) currently renders one placeholder function per mode in `ui.rs`; `App` holds only `mode` + `should_quit`; `event.rs` maps global keys only. Dependencies in `Cargo.toml` cover everything needed (`aes` stays unused until a real cipher demo needs it).

## Goals / Non-Goals

**Goals:**

- Four panels with working starter slices — interactive, tick-animated where visual, and honest about the math they show.
- A pure `cipher` core that later changes (SWEET32/FREAK/Logjam) extend rather than rewrite.
- Per-mode state that survives mode switches without cloning or rebuilding.

**Non-Goals:**

- User-supplied ciphertext input, editing, persistence (later changes).
- SWEET32/FREAK/Logjam (roadmap changes of their own).
- Scrolling long lesson bodies — steps are sized to fit; pagination can come later.
- Real randomness: all samples and keys are fixed constants so tests are deterministic.

## Decisions

**1. Per-mode modules with exhaustive match dispatch, not a `Screen` trait.**
Each mode gets a module in `src/modes/` owning `State`, `handle_key`, `tick`, and `render`. The shell routes via `match app.mode` — the compiler forces every mode to be handled everywhere (keys, ticks, render), and state lives in plain structs that are trivially unit-testable. The tui-shell design note deferred "the trait" to the first real mode; with four landing at once, exhaustive `match` beats `dyn Screen` (no object safety constraints, no vtable, dead-simple tests). Revising that note deliberately.

**2. Pure cipher core, one direction of dependency.**
`src/cipher/` = `caesar`, `vigenere`, `xor`, `freq` (chi-squared vs English, index of coincidence, English letter profile). No TUI types, no ratatui imports, no `App` references — modes import cipher, never the reverse. Every function pure over `&[u8]`/`&str`, unit-tested against fixed vectors (e.g. chi-squared of the English sample < chi-squared of any rotated version; known Caesar/Vigenère/XOR round-trips and recoveries). This is the cryptopals-style test spine.

**3. Global keys resolve first; screens see the rest.**
`key_to_action` keeps handling Tab/BackTab/`1`-`4`/`q`/Ctrl-C. Unmapped keys route to the active mode's `handle_key` (Enter, arrows, `r`, Escape, letters). Consequence: modes can never shadow global navigation — matches the shell spec ("quit key exits in any mode").

**4. Ticks drive the active mode only; state persists across switches.**
`Tick` dispatches to the active mode's `tick()`. Switching away simply stops ticking that mode — its state sits untouched (spec: "progress survives a mode switch"). Pacing: one attack step per tick (250 ms ⇒ Caesar's 26 candidates ≈ 6.5 s); Vigenère advances per-candidate across key-length × 26 columns by stepping one column-candidate per tick with an accelerated 2-per-tick stride so it finishes in a similar window. Step logic is a pure `advance(state) -> state` so tests never depend on wall-clock.

**5. Attack implementations (starter versions, real math).**
- Caesar: try all 26 shifts, score each decrypt by chi-squared vs English; best score wins. Streams as candidate rows.
- Vigenère: report the index-of-coincidence key-length scan as visible steps, then crack each key column by per-column chi-squared.
- Repeating-XOR (traffic sim): sessions are fake HTTP requests XOR'd with fixed 2–5 byte keys; the break is the cryptopals set-1 attack — per-column single-byte solve by scoring, revealed progressively per tick. Known-plaintext structure ("GET /", "Host:") is displayed as the justification, but the break itself only scores candidate bytes.

**6. Content as constants.**
English sample text, ciphertext samples (derived by running the cipher core over the sample — one source of truth), fake sessions, and both lessons live as `const`/`static` data in their mode modules. If lessons outgrow const structs, a later change moves them to files.

**7. Rendering reuse.**
`ui.rs` keeps the tab bar/footer chrome and dispatches bodies to mode `render` functions. Footer gains the active mode's key hints after the global bindings. `BarChart` (ratatui) drives Math Views — 26 bars fit an 80-col layout; the chart renders from the same `freq` module the attacks score with.

## Risks / Trade-offs

- [Cipher module creeps toward TUI concerns] → Dependency direction is one-way by construction; `just lint` plus review keeps it honest.
- [Animation pacing feels slow/fast] → Step stride is a named constant per attack; trivially tunable without touching logic.
- [26-bar chart cramped below ~60 cols] → Acceptable floor for a TUI; chart tests use TestBackend at 80×24, the documented minimum.
- [Const lesson content goes stale vs code] → Lessons reference the attacks by name and their steps cite the same statistics the code computes; the lessons test asserts both starter lessons exist rather than freeze-walking prose.
- [Removing the tui-shell placeholder requirement orphans its old test] → Task removes the placeholder render path and its TestBackend test with it; per-mode specs carry new coverage.

## Migration Plan

One commit series behind the change; each task group lands with its tests (`cargo test`, `just lint`). Rollback: `git revert` — no persistent state, no external systems.
