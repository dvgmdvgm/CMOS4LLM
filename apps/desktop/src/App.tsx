import { useEffect } from "react";
import { useAppStore } from "./store";
import { Sidebar } from "./components/Sidebar";
import { StatsPanel } from "./components/StatsPanel";
import { FactsPanel } from "./components/FactsPanel";
import { EventsPanel } from "./components/EventsPanel";
import { TokensPanel } from "./components/TokensPanel";

function App() {
  const { activeTab, setActiveTab, fetchVersion, fetchProjects, fetchTokenStats } =
    useAppStore();

  useEffect(() => {
    fetchVersion();
    fetchProjects();
    fetchTokenStats();
  }, []);

  return (
    <div className="flex h-screen">
      <Sidebar />
      <main className="flex-1 flex flex-col min-w-0">
        <TabBar activeTab={activeTab} onTabChange={setActiveTab} />
        <div className="flex-1 overflow-hidden">
          {activeTab === "stats" && <StatsPanel />}
          {activeTab === "facts" && <FactsPanel />}
          {activeTab === "events" && <EventsPanel />}
          {activeTab === "tokens" && <TokensPanel />}
        </div>
      </main>
    </div>
  );
}

const tabs = [
  { id: "stats" as const, label: "Stats" },
  { id: "facts" as const, label: "L4 Facts" },
  { id: "events" as const, label: "L2/L3 Events" },
  { id: "tokens" as const, label: "Token Analytics" },
];

function TabBar({
  activeTab,
  onTabChange,
}: {
  activeTab: string;
  onTabChange: (tab: "stats" | "facts" | "events" | "tokens") => void;
}) {
  return (
    <div className="flex border-b border-border bg-surface-1">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`px-4 py-2 text-[12px] font-mono border-b-2 transition-colors ${
            activeTab === tab.id
              ? "border-accent text-text-primary"
              : "border-transparent text-text-secondary hover:text-text-primary"
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}

export default App;
