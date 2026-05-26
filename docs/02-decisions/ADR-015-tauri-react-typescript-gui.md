# ADR-015: Tauri 2.x + React + TypeScript for GUI

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-006, ADR-007, ADR-011

## Context

ADR-007 established the hybrid GUI architecture: web core + Tauri desktop shell + IDE plugins. This ADR finalizes the specific frontend stack within that shell.

Requirements from ADR-006: high-density DevTools-style UI, dark theme, monospace headings, real-time updates via WebSocket, interactive graphs (Cytoscape.js), code viewing (Monaco), timeseries charts (uPlot).

## Decision

**The Cognitive Console frontend uses React 19 + TypeScript 5.x, bundled with Vite, state managed by Zustand, running inside Tauri 2.x shell.** The same web core deploys as VS Code webview (V1) and standalone web app.

## Rationale

1. **React:** Largest ecosystem, best library support for all required visualization tools (Cytoscape.js, Monaco, uPlot, Recharts all have React wrappers). Component model fits the multi-panel DevTools layout.
2. **TypeScript:** Non-negotiable for a complex UI with many data types (InferenceRecord, PolicyObject, MemoryItem, etc.). Catches integration errors at compile time.
3. **Zustand:** Minimal, performant state management. No boilerplate (unlike Redux), supports WebSocket subscriptions natively, works well with React concurrent features. ~40K GitHub stars, actively maintained.
4. **Vite:** Fast HMR for development, efficient production builds. Native TypeScript support. Standard choice for React + Tauri projects.
5. **Tauri 2.x:** Already decided in ADR-007. Rust backend commands expose daemon functionality directly to the frontend — no HTTP proxy needed for local operations.

## Consequences

### Positive
- One codebase serves desktop (Tauri), VS Code extension (webview), and standalone web (direct HTTP).
- Rich ecosystem: every visualization library needed is available as a React component.
- TypeScript ensures type safety across the complex data model.
- Zustand + WebSocket provides reactive real-time updates without complexity.
- Vite + Tauri dev experience is fast (HMR in <100ms).

### Negative
- React bundle size (~40KB gzipped) — acceptable for desktop, but adds to initial load.
- Zustand is simple but may need middleware for complex cross-store subscriptions in V1+.
- Tauri 2.x is still evolving — occasional breaking changes between minor versions.

### Neutral / unknowns
- Component library choice: custom components vs Radix UI vs shadcn/ui. Leaning toward Radix primitives + custom styling for maximum density control.
- CSS approach: Tailwind CSS (utility-first, good for dense UIs) vs CSS Modules. TBD in implementation.
- Testing strategy: Vitest for unit, Playwright for E2E.

## Alternatives considered

- **Svelte/SvelteKit:** Smaller bundle, less boilerplate. Rejected: smaller ecosystem for visualization libraries (Cytoscape, Monaco wrappers are React-first), fewer developers familiar with it, less mature TypeScript support.
- **Solid.js:** Better performance than React (fine-grained reactivity). Rejected: much smaller ecosystem, visualization library support is limited, harder to hire/contribute.
- **Vue 3:** Viable alternative. Rejected: React has better TypeScript integration, larger ecosystem for the specific libraries we need (Monaco, Cytoscape), and Tauri's official examples are React-first.
- **Electron instead of Tauri:** Rejected in ADR-007 (10x larger binary, more memory usage, no Rust backend sharing).

## Implementation notes

- Frontend location: `apps/desktop/src/` (React app), `apps/desktop/src-tauri/` (Rust backend).
- Key dependencies: `react`, `react-dom`, `zustand`, `@tauri-apps/api`, `cytoscape`, `monaco-editor`, `uplot`, `recharts`.
- Build: `vite build` for production, `vite dev` + `cargo tauri dev` for development.
- Shared types: generate TypeScript types from Rust structs (via `ts-rs` or `specta` crate) to keep frontend/backend in sync.
- Design tokens: CSS custom properties for theming (dark default, potential light variant in V1).

## Revisit conditions

- If React's bundle size or runtime overhead becomes a measurable problem for the dense UI — consider Solid.js migration (API is similar enough for incremental migration).
- If Tauri 2.x introduces breaking changes that are costly to adapt — evaluate Tauri 3.x timeline or Wry (lower-level webview library Tauri is built on).
- If VS Code webview constraints conflict with the React app architecture — may need a separate lightweight build for the extension.
