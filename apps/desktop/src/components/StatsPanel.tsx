import { useAppStore, type MemoryStats as Stats } from "../store";

export function StatsPanel() {
  const { stats, selectedProject } = useAppStore();

  if (!selectedProject) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-[12px]">
        Select a project to view stats
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-[12px]">
        Loading...
      </div>
    );
  }

  return (
    <div className="p-4">
      <h2 className="font-mono text-[11px] text-text-muted uppercase tracking-wider mb-3">
        Memory Stats — {selectedProject}
      </h2>

      <div className="grid grid-cols-3 gap-3">
        <StatCard
          label="L1 Working"
          value={stats.l1_slots}
          sub={`${stats.l1_tokens.toLocaleString()} tokens`}
          color="accent"
        />
        <StatCard
          label="L2 Episodic"
          value={stats.l2_count}
          sub="events"
          color="success"
        />
        <StatCard
          label="L3 Consolidated"
          value={stats.l3_count}
          sub="events"
          color="warning"
        />
        <StatCard
          label="L4 Semantic"
          value={stats.l4_count}
          sub="facts"
          color="accent"
        />
        <StatCard
          label="Vector Index"
          value={stats.vector_count}
          sub="embeddings"
          color="success"
        />
        <StatCard
          label="Total Items"
          value={stats.l1_slots + stats.l2_count + stats.l3_count + stats.l4_count}
          sub="across all layers"
          color="text-primary"
        />
      </div>

      <div className="mt-4 p-3 bg-surface-2 rounded border border-border">
        <div className="text-[10px] text-text-muted uppercase tracking-wider mb-2">
          Layer Distribution
        </div>
        <LayerBar stats={stats} />
      </div>
    </div>
  );
}

function StatCard({
  label,
  value,
  sub,
  color,
}: {
  label: string;
  value: number;
  sub: string;
  color: string;
}) {
  return (
    <div className="p-3 bg-surface-2 rounded border border-border">
      <div className="text-[10px] text-text-muted uppercase tracking-wider">
        {label}
      </div>
      <div className={`text-xl font-mono font-bold mt-1 text-${color}`}>
        {value.toLocaleString()}
      </div>
      <div className="text-[10px] text-text-muted mt-0.5">{sub}</div>
    </div>
  );
}

function LayerBar({ stats }: { stats: Stats }) {
  const total =
    stats.l1_slots + stats.l2_count + stats.l3_count + stats.l4_count || 1;
  const segments = [
    { label: "L1", count: stats.l1_slots, color: "bg-accent" },
    { label: "L2", count: stats.l2_count, color: "bg-success" },
    { label: "L3", count: stats.l3_count, color: "bg-warning" },
    { label: "L4", count: stats.l4_count, color: "bg-accent-dim" },
  ];

  return (
    <div>
      <div className="flex h-2 rounded overflow-hidden gap-px">
        {segments.map((s) => (
          <div
            key={s.label}
            className={`${s.color} transition-all`}
            style={{ width: `${(s.count / total) * 100}%` }}
          />
        ))}
      </div>
      <div className="flex justify-between mt-1.5">
        {segments.map((s) => (
          <div key={s.label} className="text-[10px] text-text-muted">
            <span className={`inline-block w-1.5 h-1.5 rounded-sm ${s.color} mr-1`} />
            {s.label}: {s.count}
          </div>
        ))}
      </div>
    </div>
  );
}
