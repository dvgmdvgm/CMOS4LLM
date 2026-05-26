# GUI Screen: Cognitive Trace Overlay

> Always-on floating panel (bottom-right). Shows recent CMOS activity and current state at a glance.

## Purpose

Persistent awareness of what CMOS is doing without switching screens. Quick access to pause, expand, or investigate.

## Layout

TODO: Define compact view (collapsed: 1-2 lines of status)
TODO: Define expanded view (recent activity feed, current inference state)
TODO: Define action buttons (open full inspector, pause CMOS, settings)

## Behavior

- Always visible regardless of active screen
- Updates in real-time via WebSocket
- Collapsible to minimal indicator

## Data Sources

- Context Hypervisor (current state)
- Observability (recent events stream)

## Scope

- [MVP M8](../03-scope/mvp.md) — one of 5 MVP GUI screens (always-on overlay)

## Open Questions

TODO: Z-index and positioning strategy across screens
TODO: Notification priority (what warrants attention vs silent log)
