//! The harness protocol layer: a pure JSON↔JSON dispatcher over
//! [`HarnessState`]. The stdio loop in `server.rs` only does I/O; every
//! protocol decision (envelope handling, method routing, tool semantics)
//! lives here so it is unit-testable without processes.

use serde_json::{Value, json};

use crate::app::App;
use crate::event::{Action, key_to_action};
use crate::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// MCP protocol version this harness speaks.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Tick bounds: at least one, at most a full slow-motion minute per call.
const TICK_MIN: i64 = 1;
const TICK_MAX: i64 = 1000;
/// Snapshot size bounds for `reset`.
const SIZE_MIN: u16 = 20;
const SIZE_MAX: u16 = 500;
/// Default snapshot geometry, matching the PTY sessions used so far.
const DEFAULT_ROWS: u16 = 30;
const DEFAULT_COLS: u16 = 100;

/// The headless app under test plus the snapshot geometry.
pub struct HarnessState {
    pub app: App,
    pub rows: u16,
    pub cols: u16,
}

impl Default for HarnessState {
    fn default() -> Self {
        Self {
            app: App::new(),
            rows: DEFAULT_ROWS,
            cols: DEFAULT_COLS,
        }
    }
}

impl HarnessState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one key press exactly the way the interactive loop does:
    /// global keys resolve to their shell action, everything else routes to
    /// the active mode. Returns how the key was routed.
    fn apply_key(&mut self, key: KeyEvent) -> &'static str {
        match key_to_action(key) {
            Some(Action::NextTab) => {
                self.app.next();
                "global:next_tab"
            }
            Some(Action::PrevTab) => {
                self.app.prev();
                "global:prev_tab"
            }
            Some(Action::Select(n)) => {
                self.app.select(n);
                "global:select_mode"
            }
            Some(Action::Quit) => {
                self.app.quit();
                "global:quit"
            }
            None => {
                self.app.mode_key(key);
                "mode"
            }
        }
    }

    /// Render the current state and flatten the buffer to text rows,
    /// trailing blank cells trimmed.
    fn snapshot_text(&self) -> String {
        let mut terminal = Terminal::new(TestBackend::new(self.cols, self.rows))
            .expect("test backend needs no tty");
        terminal
            .draw(|frame| ui::render(frame, &self.app))
            .expect("draw into test backend");
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Parse a key name from the harness vocabulary into a key event.
/// Returns `None` for anything the interactive app could not receive.
pub fn key_event(name: &str) -> Option<KeyEvent> {
    let none = KeyModifiers::NONE;
    let code = match name {
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "esc" => KeyCode::Esc,
        "q" => KeyCode::Char('q'),
        "r" => KeyCode::Char('r'),
        "n" => KeyCode::Char('n'),
        "p" => KeyCode::Char('p'),
        "1" => KeyCode::Char('1'),
        "2" => KeyCode::Char('2'),
        "3" => KeyCode::Char('3'),
        "4" => KeyCode::Char('4'),
        "ctrl-c" => return Some(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        _ => return None,
    };
    Some(KeyEvent::new(code, none))
}

/// The tools the harness exposes, as MCP tool descriptors.
fn tool_descriptors() -> Value {
    json!([
        {
            "name": "send_key",
            "description": "Send one key press through the app's real global-then-mode dispatch.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "enum": ["enter", "tab", "backtab", "up", "down", "left", "right",
                                 "esc", "q", "r", "n", "p", "1", "2", "3", "4", "ctrl-c"]
                    }
                },
                "required": ["key"]
            }
        },
        {
            "name": "tick",
            "description": "Advance the active mode's animation by n ticks (1-1000, default 1).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "n": { "type": "integer", "minimum": 1, "maximum": 1000 }
                }
            }
        },
        {
            "name": "snapshot",
            "description": "Render the current screen and return it as text rows.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "reset",
            "description": "Replace the app with a fresh instance, optionally resizing the snapshot (rows/cols, 20-500 each).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "rows": { "type": "integer", "minimum": 20, "maximum": 500 },
                    "cols": { "type": "integer", "minimum": 20, "maximum": 500 }
                }
            }
        }
    ])
}

