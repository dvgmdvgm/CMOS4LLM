use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn cargo_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("cmos-cli.exe");
    path
}

fn temp_root() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn send_jsonrpc(stdin: &mut impl Write, id: u64, method: &str, params: serde_json::Value) {
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let line = serde_json::to_string(&msg).unwrap();
    writeln!(stdin, "{}", line).unwrap();
    stdin.flush().unwrap();
}

fn send_notification(stdin: &mut impl Write, method: &str, params: serde_json::Value) {
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    let line = serde_json::to_string(&msg).unwrap();
    writeln!(stdin, "{}", line).unwrap();
    stdin.flush().unwrap();
}

fn read_response(reader: &mut BufReader<impl std::io::Read>) -> serde_json::Value {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 {
            panic!("EOF from MCP server");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(trimmed)
            .unwrap_or_else(|e| panic!("Failed to parse JSON: {e}\nLine: {trimmed}"));
        // Skip notifications (no "id" field)
        if val.get("id").is_some() {
            return val;
        }
    }
}

#[test]
fn test_mcp_full_lifecycle() {
    let bin = cargo_bin();
    if !bin.exists() {
        panic!(
            "cmos-cli binary not found at {:?}. Run `cargo build` first.",
            bin
        );
    }

    let root = temp_root();
    let cmos_dir = root.path().join(".cmos");
    std::fs::create_dir_all(&cmos_dir).unwrap();

    let mut child = Command::new(&bin)
        .args(["mcp", "--root", root.path().to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn cmos mcp");

    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // --- Initialize ---
    send_jsonrpc(
        &mut stdin,
        1,
        "initialize",
        serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "0.1.0" }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["serverInfo"]["name"], "cmos");
    assert!(resp["result"]["capabilities"]["tools"].is_object());

    // Send initialized notification
    send_notification(&mut stdin, "notifications/initialized", serde_json::json!({}));

    // --- List tools ---
    send_jsonrpc(&mut stdin, 2, "tools/list", serde_json::json!({}));
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 2);
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 6);
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"cmos_write_memory"));
    assert!(tool_names.contains(&"cmos_read_memory"));
    assert!(tool_names.contains(&"cmos_query_memory"));
    assert!(tool_names.contains(&"cmos_assemble_context"));
    assert!(tool_names.contains(&"cmos_search_similar"));
    assert!(tool_names.contains(&"cmos_memory_stats"));

    // --- Write to L1 ---
    send_jsonrpc(
        &mut stdin,
        3,
        "tools/call",
        serde_json::json!({
            "name": "cmos_write_memory",
            "arguments": {
                "project_id": "test-proj",
                "slot_id": "ctx-auth",
                "content": "Authentication uses JWT with RS256. Tokens expire after 1h.",
                "priority": "policy"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 3);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["status"], "written");
    assert_eq!(parsed["slot_id"], "ctx-auth");
    assert!(parsed["total_tokens"].as_u64().unwrap() > 0);

    // --- Read from L1 ---
    send_jsonrpc(
        &mut stdin,
        4,
        "tools/call",
        serde_json::json!({
            "name": "cmos_read_memory",
            "arguments": {
                "project_id": "test-proj",
                "slot_id": "ctx-auth"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 4);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["id"], "ctx-auth");
    assert!(parsed["content"].as_str().unwrap().contains("JWT"));
    assert_eq!(parsed["priority"], "Policy");

    // --- Read non-existent slot (error case) ---
    send_jsonrpc(
        &mut stdin,
        5,
        "tools/call",
        serde_json::json!({
            "name": "cmos_read_memory",
            "arguments": {
                "project_id": "test-proj",
                "slot_id": "nonexistent"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 5);
    assert_eq!(resp["result"]["isError"], true);

    // --- Memory stats ---
    send_jsonrpc(
        &mut stdin,
        6,
        "tools/call",
        serde_json::json!({
            "name": "cmos_memory_stats",
            "arguments": {
                "project_id": "test-proj"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 6);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["l1"]["slot_count"], 1);
    assert!(parsed["l1"]["total_tokens"].as_u64().unwrap() > 0);

    // --- Assemble context (keyword-only, no vector index) ---
    send_jsonrpc(
        &mut stdin,
        7,
        "tools/call",
        serde_json::json!({
            "name": "cmos_assemble_context",
            "arguments": {
                "project_id": "test-proj",
                "task_description": "Fix the authentication token refresh logic",
                "max_tokens": 8000
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 7);
    // assemble_context may return an error if no events/facts DB exists yet — that's fine
    if resp["result"]["isError"] == true {
        let err_text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
        eprintln!("assemble_context returned error (expected for empty DB): {err_text}");
    } else {
        let content = resp["result"]["content"][0]["text"].as_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(content)
            .unwrap_or_else(|e| panic!("Failed to parse assemble_context response: {e}\nRaw: {content}"));
        assert!(!parsed["context"].as_str().unwrap().is_empty());
        assert!(parsed["total_tokens"].as_u64().unwrap() > 0);
        assert_eq!(parsed["budget"], 8000);
    }

    // --- Query L4 (empty, no facts stored yet) ---
    send_jsonrpc(
        &mut stdin,
        8,
        "tools/call",
        serde_json::json!({
            "name": "cmos_query_memory",
            "arguments": {
                "project_id": "test-proj",
                "layer": "L4"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 8);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert!(parsed.as_array().unwrap().is_empty());

    // --- Unknown tool ---
    send_jsonrpc(
        &mut stdin,
        9,
        "tools/call",
        serde_json::json!({
            "name": "nonexistent_tool",
            "arguments": {}
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 9);
    // SDK may return either a top-level "error" or a result with isError=true
    let is_error = resp["error"].is_object() || resp["result"]["isError"] == true;
    assert!(is_error, "Expected error for unknown tool, got: {resp}");

    // --- Write second slot, verify count ---
    send_jsonrpc(
        &mut stdin,
        10,
        "tools/call",
        serde_json::json!({
            "name": "cmos_write_memory",
            "arguments": {
                "project_id": "test-proj",
                "slot_id": "ctx-db",
                "content": "Database is PostgreSQL 15 with pgvector extension.",
                "priority": "context"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 10);
    let content = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    assert_eq!(parsed["slot_count"], 2);

    // --- Cleanup ---
    drop(stdin);
    child.kill().ok();
    child.wait().ok();
}

#[test]
fn test_mcp_invalid_params() {
    let bin = cargo_bin();
    if !bin.exists() {
        panic!(
            "cmos-cli binary not found at {:?}. Run `cargo build` first.",
            bin
        );
    }

    let root = temp_root();
    let cmos_dir = root.path().join(".cmos");
    std::fs::create_dir_all(&cmos_dir).unwrap();

    let mut child = Command::new(&bin)
        .args(["mcp", "--root", root.path().to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn cmos mcp");

    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // Initialize
    send_jsonrpc(
        &mut stdin,
        1,
        "initialize",
        serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "0.1.0" }
        }),
    );
    read_response(&mut reader);
    send_notification(&mut stdin, "notifications/initialized", serde_json::json!({}));

    // Call write_memory with missing required fields
    send_jsonrpc(
        &mut stdin,
        2,
        "tools/call",
        serde_json::json!({
            "name": "cmos_write_memory",
            "arguments": {
                "project_id": "test-proj"
            }
        }),
    );
    let resp = read_response(&mut reader);
    assert_eq!(resp["id"], 2);
    assert_eq!(resp["result"]["isError"], true);

    drop(stdin);
    child.kill().ok();
    child.wait().ok();
}
