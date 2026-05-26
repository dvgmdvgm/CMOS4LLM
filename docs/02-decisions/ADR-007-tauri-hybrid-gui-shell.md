# ADR-007: Tauri hybrid shell + web core + IDE plugins

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner
- **Related:** ADR-006 (GUI density), ADR-010 (MCP-first integration)

## Context

The Cognitive Console needs to be reachable from several surfaces:

- A standalone desktop app (system tray, hotkeys, persistent presence).
- A web view that any browser can hit (when the user is on another machine, when they want to share a screen, etc.).
- An IDE panel inside VS Code (and later JetBrains) — where the work actually happens.

Three shell strategies were on the table:

| Option | Pros | Cons |
|---|---|---|
| Native desktop (Tauri or Electron) | OS integration, hotkeys, tray, polish | Single binary surface |
| Web-served local daemon (browser at `localhost:7000`) | Cross-platform free, easy IDE integration via webview | No OS integration, no tray |
| IDE-only (VS Code panel) | Lives where the work happens | Cramped for deep dashboards, vendor-coupled |
| **Hybrid** | All three above, one source of truth | More work upfront |

Owner explicitly chose hybrid.

## Decision

**Cognitive Console is built as a hybrid:**

1. **Web core** — a single React + TypeScript application that contains all UI for all screens. Served by the CMOS daemon over HTTP/WebSocket on `localhost:7077` (or a configurable port).
2. **Tauri desktop shell** — wraps the web core in a native window. Adds: system tray icon, global hotkeys, native notifications, single-instance enforcement, deep-link handling (`cmos://` URLs).
3. **VS Code extension (V1)** — embeds the same web core as a VS Code webview, scoped to a panel. Sends/receives messages to the CMOS daemon via the same WebSocket protocol.
4. **JetBrains plugin (V2)** — same pattern, JetBrains plugin with embedded webview.

All four surfaces talk to one CMOS daemon. The daemon is the source of truth; the surfaces are renderers.

## Rationale

- **One UI codebase, three surfaces.** Without the hybrid, we'd write the same memory browser three times (desktop, web, IDE) — unaffordable.
- **Tauri vs Electron:** Tauri ships ~10× smaller binaries (3–10 MB vs 100+ MB), uses the OS WebView (no Chromium bundled), and is implemented in Rust — same language as the CMOS core (ADR-011 TBD). One language across the stack reduces context-switching cost.
- **localhost web access matters.** Sharing a session with a colleague, opening Cognitive Console from a different machine on the same LAN, embedding screenshots in PRs — all become trivial when the same UI is reachable in any browser.
- **IDE integration matters.** A developer's hands are in VS Code; making them context-switch to a desktop window for inspection is friction.
- **Single daemon is necessary anyway.** Memory and policies cannot be split across surfaces; the daemon owns state. Once we have a daemon, multi-surface is mostly UX work, not architectural.

## Consequences

### Positive
- All surfaces stay in sync trivially (they read the same daemon).
- VS Code extension is a thin shell — no duplicate React app maintenance.
- Tauri's small binary makes installation trivial; auto-update via Tauri Updater.
- Web access enables remote-pair scenarios.

### Negative
- We must be careful about WebSocket auth/CORS — `localhost:7077` is reachable from any browser process on the machine. Mitigations: strict origin checks, token-bound sessions, optional Unix-domain-socket mode.
- Tauri's IPC API is different from Electron's; learning curve.
- WebView capabilities differ across OS (Windows uses WebView2, macOS uses WKWebView, Linux uses WebKitGTK). Most modern web is fine, but heavy graph viz (Cytoscape) needs cross-WebView testing.

### Neutral / unknowns
- VS Code Webview vs LSP-server-only integration — Webview is the choice, LSP is overkill for dashboards.
- Mobile companion app — explicitly out of scope (Future per ROADMAP).

## Alternatives considered

- **Pure Tauri, no web access:** rejected — loses sharing/remote benefit, blocks IDE plugin reuse.
- **Pure web (browser only, no native shell):** rejected — no tray, no hotkeys, awkward for daily use.
- **Electron:** rejected — bundle size, RAM footprint, no shared language with backend.
- **Native (Cocoa / WPF / GTK):** rejected — three implementations, no IDE reuse, glacial development.

## Implementation notes

- Repo layout (target):
  ```
  apps/
    desktop/        Tauri shell (Rust + minimal config)
    web/            standalone dev server for the web core
    vscode/         VS Code extension (TS + webview)
    jetbrains/      [V2]
  packages/
    ui/             React + TS UI components, screens, design system
    api-client/     WebSocket / HTTP client for CMOS daemon
    contracts/      shared TS types mirrored from Rust core
  ```
- The web core is one application (`packages/ui`); each shell mounts it.
- Auth: short-lived bearer token; daemon refuses requests without it; token is provisioned to each shell via OS-keychain (Tauri) / extension secret store (VS Code) / explicit pairing (browser).
- Network: WebSocket for live (Cognitive Trace, Live Inference Inspector); HTTP for snapshot reads; gRPC reserved for future high-throughput cases.
- Hotkeys: Tauri only. Web shell and VS Code shell rely on browser/IDE keybindings.

## Revisit conditions

- If WebView2 / WKWebView capability gaps make heavy graph viz unusable, consider bundling Chromium (Electron fallback) for the desktop shell only.
- If JetBrains plugin's webview has fundamental incompatibilities, consider a thin-LSP variant for JetBrains and keep webview elsewhere.
- If the security model of "localhost web" proves leaky, switch desktop shell to Unix-domain socket + named pipe (Windows) by default.