/// Dispatch one JSON-RPC request. Returns the response value, or `None` for
/// notifications (no `id`). Pure with respect to I/O.
pub fn dispatch(state: &mut HarnessState, request: Value) -> Option<Value> {
    let id = request.get("id").cloned();
    let respond = |result: Result<Value, (i64, String)>| -> Option<Value> {
        let id = id.clone().unwrap_or(Value::Null);
        Some(match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
            }
        })
    };

    // Envelope: an object with a string method. Arrays (batches) and
    // anything else are invalid requests; MCP does not use batching.
    let method = match request {
        Value::Object(ref map) => match map.get("method") {
            Some(Value::String(m)) => m.clone(),
            _ => return respond(Err((-32600, "invalid request: missing method".to_string()))),
        },
        _ => return respond(Err((-32600, "invalid request: not an object".to_string()))),
    };
    if request.get("jsonrpc") != Some(&Value::String("2.0".into())) {
        return respond(Err((
            -32600,
            "invalid request: jsonrpc must be \"2.0\"".to_string(),
        )));
    }
    // Notifications carry no id and get no response.
    let is_notification = request.get("id").is_none();

    let result: Result<Value, (i64, String)> = match method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "why-is-my-cipher-vulnerable-bro", "version": env!("CARGO_PKG_VERSION") }
        })),
        "notifications/initialized" => {
            if is_notification {
                return None;
            }
            Err((-32600, "initialized is a notification".to_string()))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_descriptors() })),
        "tools/call" => {
            if is_notification {
                return None;
            }
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            call_tool(state, &params).map_err(|msg| (-32602, msg))
        }
        other => Err((-32601, format!("method not found: {other}"))),
    };

    if is_notification && result.is_ok() {
        return None; // any well-formed notification we accept silently
    }
    respond(result)
}

