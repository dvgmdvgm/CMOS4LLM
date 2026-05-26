# ADR-010: MCP-first integration, with HTTP and gRPC as siblings

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-001 (stateless LLM), ADR-007 (Tauri hybrid shell)

## Context

CMOS is the substrate; LLM clients (Claude Code, Cursor, Continue, custom agents) are *consumers* of that substrate. The integration protocol determines:

- Which clients can use CMOS without writing a custom integration.
- How quickly a new client can be added.
- Whether tool authors target our protocol or have to glue.

The Model Context Protocol (MCP, Anthropic) has emerged in 2025 as the de-facto standard for connecting tools and resources to LLMs. By 2026 most major clients (Claude Desktop, Claude Code, Cursor, Windsurf, Continue) support MCP servers.

## Decision

**CMOS exposes MCP as the primary integration protocol.** The CMOS daemon ships as an MCP server out of the box. Any MCP-compatible client gets CMOS without code changes — the user installs CMOS, points the client at the MCP endpoint, and CMOS surfaces:

- **Resources:** project DNA, active policies, memory items, episodes, symbol graph slices.
- **Tools:** `assemble_context(query, scope)`, `record_decision(...)`, `extract_facts(...)`, `find_similar_episodes(...)`, `validate_against_policies(...)`, `time_travel(inference_id)`, `counterfactual(...)`.
- **Prompts:** parameterized prompt templates for common workflows.

Alongside MCP, the daemon also exposes:

- **HTTP/REST + WebSocket** for the GUI (Cognitive Console) and any non-MCP-aware client.
- **gRPC** (V1+) for high-throughput internal use (e.g., Sub-LM workers talking to the daemon, future internal microservices).

The three are different transports onto the same internal API. The internal API is the source of truth.

## Rationale

- **MCP is where the ecosystem is.** Building yet another protocol when MCP exists is wasted effort and isolates CMOS.
- **MCP-first means zero-friction adoption.** The user already has Claude Code or Cursor configured for MCP servers — adding CMOS is a config-file edit, not a custom plugin install.
- **Provider-portable.** MCP is not Anthropic-locked; its design is general. Other providers' agentic frameworks are converging on MCP-compatibility.
- **HTTP/WebSocket is required anyway** for the GUI (ADR-007). Once we have it, exposing the same API to any HTTP client is free.
- **gRPC is the right choice for daemon-internal coupling.** It's not user-facing, but it's the right tool for sub-millisecond, schema-strict internal RPCs.

## Consequences

### Positive
- Day-one compatibility with all major MCP clients.
- The user can mix-and-match: use CMOS through Claude Code for code generation, through a custom Python agent for analysis, through the GUI for inspection — all see the same memory.
- Adding a new client is a config edit, not a release.
- Internal architecture stays clean: one API, three transports.

### Negative
- MCP's schema and semantics evolve. We must track the spec and ship updates.
- Some advanced CMOS features (counterfactual mode UI, time-travel debugging) don't fit naturally into MCP tool calls — those require GUI / HTTP. We don't fight MCP to make these fit; we expose them where they belong.
- gRPC adds a build-time dependency (protobuf toolchain) for internal use. Acceptable.

### Neutral / unknowns
- Whether MCP gains streaming-resource semantics in time for our V1 — if not, we use WebSocket for the GUI live views (already planned).
- Auth in MCP is still maturing as of 2026 — for V1 we ship explicit token-based auth and revisit when the spec stabilizes.

## Alternatives considered

- **HTTP/REST only, no MCP:** rejected — every client author would have to write a custom CMOS plugin. Adoption would die.
- **MCP only, no HTTP:** rejected — GUI cannot run on MCP, and "headless-only" CMOS undermines the observability principle of the charter.
- **Custom protocol:** rejected — no upside; significant downside in adoption and integration cost.

## Implementation notes

- Repo structure (target):
  ```
  crates/
    cmos-core/        internal API + business logic
    cmos-mcp/         MCP server adapter (uses cmos-core)
    cmos-http/        HTTP + WebSocket adapter
    cmos-grpc/        gRPC adapter (V1+)
    cmos-daemon/      binary that wires all adapters together
  ```
- Internal API uses Rust trait objects / async traits; transports translate to/from the API.
- MCP resource URIs: `cmos://project/{project_id}/dna`, `cmos://project/{project_id}/policies`, `cmos://project/{project_id}/episodes/{id}`, etc.
- MCP tool names: prefix `cmos.` to avoid collisions with other servers (e.g., `cmos.assemble_context`).
- HTTP routes mirror MCP resource URIs where possible, simplifying translation.
- Auth: single token model across all transports (bearer for HTTP, MCP session token, gRPC metadata header).

## Revisit conditions

- If MCP forks (different vendors ship incompatible variants), CMOS commits to the canonical Anthropic spec and offers compatibility shims for variants.
- If a non-MCP standard wins the agentic ecosystem (unlikely but possible), CMOS adds a transport for it — the internal API design makes this cheap.
