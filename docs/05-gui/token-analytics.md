# GUI Screen: Token Analytics

> Visualize cloud token savings, cost breakdown, and reduction techniques in action.

## Purpose

Prove CMOS value quantitatively. Show headline metrics, per-technique breakdown, daily trends vs simulated baseline, and top-cost queries.

## Key Panels

TODO: Define headline panel (cloud used / saved / reduction ratio)
TODO: Define savings breakdown by technique (stacked bar or table)
TODO: Define daily timeseries (actual vs simulated baseline without CMOS)
TODO: Define top-cost queries list (drill-down to InferenceRecord)

## Data Sources

- Observability (InferenceRecords with token counts)
- Simulated baseline (counterfactual: what would have been sent without CMOS)

## Scope

- [MVP M8](../03-scope/mvp.md) — one of 5 MVP GUI screens

## Open Questions

TODO: Baseline simulation methodology (replay without filtering?)
TODO: Cost model (per-provider pricing integration?)
