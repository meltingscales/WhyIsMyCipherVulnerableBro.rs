//! End-to-end harness tests: spawn the real binary in `--mcp` mode and
//! speak JSON-RPC over stdio. Every stdout line read here must parse as
//! JSON, which is exactly the protocol-clean guarantee.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

use serde_json::{Value, json};

struct Harness {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl Harness {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_why_is_my_cipher_vulnerable_bro"))
            .arg("--mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn harness binary");
        let stdout = child.stdout.take().expect("piped stdout");
        Self {
            reader: BufReader::new(stdout),
            child,
        }
    }

    /// Send one request line and read its one response line.
    fn request(&mut self, id: i64, method: &str, params: Value) -> Value {
        let stdin = self.child.stdin.as_mut().expect("piped stdin");
        writeln!(
            stdin,
            "{}",
            json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
        )
        .expect("write request");
        stdin.flush().expect("flush request");
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("read response");
        serde_json::from_str(&line).expect("every stdout line is JSON")
    }

    /// Send a notification (no response expected).
    fn notify(&mut self, method: &str) {
        let stdin = self.child.stdin.as_mut().expect("piped stdin");
        writeln!(stdin, "{}", json!({ "jsonrpc": "2.0", "method": method })).expect("write");
        stdin.flush().expect("flush");
    }

    fn call(&mut self, id: i64, tool: &str, args: Value) -> Value {
        self.request(id, "tools/call", json!({ "name": tool, "arguments": args }))
    }

