import { useAppStore } from "../store";

export function Sidebar() {
  const { projects, selectedProject, selectProject, version } = useAppStore();

  return (
    <aside className="w-52 h-screen bg-surface-1 border-r border-border flex flex-col">
      <div className="px-3 py-2 border-b border-border">
        <div className="font-mono text-[11px] text-text-muted uppercase tracking-wider">
          CMOS
        </div>
        <div className="text-[10px] text-text-muted mt-0.5">{version}</div>
      </div>

      <div className="px-3 py-2 border-b border-border">
        <div className="text-[10px] text-text-muted uppercase tracking-wider mb-1">
          Projects
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {projects.length === 0 && (
          <div className="px-3 py-4 text-[11px] text-text-muted text-center">
            No projects found.
            <br />
            Run bootstrap first.
          </div>
        )}
        {projects.map((p) => (
          <button
            key={p.id}
            onClick={() => selectProject(p.id)}
            className={`w-full text-left px-3 py-1.5 text-[12px] border-l-2 transition-colors ${
              selectedProject === p.id
                ? "border-accent bg-surface-2 text-text-primary"
                : "border-transparent text-text-secondary hover:bg-surface-2 hover:text-text-primary"
            }`}
          >
            <div className="font-mono truncate">{p.id}</div>
            <div className="text-[10px] text-text-muted mt-0.5">
              {p.l4_facts} facts &middot; {p.l2_events + p.l3_events} events
            </div>
          </button>
        ))}
      </div>

      <div className="px-3 py-2 border-t border-border text-[10px] text-text-muted">
        Memory System v1
      </div>
    </aside>
  );
}
