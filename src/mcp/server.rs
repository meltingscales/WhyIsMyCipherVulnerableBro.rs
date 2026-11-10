//! The harness stdio server: newline-delimited JSON-RPC 2.0. Every line in
//! is one request; every request yields one response line (notifications
//! yield none). stdout carries only protocol messages; diagnostics go to
//! stderr.

use std::io::{self, BufRead, Write};

use serde_json::{Value, json};

use super::dispatch::{HarnessState, dispatch};

/// Serve MCP over stdin/stdout until stdin closes. Never touches the
/// terminal; safe to call from any process, including CI.
pub fn serve() -> io::Result<()> {
    let mut state = HarnessState::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => dispatch(&mut state, request),
            Err(err) => Some(json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32700, "message": format!("parse error: {err}") }
            })),
        };
        if let Some(response) = response {
            writeln!(out, "{response}")?;
            out.flush()?;
        }
    }
    Ok(())
}
