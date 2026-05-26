import { useAppStore } from "../store";

export function FactsPanel() {
  const { facts, selectedProject } = useAppStore();

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
        L4 Facts — {selectedProject}
      </h2>

      <div className="flex-1 overflow-y-auto">
        {facts.length === 0 ? (
          <div className="text-[12px] text-text-muted text-center py-8">
            No facts stored yet.
          </div>
        ) : (
          <table className="w-full text-[11px]">
            <thead>
              <tr className="text-left text-text-muted border-b border-border">
                <th className="pb-1 pr-2 font-normal">Kind</th>
                <th className="pb-1 pr-2 font-normal">Label</th>
                <th className="pb-1 pr-2 font-normal">Confidence</th>
                <th className="pb-1 font-normal">Accesses</th>
              </tr>
            </thead>
            <tbody>
              {facts.map((f) => (
                <tr
                  key={f.id}
                  className="border-b border-border/50 hover:bg-surface-2 transition-colors"
                >
                  <td className="py-1 pr-2">
                    <KindBadge kind={f.kind} />
                  </td>
                  <td className="py-1 pr-2 text-text-primary font-mono">
                    {f.label}
                  </td>
                  <td className="py-1 pr-2 text-text-secondary">
                    {(f.confidence * 100).toFixed(0)}%
                  </td>
                  <td className="py-1 text-text-muted">{f.access_count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

function KindBadge({ kind }: { kind: string }) {
  const colors: Record<string, string> = {
    decision: "text-accent bg-accent/10",
    policy: "text-warning bg-warning/10",
    convention: "text-success bg-success/10",
    lesson: "text-text-primary bg-surface-3",
    constraint: "text-error bg-error/10",
  };

  return (
    <span
      className={`px-1.5 py-0.5 rounded text-[10px] font-mono ${colors[kind] ?? "text-text-muted bg-surface-3"}`}
    >
      {kind}
    </span>
  );
}
