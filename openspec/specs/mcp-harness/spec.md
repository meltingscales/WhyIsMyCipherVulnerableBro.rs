# mcp-harness Specification

## Purpose

A headless MCP-compatible harness for the TUI: a stdio JSON-RPC server mode that lets agents and scripts send keys, tick the animation loop, snapshot the rendered screen, and reset state — driving the same app the interactive terminal runs, without a terminal.

## Requirements

### Requirement: Headless harness mode
The binary SHALL support a `--mcp` flag that runs a JSON-RPC 2.0 server over stdio (newline-delimited) instead of the interactive TUI, and in that mode SHALL NOT change terminal state (no raw mode, no alternate screen), SHALL keep stdout protocol-clean, and SHALL route diagnostics to stderr only.

#### Scenario: Flag starts the harness without touching the terminal
- **WHEN** the binary is started with `--mcp`
- **THEN** it reads JSON-RPC requests from stdin and writes responses to stdout without altering the caller's terminal settings

#### Scenario: stdout stays protocol-clean
- **WHEN** the harness logs or errors while serving
- **THEN** nothing but JSON-RPC messages is ever written to stdout

### Requirement: MCP handshake and tool listing
The harness SHALL implement `initialize` (returning a protocol version and tools capability), accept `notifications/initialized`, and answer `tools/list` with the four tools (`send_key`, `tick`, `snapshot`, `reset`) including their input schemas.

#### Scenario: Initialize responds with tools capability
- **WHEN** a client sends a valid `initialize` request
- **THEN** the response advertises tool support and a protocol version

#### Scenario: tools/list enumerates the harness tools
- **WHEN** a client sends `tools/list` after initialization
- **THEN** the response lists `send_key`, `tick`, `snapshot`, and `reset`

### Requirement: send_key drives the real dispatch
`send_key` SHALL accept a named key (`enter`, `tab`, `backtab`, `up`, `down`, `left`, `right`, `esc`, `q`, `r`, `n`, `p`, `1`-`4`, `ctrl-c`) and route it through the same global-then-mode key dispatch the interactive loop uses, so global keys act globally and mode keys reach the active mode; unknown names SHALL return a JSON-RPC error.

#### Scenario: A mode key changes visible state
- **WHEN** a client calls `send_key` with `enter` while Autocrack is active on the victim list
- **THEN** the app starts the attack, visible in the next `snapshot`

#### Scenario: A global key still switches modes
- **WHEN** a client calls `send_key` with `tab`
- **THEN** the active mode advances exactly as it does in the interactive TUI

#### Scenario: Unknown key name is an error
- **WHEN** a client calls `send_key` with an unrecognized key name
- **THEN** the harness returns a JSON-RPC error and the app state is unchanged

### Requirement: tick advances the active mode
`tick` SHALL advance the active mode's animation by `n` ticks (default 1), each tick equivalent to one animation heartbeat of the interactive loop; `n` below 1 or above a sane cap SHALL be an error.

#### Scenario: Ticks drive an attack to completion
- **WHEN** a client starts a Caesar attack and calls `tick` with `n` 26 or more
- **THEN** the attack completes and the done screen is visible in the next `snapshot`

#### Scenario: Out-of-range n is an error
- **WHEN** a client calls `tick` with `n` 0 or 10 000
- **THEN** the harness returns a JSON-RPC error and no ticks are applied

### Requirement: snapshot returns the rendered screen
`snapshot` SHALL render the current app state to its internal backend at the configured size and return the screen as text rows, preserving row structure, with trailing blank cells trimmed per row.

#### Scenario: Snapshot reflects current state
- **WHEN** the app is on the Autocrack victim list
- **THEN** the snapshot rows contain the victim list block title and both victims' names

### Requirement: reset restores initial state
`reset` SHALL replace the app state with a fresh instance, optionally changing the snapshot size (`rows`, `cols`, each within 20-500), and defaults SHALL apply for omitted arguments.

#### Scenario: Reset clears in-progress state
- **WHEN** an attack is running and a client calls `reset`
- **THEN** the next snapshot shows the initial victim list with no attack in progress

#### Scenario: Out-of-range size is an error
- **WHEN** a client calls `reset` with `rows` 5
- **THEN** the harness returns a JSON-RPC error and the previous state and size are kept
