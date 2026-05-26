# GUI Screen: Dashboard

> Project health-check in one screen. Entry point after launch.

## Purpose

Single-pane overview: token economy, memory health, drift status, recent episodes, active policies.

## Layout

TODO: Define panel grid (likely 2×3 or 3×2)
TODO: Define key metrics (token reduction ratio, memory item count, drift violations open, active policies)

## Data Sources

- Token Analytics (aggregated from Observability)
- Memory Layers (item counts, promotion activity)
- Policy Engine (active policies, recent violations)
- L3 Episodic (recent episodes)

## Interactions

TODO: Click-through to detailed screens
TODO: Time range selector (24h / 7d / 30d)

## Scope

- [MVP M8](../03-scope/mvp.md) — one of 5 MVP GUI screens
- Design: [ADR-006](../02-decisions/ADR-006-gui-density-devtools-style.md) — high density

## Open Questions

TODO: Which metrics are most actionable for daily use?