    fn snapshot_text(&mut self, id: i64) -> String {
        let resp = self.call(id, "snapshot", json!({}));
        resp["result"]["content"][0]["text"]
            .as_str()
            .expect("snapshot carries text")
            .to_string()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

#[test]
fn harness_drives_full_scenarios_end_to_end() {
    let mut h = Harness::start();

    // Handshake.
    let init = h.request(1, "initialize", json!({}));
    assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    h.notify("notifications/initialized");

    let tools = h.request(2, "tools/list", json!({}));
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .expect("tools list")
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert_eq!(
        names,
        ["send_key", "send_click", "tick", "snapshot", "reset"]
    );

    // Error paths leave state intact.
    assert!(h.call(3, "send_key", json!({ "key": "make coffee" }))["error"].is_object());
    assert!(h.call(4, "tick", json!({ "n": 0 }))["error"].is_object());

    // Autocrack scenario: enter + ticks + snapshot shows the recovered key.
    h.call(5, "send_key", json!({ "key": "enter" }));
    h.call(6, "tick", json!({ "n": 30 }));
    let snapshot = h.snapshot_text(7);
    assert!(
        snapshot.contains("Attack — done"),
        "attack completed: {snapshot}"
    );
    assert!(snapshot.contains("Key:      shift 11"), "key recovered");

    // Traffic scenario (fill-mode-panels re-verification): tab over, decrypt,
    // tick the reveal, snapshot shows recovered key and plaintext.
    h.call(8, "send_key", json!({ "key": "tab" }));
    h.call(9, "send_key", json!({ "key": "enter" }));
    h.call(10, "tick", json!({ "n": 20 }));
    let snapshot = h.snapshot_text(11);
    assert!(
        snapshot.contains("Recovered — key 6b3f"),
        "xor key recovered"
    );
    assert!(snapshot.contains("POST /login"), "plaintext revealed");

    // Reset clears everything back to the victim list; an error call after
    // it leaves that fresh state untouched.
    h.call(12, "reset", json!({}));
    assert!(h.call(13, "send_key", json!({ "key": "no-such-key" }))["error"].is_object());
    let snapshot = h.snapshot_text(14);
    assert!(
        snapshot.contains("Pick a victim"),
        "reset returned to autocrack start"
    );

    // SWEET32 scenario: third victim, tick to the collision, snapshot shows
    // the birthday bound and the leaked cookie.
    h.call(15, "send_key", json!({ "key": "down" }));
    h.call(16, "send_key", json!({ "key": "down" }));
    h.call(17, "send_key", json!({ "key": "enter" }));
    h.call(18, "tick", json!({ "n": 60 }));
    let snapshot = h.snapshot_text(19);
    assert!(
        snapshot.contains("Attack — done"),
        "birthday attack completed"
    );
    assert!(
        snapshot.contains("Result:   birthday collision after"),
        "collision reported"
    );
    assert!(
        snapshot.contains("Leak:     leaked cookie: "),
        "cookie leaked"
    );

    // FREAK scenario: fourth victim, downgrade then factoring, full chain.
    h.call(20, "reset", json!({}));
    for _ in 0..3 {
        h.call(21, "send_key", json!({ "key": "down" }));
    }
    h.call(22, "send_key", json!({ "key": "enter" }));
    h.call(23, "tick", json!({ "n": 40 }));
    let snapshot = h.snapshot_text(24);
    assert!(snapshot.contains("Attack — done"), "factoring completed");
    assert!(snapshot.contains("ClientHello"), "downgrade narrated");
    assert!(
        snapshot.contains("Result:   N = "),
        "factorization reported"
    );
    assert!(
        snapshot.contains("premaster secret: PMS-42!"),
        "chain completes"
    );

    // Logjam scenario: fifth victim, downgrade then discrete log.
    h.call(25, "reset", json!({}));
    for _ in 0..4 {
        h.call(26, "send_key", json!({ "key": "down" }));
    }
    h.call(27, "send_key", json!({ "key": "enter" }));
    h.call(28, "tick", json!({ "n": 60 }));
    let snapshot = h.snapshot_text(29);
    assert!(snapshot.contains("Attack — done"), "discrete log completed");
    assert!(
        snapshot.contains("Result:   y = 314159265"),
        "exponent recovered"
    );
    assert!(
        snapshot.contains("session: GET /export HTTP/1.1"),
        "session decrypted"
    );

    // Mouse scenario: click the Lessons tab, then click a lesson row.
    h.call(30, "reset", json!({}));
    let tab3 = {
        let resp = h.call(31, "snapshot", json!({}));
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        // Tab row is screen row 1 (0-based); find the Lessons tab's column
        // by locating its rendered title in that row.
        let row = text.lines().nth(1).unwrap();
        let col = row.find("3: Lessons").expect("lessons tab rendered") as i64;
        (1i64, col)
    };
    let resp = h.call(
        32,
        "send_click",
        json!({ "row": tab3.0, "column": tab3.1 + 2 }),
    );
    assert!(
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("shell:tab"),
        "tab click routed to the shell"
    );
    let snapshot = h.snapshot_text(33);
    assert!(snapshot.contains("Lessons"), "lessons mode active");
    assert!(!snapshot.contains("Birthday collisions"), "not math views");

    // Click the SWEET32 lesson row: find its screen row by its title.
    let resp = h.call(34, "snapshot", json!({}));
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    let (lesson_row, col) = text
        .lines()
        .enumerate()
        .find_map(|(y, line)| line.find("SWEET32").map(|x| (y as i64, x as i64)))
        .expect("lesson row rendered");
    h.call(
        35,
        "send_click",
        json!({ "row": lesson_row, "column": col }),
    );
    let snapshot = h.snapshot_text(36);
    assert!(snapshot.contains("step 1 of"), "lesson opened by click");

    // Out-of-range click errors and leaves state intact.
    assert!(h.call(37, "send_click", json!({ "row": 999, "column": 5 }))["error"].is_object());
}

#[test]
fn unknown_flags_exit_with_usage_on_stderr() {
    for args in [vec!["--bogus"], vec!["--mcp", "extra"]] {
        let out = Command::new(env!("CARGO_BIN_EXE_why_is_my_cipher_vulnerable_bro"))
            .args(&args)
            .output()
            .expect("run binary");
        assert_eq!(out.status.code(), Some(2), "args: {args:?}");
        assert!(!out.stderr.is_empty(), "usage goes to stderr");
        assert!(out.stdout.is_empty(), "stdout stays clean");
    }
}