/// Handle a `tools/call`: tool-logic failures become tool results with
/// `isError: true`; protocol failures (unknown tool, bad arguments) become
/// JSON-RPC errors.
fn call_tool(state: &mut HarnessState, params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "params.name is required".to_string())?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
        "send_key" => {
            let key_name = args
                .get("key")
                .and_then(Value::as_str)
                .ok_or_else(|| "arguments.key is required".to_string())?;
            let key = key_event(key_name).ok_or_else(|| format!("unknown key: {key_name}"))?;
            let routed = state.apply_key(key);
            Ok(tool_text(
                json!({ "routed": routed, "quit": state.app.should_quit }).to_string(),
            ))
        }
        "tick" => {
            let n = match args.get("n") {
                None => 1,
                Some(v) => v
                    .as_i64()
                    .ok_or_else(|| "arguments.n must be an integer".to_string())?,
            };
            if !(TICK_MIN..=TICK_MAX).contains(&n) {
                return Err(format!("arguments.n must be {TICK_MIN}..={TICK_MAX}"));
            }
            for _ in 0..n {
                state.app.tick();
            }
            Ok(tool_text(json!({ "ticked": n }).to_string()))
        }
        "snapshot" => Ok(tool_text(state.snapshot_text())),
        "reset" => {
            let rows = optional_size(&args, "rows")?;
            let cols = optional_size(&args, "cols")?;
            state.app = App::new();
            if let Some(rows) = rows {
                state.rows = rows;
            }
            if let Some(cols) = cols {
                state.cols = cols;
            }
            Ok(tool_text(
                json!({ "rows": state.rows, "cols": state.cols }).to_string(),
            ))
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Validate one optional size argument against the allowed range.
fn optional_size(args: &Value, field: &str) -> Result<Option<u16>, String> {
    match args.get(field) {
        None => Ok(None),
        Some(v) => {
            let n = v
                .as_i64()
                .ok_or_else(|| format!("arguments.{field} must be an integer"))?;
            if !(SIZE_MIN as i64..=SIZE_MAX as i64).contains(&n) {
                return Err(format!("arguments.{field} must be {SIZE_MIN}..={SIZE_MAX}"));
            }
            Ok(Some(n as u16))
        }
    }
}

/// Wrap text in an MCP tool-result content block.
fn tool_text(text: String) -> Value {
    json!({ "content": [ { "type": "text", "text": text } ] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Mode;
    use crate::modes::autocrack::Phase;

    fn request(method: &str, params: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
    }

    fn call(state: &mut HarnessState, tool: &str, args: Value) -> Value {
        dispatch(
            state,
            request("tools/call", json!({ "name": tool, "arguments": args })),
        )
        .unwrap()
    }

    fn tool_text_of(response: &Value) -> String {
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool results carry text")
            .to_string()
    }

    fn is_error(response: &Value) -> bool {
        response.get("error").is_some() || response["result"].get("isError").is_some()
    }

    #[test]
    fn initialize_advertises_tools_and_protocol_version() {
        let mut state = HarnessState::new();
        let resp = dispatch(&mut state, request("initialize", json!({}))).unwrap();
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert_eq!(resp["id"], 1);
    }

    #[test]
    fn tools_list_names_all_four_tools() {
        let mut state = HarnessState::new();
        let resp = dispatch(&mut state, request("tools/list", json!({}))).unwrap();
        let names: Vec<&str> = resp["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["send_key", "tick", "snapshot", "reset"]);
    }

    #[test]
    fn unknown_method_is_method_not_found_with_matching_id() {
        let mut state = HarnessState::new();
        let resp = dispatch(&mut state, request("resources/list", json!({}))).unwrap();
        assert_eq!(resp["error"]["code"], -32601);
        assert_eq!(resp["id"], 1);
    }

    #[test]
    fn bad_envelope_and_batches_are_invalid_requests() {
        let mut state = HarnessState::new();
        let resp = dispatch(&mut state, json!("not an object")).unwrap();
        assert_eq!(resp["error"]["code"], -32600);
        let resp = dispatch(&mut state, json!([request("initialize", json!({}))])).unwrap();
        assert_eq!(resp["error"]["code"], -32600);
        let resp = dispatch(&mut state, json!({ "id": 9, "method": "ping" })).unwrap();
        assert_eq!(resp["error"]["code"], -32600);
        assert_eq!(resp["id"], 9);
    }

    #[test]
    fn notifications_get_no_response() {
        let mut state = HarnessState::new();
        let note = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(dispatch(&mut state, note).is_none());
    }

    #[test]
    fn send_key_enter_starts_the_attack_and_tab_advances_the_mode() {
        let mut state = HarnessState::new();
        call(&mut state, "send_key", json!({ "key": "enter" }));
        assert!(
            matches!(state.app.autocrack.phase, Phase::Running(_)),
            "enter starts the victim's attack"
        );

        call(&mut state, "send_key", json!({ "key": "tab" }));
        assert_eq!(state.app.mode, Mode::TrafficSimulator);
        // And it matches the live loop's global action exactly.
        let mut live = App::new();
        live.next();
        assert_eq!(state.app.mode, live.mode);
    }

    #[test]
    fn unknown_key_is_a_tool_error_and_leaves_state_unchanged() {
        let mut state = HarnessState::new();
        let resp = call(&mut state, "send_key", json!({ "key": "floppy eject" }));
        assert!(is_error(&resp), "unknown key errors");
        assert!(matches!(state.app.autocrack.phase, Phase::Select { .. }));
    }

    #[test]
    fn ticks_drive_the_caesar_attack_to_completion() {
        let mut state = HarnessState::new();
        call(&mut state, "send_key", json!({ "key": "enter" }));
        call(&mut state, "tick", json!({ "n": 30 }));
        let snapshot = tool_text_of(&call(&mut state, "snapshot", json!({})));
        assert!(snapshot.contains("done"), "attack finished");
        assert!(
            snapshot.contains("Key:      shift 11"),
            "recovered key visible"
        );
    }

    #[test]
    fn tick_bounds_are_enforced() {
        let mut state = HarnessState::new();
        for bad in [0, -1, 1001, 10_000] {
            let resp = call(&mut state, "tick", json!({ "n": bad }));
            assert!(is_error(&resp), "n={bad} rejected");
        }
    }

    #[test]
    fn snapshot_shows_the_victim_list() {
        let state = HarnessState::new();
        let text = state.snapshot_text();
        assert!(text.contains("Pick a victim"));
        assert!(text.contains("Caesar (shift 11)"));
        assert!(text.contains("Vigenère (key \"lemon\")"));
    }

    #[test]
    fn reset_clears_state_and_validates_sizes() {
        let mut state = HarnessState::new();
        call(&mut state, "send_key", json!({ "key": "enter" }));
        call(&mut state, "tick", json!({ "n": 3 }));
        call(&mut state, "reset", json!({}));
        assert!(matches!(state.app.autocrack.phase, Phase::Select { .. }));

        let before = (state.rows, state.cols);
        let resp = call(&mut state, "reset", json!({ "rows": 5 }));
        assert!(is_error(&resp), "rows 5 rejected");
        assert_eq!((state.rows, state.cols), before, "size kept on error");

        call(&mut state, "reset", json!({ "rows": 40, "cols": 120 }));
        assert_eq!((state.rows, state.cols), (40, 120));
    }
}
