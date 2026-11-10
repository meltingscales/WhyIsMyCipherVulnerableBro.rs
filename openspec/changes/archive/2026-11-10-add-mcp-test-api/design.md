# Design

## Context

See `proposal.md` — Why. The app's loop is already cleanly split: `event::next_app_event` produces `AppEvent`s, `App` applies them (`mode_key`/`tick`/global actions), `ui::render` draws to any ratatui backend. The harness reuses all three; only the terminal I/O layer is replaced.

## Goals / Non-Goals

**Goals:**

- Deterministic end-to-end driving of the real app: same dispatch, same rendering, zero terminal.
- MCP-compatible enough that standard MCP clients work unchanged (initialize/tools handshake, JSON-RPC 2.0, stdio newline framing).
- The harness becomes the scenario-verification path for future changes.

**Non-Goals:**

- MCP resources, prompts, sampling, notifications beyond `notifications/initialized`; other MCP versions' optional fields.
- Remote transports (TCP/SSE) — stdio only.
- Configurable tick interval (ticks are counted, not timed) or input replay macros (compose them from `send_key` calls).

## Decisions

**1. `--mcp` branch in `main.rs`, single binary.**
`main` parses argv before `ratatui::init()`; `--mcp` runs the harness and returns, anything else runs the TUI (unknown flags exit 2 with usage on stderr). Alternative: a second bin target — splits the artifact for no benefit; the harness shares everything except terminal setup.

**2. Transport: line-delimited JSON-RPC 2.0 on stdio.**
Read stdin line-by-line, parse each line as one JSON-RPC message, write one response line per request. This is the MCP stdio framing. Empty lines and unparseable lines get a JSON-RPC parse error response (id null). Notifications (no `id`) get no response. Batch arrays: reject with invalid-request — MCP does not use batches.

**3. New deps: `serde` + `serde_json` (derive).**
The first non-TUI dependency. Justification: MCP clients emit arbitrary JSON; a parser bug in hand-rolled JSON becomes a harness security/stability hole. Rejected hand-rolled parsing for exactly that reason.

**4. Dispatch as a pure function over JSON.**
`dispatch(state: &mut HarnessState, request: Value) -> Option<Value>` (None for notifications) — the whole protocol layer is pure JSON↔JSON on top of `HarnessState { app: App, rows, cols }`. The stdio loop only does I/O. This makes the protocol unit-testable without processes.

**5. Key names as a fixed vocabulary.**
`key_event(name) -> Option<KeyEvent>` covers exactly the keys the app maps (spec list). The harness applies keys via the same path the live loop uses: global keys through `key_to_action`'s actions (`next/prev/select/quit` on App), everything else via `App::mode_key` — mirroring `AppEvent::Global`/`ModeKey` handling in `run()`. `quit` sets `should_quit`; the harness reports `"quit": true` in tool results and keeps serving (a closed harness helps nobody mid-session; `reset` clears the flag).
- Default snapshot size 100×30 (matches the PTY sessions used so far).

**6. Snapshot via TestBackend.**
`snapshot` renders `ui::render` into `ratatui::backend::TestBackend` sized `rows×cols`, then flattens the buffer: one string per row, trailing spaces trimmed. The buffer object guarantees no ANSI, no diff artifacts — the exact class of bug that motivated this change.

**7. Result shape: MCP content blocks.**
Tool results use MCP's `{content: [{type: "text", text}], isError}`; snapshots embed the screen rows joined by newlines in `text`. Errors from tool logic are tool results with `isError: true`; protocol-level problems (unknown method, bad params) are JSON-RPC error responses. Unit tests pin both paths.

## Risks / Trade-offs

- [MCP spec drift (protocol versions, optional fields)] → Implement the small stable core (initialize/tools); tolerate unknown methods with standard `-32601`. Full SDK adoption can come later without changing the tool semantics.
- [serde_json pulls in a transitive tree] — Accepted; it is the ecosystem-standard JSON stack and widely audited.
- [Harness and live loop drift (keys handled differently)] → Mitigated structurally: both consume the same `AppEvent`-equivalent dispatch; a unit test asserts `send_key` routes identically to the live loop's application of `AppEvent`.
- [stdout pollution from panics/lib prints] → `--mcp` mode is headless (no TUI crates print); a panic hook in harness mode prints to stderr and exits non-zero.

## Migration Plan

Additive, behind a new flag — no existing behavior changes; rollback is `git revert`. CI can adopt the harness for scenario checks incrementally (the PTY emulator stays for smoke-testing the interactive path itself).
