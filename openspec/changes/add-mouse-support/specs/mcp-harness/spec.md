# Spec Delta

## MODIFIED Requirements

### Requirement: MCP handshake and tool listing
The harness SHALL implement `initialize` (returning a protocol version and tools capability), accept `notifications/initialized`, and answer `tools/list` with the five tools (`send_key`, `send_click`, `tick`, `snapshot`, `reset`) including their input schemas.

#### Scenario: Initialize responds with tools capability
- **WHEN** a client sends a valid `initialize` request
- **THEN** the response advertises tool support and a protocol version

#### Scenario: tools/list enumerates the harness tools
- **WHEN** a client sends `tools/list` after initialization
- **THEN** the response lists `send_key`, `send_click`, `tick`, `snapshot`, and `reset`

## ADDED Requirements

### Requirement: send_click drives the real mouse dispatch
`send_click` SHALL accept `row` and `column` integers and route the click through the same shell-then-mode dispatch real mouse events use: tab-bar clicks switch modes, other clicks reach the active mode; out-of-range coordinates SHALL be a JSON-RPC error.

#### Scenario: A harness click switches modes
- **WHEN** a client calls `send_click` with the coordinates of a tab
- **THEN** the active mode switches exactly as a real tab click would

#### Scenario: Out-of-range coordinates are an error
- **WHEN** a client calls `send_click` with `row` 999
- **THEN** the harness returns a JSON-RPC error and the app state is unchanged
