# Proposal

## Why

End-to-end verification of the TUI currently means a PTY plus an ANSI screen emulator — workable but brittle (ratatui's cell-diff skips unchanged cells, so raw byte-stream assertions are unsound, as task 7.1 of `fill-mode-panels` demonstrated). An MCP-compatible harness mode gives agents, IDEs, and scripts a deterministic, headless way to drive the real app: send keys, tick the loop, and read the rendered screen as plain text — no terminal, no escape codes, no timing.

## What Changes

- Add a `--mcp` flag to the binary: a headless harness speaking JSON-RPC 2.0 over stdio (MCP stdio transport) — it never enters raw mode or the alternate screen, so it is safe to run from any process, including CI.
- Implement the MCP handshake (`initialize`, `notifications/initialized`, `tools/list`) and four tools:
  - `send_key {key}` — route a named key (enter/tab/backtab/up/down/left/right/esc/q/r/n/p/1-4/ctrl-c) through the app's real global-then-mode dispatch;
  - `tick {n}` — advance the active mode n ticks (n ≥ 1, capped);
  - `snapshot {}` — return the rendered screen as text rows at the configured size;
  - `reset {rows?, cols?}` — fresh app state, optionally resized.
- Keys route through the same `App` dispatch as the live loop, so harness behavior is identical to interactive behavior by construction.
- New dependency: `serde` + `serde_json` — the first non-TUI dependency; hand-rolled JSON parsing of client input is not worth the risk.

Not included: MCP resources/prompts/sampling (tools suffice for driving the app), remote transports (stdio only), persistence between sessions.

## Capabilities

### New Capabilities

- `mcp-harness`: The headless test harness — stdio JSON-RPC server mode exposing key injection, ticking, screen snapshots, and reset over MCP tools, driving the same app the interactive TUI runs.

### Modified Capabilities

(none — the interactive shell and mode specs are untouched; the harness reuses them unchanged)

## Impact

- **Code**: new `src/mcp/` module (transport, dispatch, tool handlers); `main.rs` gains the `--mcp` branch before terminal init; `Cargo.toml` gains serde/serde_json.
- **Dependencies**: `serde` (derive) + `serde_json`.
- **Testing**: the harness itself becomes the end-to-end verification path for future changes (replacing PTY+emulator for scenarios); unit tests cover dispatch JSON↔JSON, one integration test spawns the real binary and drives a full scenario.
- **Specs**: one new capability; no behavior of existing capabilities changes.
