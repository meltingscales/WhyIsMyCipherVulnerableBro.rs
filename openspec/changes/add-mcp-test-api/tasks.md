# Tasks

## 1. Protocol layer (pure JSON dispatch)

- [x] 1.1 Add `serde`/`serde_json` to `Cargo.toml`; verify `cargo build` succeeds with the new dependency tree
- [x] 1.2 Create `src/mcp/dispatch.rs` with `HarnessState { app, rows, cols }` and pure `dispatch(state, request) -> Option<Value>`: JSON-RPC 2.0 envelope (parse errors, invalid requests, unknown method `-32601`, notifications return None), `initialize`, `notifications/initialized`, `tools/list` with the four tools and input schemas; verify with unit tests covering each envelope path (valid request, unknown method, bad JSON params, notification)
- [x] 1.3 Implement `send_key`/`tick`/`snapshot`/`reset` tool handlers: key-name→KeyEvent vocabulary, routing through the same dispatch the live loop uses, tick range validation (1..=1000), snapshot via TestBackend flatten (rows joined, trailing blanks trimmed), reset with size validation (20..=500, defaults preserved); verify with unit tests per the `mcp-harness` scenarios (enter starts attack, tab advances mode, unknown key errors state-unchanged, tick 26 completes Caesar, tick 0/10000 errors, snapshot shows victim list, reset clears and validates)

## 2. Server wiring

- [x] 2.1 Create `src/mcp/server.rs` (newline-delimited stdio loop feeding `dispatch`, diagnostics to stderr only) and `src/mcp/mod.rs`; wire the `--mcp` branch into `main.rs` before terminal init, with unknown flags exiting 2 with usage on stderr; verify stdout stays protocol-clean by driving a scripted stdin and asserting every stdout line parses as JSON
- [x] 2.2 Add an integration test (`tests/mcp_harness.rs`) spawning the built binary via `CARGO_BIN_EXE`: initialize → tools/list → send_key enter → tick n=30 → snapshot, asserting the done screen (key and plaintext) appears in the snapshot text; verify with `cargo test`

## 3. Adoption and verification

- [x] 3.1 Re-verify one `fill-mode-panels` scenario through the harness (traffic decrypt reveal: send_key tab/enter, tick, snapshot contains `Recovered` and plaintext) to prove the harness replaces the PTY emulator for scenario checks; verify by running the integration test flow or a scripted `--mcp` session
- [x] 3.2 Full pass: `cargo test`, `just lint`, `cargo fmt --check` clean; scripted `--mcp` session exercising handshake, all four tools, both error paths (unknown key, bad tick), and a clean exit

## Workflow follow-up

- Archive the change after implementation is verified.
- Consider wiring the harness into CI as the scenario-check path for future changes (replacing PTY-based checks except for interactive smoke tests).
