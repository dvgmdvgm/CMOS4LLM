# ADR-006: GUI density — DevTools / Datadog style, not Linear / Notion style

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner
- **Related:** ADR-007 (Tauri hybrid shell), ADR-008 (counterfactual mode)

## Context

When we designed the Cognitive Console, three density options were on the table:

- **Minimalist** (Linear / Notion-style) — large whitespace, big touch-targets, beautiful but information-sparse.
- **Medium** (GitHub / GitLab) — a balance, suitable for collaborative product UIs.
- **High density** (Datadog / Grafana / Wireshark / Charles Proxy / browser DevTools) — every pixel carries information, monospace headings, charts dense, tables tight.

CMOS is an **introspection tool for power users**. The screens that matter most (Live Inference Inspector, Memory Browser, Drift Monitor, Token Analytics) are dashboards that succeed or fail by how much information they convey per glance.

Owner's choice: high density.

## Decision

**Cognitive Console adopts a high-density, DevTools-style information design.**

Concretely:

- Default theme: dark, monospace headings, condensed sans-serif body.
- Tables and lists prefer many columns over generous row heights. Truncation with hover-to-expand is acceptable.
- Charts default to a "small multiples" layout where applicable (multiple sparklines beat one big chart).
- Scrolling is fine; whitespace for the sake of whitespace is not.
- Side-by-side layouts (split panes) are first-class. Single-column reading layouts are reserved for documentation views.
- Real-time data uses tight color coding (status dots, severity strips) rather than verbose labels.

We are explicitly **not** designing for executive demos or non-technical viewers as the default surface (a separate "manager view" is a V2 feature — see ROADMAP).

## Rationale

- **The user is a developer doing development.** Information per square inch matters. A screen that requires three scrolls to see the same data Wireshark fits in one isn't doing its job.
- **CMOS competes with reading raw logs.** If our GUI is less dense than `tail -f` plus `grep`, developers will use the logs.
- **High density preserves the cognitive trace.** When debugging "why did the model choose this," you want all candidates on screen at once, not paginated.
- **Owner explicitly chose this** in the GUI design discussion ("делаем плотно, как ты и рекомендуешь").

## Consequences

### Positive
- More information visible at once → faster debugging.
- Skews the audience to power users (which is the V1 intended audience anyway).
- Aligns with the "observability is first-class" principle of the charter.

### Negative
- New users will feel overwhelmed at first. Onboarding tooltips and a "guided tour" mode are mitigations.
- Smaller laptop screens (13" sub-1080p) will be cramped. Mitigation: collapsible side rails, but no compromise on density of the main pane.
- Accessibility (visual): high-density UIs are harder for users with low vision. Mitigation: respect OS-level zoom, provide a "comfortable" density toggle (≤V1), but the *default* stays dense.

### Neutral / unknowns
- Whether to ship the "comfortable" density toggle in MVP or V1 — TBD; bias is V1.
- Long-term, a manager / overview surface (V2) will be lower density by design.

## Alternatives considered

- **Linear / Notion style:** rejected — beautiful but does not serve the introspection use case.
- **GitHub style (medium):** considered. Reasonable for a general developer tool, but our screens that matter most (Live Inference Inspector, Memory Browser) need more than medium can fit.
- **Configurable density default:** rejected for MVP — first impressions matter, and "configurable" means "everyone gets the wrong default until they fix it."

## Implementation notes

- Design tokens: spacing scale starts at 4px (not 8); typography sizes go down to 11–12px for table content; line-height tight (1.25–1.4).
- Component library: avoid heavyweight UI kits (Material, Chakra) that bake in generous spacing. Prefer Radix primitives + Tailwind with a custom dense scale, or shadcn/ui with custom spacing overrides.
- Charts: uPlot for timeseries (intentional — handles thousands of points without lag); Recharts for general (with custom tight margins).
- Tables: virtualized (TanStack Table) — required for Memory Browser, Episodes Browser, Drift Monitor.

## Revisit conditions

- If real users (V1+) consistently report "too dense to use," ship a "comfortable" density profile and re-evaluate.
- If a manager/exec persona becomes important pre-V2, design a separate surface — do not relax this default.
