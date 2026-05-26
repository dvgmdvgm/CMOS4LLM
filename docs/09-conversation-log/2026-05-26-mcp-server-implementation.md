# 2026-05-26 — MCP Server Implementation

> MCP protocol handler для CMOS — expose memory layers через Model Context Protocol.

---

## Контекст

M4 (Vector index) был на ~50%: vector search (USearch + hybrid retrieval) готов, MCP server — следующий шаг по roadmap. ADR-010 определяет MCP как primary integration protocol. Gateway crate существовал как skeleton (`init()` function).

---

## Что сделано

### `crates/gateway/src/tools.rs` (новый)
- 6 MCP tool definitions с `ToolInputSchema` (properties, required, descriptions).
- Helper `make_tool()` для DRY конструирования `Tool` struct (2025_11_25 schema имеет много полей).

### `crates/gateway/src/handler.rs` (новый)
- `CmosState`: holds `WorkingMemory` + `data_root` path, lazy-opens EventStore/ProjectMemory/VectorIndex.
- `CmosHandler`: implements `ServerHandler` trait, dispatches 6 tool calls.
- Tools: `cmos_read_memory`, `cmos_write_memory`, `cmos_query_memory`, `cmos_assemble_context`, `cmos_search_similar`, `cmos_memory_stats`.

### `crates/gateway/src/server.rs` (новый)
- `start_mcp_server(data_root)`: creates `McpServerOptions`, stdio transport, starts server.
- Protocol version: 2025_03_26.

### `crates/gateway/src/lib.rs` (изменён)
- Заменён skeleton на `pub mod handler/server/tools` + re-export `start_mcp_server`.

### `crates/gateway/Cargo.toml` (изменён)
- Добавлены deps: `rust-mcp-sdk`, `serde_json`, `async-trait`, `chrono`, `cmos-memory`, `cmos-retrieval`.

### `Cargo.toml` (workspace, изменён)
- Добавлен `rust-mcp-sdk = { version = "0.9", features = ["server"] }`.

### `crates/cli/src/main.rs` (изменён)
- Добавлена команда `cmos mcp --root <path>` — запускает MCP server.

### `crates/cli/Cargo.toml` (изменён)
- Добавлена зависимость `cmos-gateway`.

---

## Ключевые решения

1. **`rust-mcp-sdk` v0.9** — выбран как SDK (138K downloads, полная поддержка protocol 2025-11-25, stdio + SSE транспорты, proc macros). Альтернативы: `mcp-attr` (6.9K downloads, менее зрелый), ручная реализация JSON-RPC (слишком много boilerplate).

2. **Keyword-only assembly в MCP** — `cmos_assemble_context` использует `assemble()` вместо `assemble_hybrid()`. Причина: `VectorIndex` содержит `rusqlite::Connection` (не `Send`/`Sync`), а `ServerHandler` trait требует `Send + Sync + 'static`. Hybrid assembly держит `&VectorIndex` через await point, что нарушает Send-safety. Решение: semantic search доступен через отдельный tool `cmos_search_similar`, а для полного hybrid assembly нужно обернуть VectorIndex в `spawn_blocking` (будущая задача).

3. **6 tools вместо 7 из ADR-010** — `cmos.time_travel` и `cmos.counterfactual` отложены (V1+, не MVP). `cmos.validate_against_policies` отложен (policy crate — skeleton). Добавлен `cmos_memory_stats` как utility tool.

4. **Stdio transport** — для zero-friction интеграции с Claude Desktop / Claude Code. HTTP/SSE — будущее (ADR-010 предусматривает sibling transports).

---

## Открытые вопросы

- VectorIndex Send-safety: нужно обернуть в `Mutex` или `spawn_blocking` для hybrid assembly в MCP.
- Тестирование с реальным MCP client (Claude Desktop) — не проведено в этой сессии.
- Resources/Prompts из ADR-010 не реализованы (только Tools).

---

## Следующий шаг

Интеграция bootstrap → event store. См. [NEXT.md](../../NEXT.md).

---

## Метаданные

- **Дата:** 2026-05-26
- **Участники:** owner + Opus 4.6
- **Файлы созданы:** `crates/gateway/src/{handler,server,tools}.rs`
- **Файлы изменены:** `Cargo.toml` (workspace), `crates/gateway/{Cargo.toml,src/lib.rs}`, `crates/cli/{Cargo.toml,src/main.rs}`
- **Тесты:** 83 существующих проходят, 0 новых (MCP server тестируется интеграционно с client)
- **Clippy:** 0 warnings
- **Результат:** MCP Server полностью реализован, M4 ~80%
