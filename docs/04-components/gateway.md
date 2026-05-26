# Component: Gateway

> Single entry point for all external communication with the CMOS daemon.

## Responsibility

- Accept connections via MCP, HTTP/WebSocket, and gRPC ([ADR-010](../02-decisions/ADR-010-mcp-first-integration.md))
- Route requests to Context Hypervisor
- Enforce `project_id` on every request ([ADR-004](../02-decisions/ADR-004-multi-project-from-day-one.md))
- Session management and authentication

## Interfaces

TODO: Define MCP resource/tool surface
TODO: Define HTTP/WS endpoint schema
TODO: Define gRPC service proto

## Dependencies

- Context Hypervisor (downstream)
- Observability (telemetry sink)

## Scope

- [MVP M10](../03-scope/mvp.md) — MCP server
- [V1.H](../03-scope/v1.md) — VS Code extension surface

## Open Questions

TODO: Auth model for localhost vs remote access
TODO: Rate limiting strategy for Sub-LM fallback calls
