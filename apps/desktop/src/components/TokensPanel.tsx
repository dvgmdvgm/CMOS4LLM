import { useAppStore } from "../store";

export function TokensPanel() {
  const { tokenStats, selectedProject } = useAppStore();

  if (!selectedProject) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-[12px]">
        Select a project
      </div>
    );
  }

  if (!tokenStats) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-[12px]">
        Loading token analytics...
      </div>
    );
  }

  const hasData = tokenStats.total_requests > 0;

  return (
    <div className="p-4 h-full flex flex-col">
      <h2 className="font-mono text-[11px] text-text-muted uppercase tracking-wider mb-3">
        Token Analytics — {selectedProject}
      </h2>

      {!hasData ? (
        <div className="flex-1 flex flex-col items-center justify-center text-center">
          <div className="text-[40px] font-mono text-text-muted mb-2">0</div>
          <div className="text-[12px] text-text-muted max-w-xs">
            No token data yet. Use CMOS with Claude to start tracking savings.
            Token analytics will appear here after MCP requests are processed.
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <MetricCard
              label="Total Requests"
              value={tokenStats.total_requests.toLocaleString()}
            />
            <MetricCard
              label="Tokens Saved"
              value={tokenStats.tokens_saved.toLocaleString()}
              highlight
            />
            <MetricCard
              label="Without CMOS"
              value={tokenStats.tokens_without_cmos.toLocaleString()}
              sub="baseline estimate"
            />
            <MetricCard
              label="With CMOS"
              value={tokenStats.tokens_with_cmos.toLocaleString()}
              sub="actual usage"
            />
          </div>

          <div className="p-4 bg-surface-2 rounded border border-border text-center">
            <div className="text-[10px] text-text-muted uppercase tracking-wider mb-1">
              Savings Ratio
            </div>
            <div className="text-3xl font-mono font-bold text-success">
              {tokenStats.savings_ratio.toFixed(1)}x
            </div>
            <div className="text-[11px] text-text-secondary mt-1">
              fewer tokens sent to cloud LLM
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function MetricCard({
  label,
  value,
  sub,
  highlight,
}: {
  label: string;
  value: string;
  sub?: string;
  highlight?: boolean;
}) {
  return (
    <div className="p-3 bg-surface-2 rounded border border-border">
      <div className="text-[10px] text-text-muted uppercase tracking-wider">
        {label}
      </div>
      <div
        className={`text-lg font-mono font-bold mt-1 ${highlight ? "text-success" : "text-text-primary"}`}
      >
        {value}
      </div>
      {sub && <div className="text-[10px] text-text-muted mt-0.5">{sub}</div>}
    </div>
  );
}
