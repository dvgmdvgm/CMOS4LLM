# ADR-011: Rust as the core daemon language

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-007, ADR-003, ADR-010

## Context

CMOS core daemon must: embed a graph store, vector index, and event log in-process; bind to llama.cpp for Sub-LM inference; serve MCP/HTTP/gRPC; and share a language with the Tauri 2.x GUI shell (ADR-007). The latency budget is <200ms p95 on the critical path. The primary deployment target is Windows 11 desktop.

## Decision

**Rust is the sole language for the CMOS core daemon, CLI, and Tauri shell backend.** All in-process components (memory stores, retrieval router, policy engine, Sub-LM bindings) are Rust crates within a single workspace.

## Rationale

1. **Tauri 2.x is Rust-native** — choosing any other language would mean maintaining two backend stacks (one for Tauri commands, one for the daemon) or proxying everything through IPC.
2. **llama-cpp-2 crate** (111K downloads/month, actively maintained) provides in-process GGUF inference with CUDA support — the best Sub-LM binding available.
3. **Embedded DB ecosystem** — rusqlite, LanceDB, redb are all Rust-native or have first-class Rust crates. No CGo overhead, no Python GIL.
4. **Performance** — zero-cost abstractions, no GC pauses, predictable latency. Critical for the <200ms retrieval+assembly budget.
5. **Single binary deployment** — Rust + Tauri produces a 3–10 MB self-contained binary. No runtime dependencies for the user to install.
6. **Memory safety** — CMOS is a long-running daemon managing persistent state. Use-after-free or data races in memory management would be catastrophic.

## Consequences

### Positive
- One language across daemon, CLI, Tauri backend, and all embedded stores.
- Excellent Windows support (MSVC toolchain, first-class target).
- Cargo workspace enables clean crate boundaries between components.
- Strong type system catches integration errors at compile time.

### Negative
- Steeper learning curve than Go for contributors unfamiliar with Rust.
- Longer compile times (mitigated by workspace crate splitting and incremental builds).
- Some C++ dependencies (llama.cpp, SQLite) still require clang/LLVM on Windows for building.

### Neutral / unknowns
- Async runtime choice (tokio vs async-std) — tokio is de facto standard, will use it.
- Whether owner has prior Rust experience (affects velocity of first milestones).

## Alternatives considered

- **Go:** Simpler language, faster compilation. Rejected because: (a) Tauri shell is Rust — would need IPC bridge or separate process; (b) CGo required for all C/C++ deps (llama.cpp, SQLite, vector index) — adds complexity and hurts cross-compilation; (c) GC pauses conflict with latency budget on hot path; (d) no equivalent to Cargo workspace for clean component boundaries.
- **Rust core + Go for CLI/utilities:** Rejected because the added complexity of two build systems and two languages outweighs the marginal ergonomic benefit of Go for CLI tools. Rust's `clap` is mature enough.
- **Python:** Not considered seriously — wrong performance class for a daemon, GIL prevents true parallelism, deployment complexity (venv, pip).

## Implementation notes

- Workspace structure: `crates/core`, `crates/memory`, `crates/sub-lm`, `crates/gateway`, `crates/policy`, `crates/retrieval`, `crates/cli`.
- Tauri app in `apps/desktop/` with `src-tauri/` linking to workspace crates.
- CI: `cargo clippy`, `cargo test`, `cargo build --release` on Windows runner.
- MSRV: latest stable (no nightly features required).

## Revisit conditions

- If owner finds Rust velocity unacceptable after M1 and wants to switch to Go for non-critical-path components, we can extract CLI/tooling into Go while keeping the daemon in Rust.
- If a superior Rust-native LLM inference library emerges that obsoletes llama-cpp-2, the language choice remains valid — only the Sub-LM crate internals change.
