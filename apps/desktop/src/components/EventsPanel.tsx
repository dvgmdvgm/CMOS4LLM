import { useAppStore } from "../store";

export function EventsPanel() {
  const { events, selectedProject } = useAppStore();

  if (!selectedProject) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-[12px]">
        Select a project
      </div>
    );
  }

  return (
    <div className="p-4 h-full flex flex-col">
      <h2 className="font-mono text-[11px] text-text-muted uppercase tracking-wider mb-3">
        L2/L3 Events — {selectedProject}
      </h2>

      <div className="flex-1 overflow-y-auto">
        {events.length === 0 ? (
          <div className="text-[12px] text-text-muted text-center py-8">
            No events recorded yet.
          </div>
        ) : (
          <div className="space-y-1">
            {events.map((e) => (
              <div
                key={e.id}
                className="flex items-start gap-2 p-2 rounded bg-surface-2 border border-border/50 hover:border-border transition-colors"
              >
                <LayerBadge layer={e.layer} />
                <TypeBadge type={e.event_type} />
                <div className="flex-1 min-w-0">
                  {e.entity_id && (
                    <span className="font-mono text-[11px] text-text-primary">
                      {e.entity_id}
                    </span>
                  )}
                  <div className="text-[10px] text-text-muted truncate mt-0.5">
                    {typeof e.payload === "object"
                      ? JSON.stringify(e.payload).slice(0, 80)
                      : String(e.payload)}
                  </div>
                </div>
                <div className="text-[10px] text-text-muted whitespace-nowrap">
                  {e.timestamp.slice(0, 16).replace("T", " ")}
                </div>
                <div className="text-[10px] text-text-muted w-8 text-right">
                  {e.importance.toFixed(1)}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function LayerBadge({ layer }: { layer: string }) {
  const color = layer === "L3" ? "text-warning bg-warning/10" : "text-success bg-success/10";
  return (
    <span className={`px-1 py-0.5 rounded text-[9px] font-mono font-bold ${color}`}>
      {layer}
    </span>
  );
}

function TypeBadge({ type }: { type: string }) {
  return (
    <span className="px-1 py-0.5 rounded text-[9px] font-mono text-text-secondary bg-surface-3">
      {type}
    </span>
  );
}
