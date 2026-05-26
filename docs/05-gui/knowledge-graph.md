# GUI Screen: Knowledge Graph Viewer

> Interactive graph visualization of project ontology and relationships.

## Purpose

Visualize domain entities, code symbols, decisions, and their relationships. Multiple view modes for different exploration needs.

## View Modes

TODO: Define modes:
- Domain ontology (entities + relationships from models)
- Code symbols (modules, classes, functions, dependencies)
- Decisions evolution (ADRs, policies, their connections)
- Module dependencies (import graph)
- Combined (all layers, filterable)

## Tech

- Cytoscape.js for rendering
- Force-directed + hierarchical layouts
- Filtering by node type, edge type, time range

## Data Sources

- L4 Project memory (graph DB)

## Scope

- **Not in MVP** — V1 feature
- Design: [ADR-006](../02-decisions/ADR-006-gui-density-devtools-style.md)

## Open Questions

TODO: Performance with large graphs (10K+ nodes)
TODO: Incremental layout updates vs full recompute
