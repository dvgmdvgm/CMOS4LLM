import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface MemoryStats {
  l1_slots: number;
  l1_tokens: number;
  l2_count: number;
  l3_count: number;
  l4_count: number;
  vector_count: number;
}

export interface ProjectInfo {
  id: string;
  l4_facts: number;
  l2_events: number;
  l3_events: number;
}

export interface FactEntry {
  id: number;
  kind: string;
  label: string;
  description: string;
  confidence: number;
  access_count: number;
}

export interface EventEntry {
  id: number;
  event_type: string;
  layer: string;
  timestamp: string;
  entity_id: string | null;
  importance: number;
  payload: unknown;
}

export interface TokenStats {
  total_requests: number;
  tokens_saved: number;
  tokens_without_cmos: number;
  tokens_with_cmos: number;
  savings_ratio: number;
}

type Tab = "stats" | "facts" | "events" | "tokens";

interface AppStore {
  version: string;
  projects: ProjectInfo[];
  selectedProject: string | null;
  stats: MemoryStats | null;
  facts: FactEntry[];
  events: EventEntry[];
  tokenStats: TokenStats | null;
  activeTab: Tab;
  loading: boolean;

  setActiveTab: (tab: Tab) => void;
  selectProject: (id: string) => void;
  fetchVersion: () => Promise<void>;
  fetchProjects: () => Promise<void>;
  fetchStats: () => Promise<void>;
  fetchFacts: (kind?: string) => Promise<void>;
  fetchEvents: (layer?: string) => Promise<void>;
  fetchTokenStats: () => Promise<void>;
}

export const useAppStore = create<AppStore>((set, get) => ({
  version: "",
  projects: [],
  selectedProject: null,
  stats: null,
  facts: [],
  events: [],
  tokenStats: null,
  activeTab: "stats",
  loading: false,

  setActiveTab: (tab) => set({ activeTab: tab }),

  selectProject: (id) => {
    set({ selectedProject: id });
    get().fetchStats();
    get().fetchFacts();
    get().fetchEvents();
  },

  fetchVersion: async () => {
    const version = await invoke<string>("get_version");
    set({ version });
  },

  fetchProjects: async () => {
    try {
      const projects = await invoke<ProjectInfo[]>("list_projects");
      set({ projects });
      if (projects.length > 0 && !get().selectedProject) {
        get().selectProject(projects[0].id);
      }
    } catch {
      set({ projects: [] });
    }
  },

  fetchStats: async () => {
    const pid = get().selectedProject;
    if (!pid) return;
    try {
      const stats = await invoke<MemoryStats>("get_memory_stats", { projectId: pid });
      set({ stats });
    } catch {
      set({ stats: null });
    }
  },

  fetchFacts: async (kind?: string) => {
    const pid = get().selectedProject;
    if (!pid) return;
    try {
      const facts = await invoke<FactEntry[]>("get_facts", {
        projectId: pid,
        kind: kind ?? null,
        limit: 100,
      });
      set({ facts });
    } catch {
      set({ facts: [] });
    }
  },

  fetchEvents: async (layer?: string) => {
    const pid = get().selectedProject;
    if (!pid) return;
    try {
      const events = await invoke<EventEntry[]>("get_events", {
        projectId: pid,
        layer: layer ?? null,
        limit: 100,
      });
      set({ events });
    } catch {
      set({ events: [] });
    }
  },

  fetchTokenStats: async () => {
    try {
      const tokenStats = await invoke<TokenStats>("get_token_stats");
      set({ tokenStats });
    } catch {
      set({ tokenStats: null });
    }
  },
}));
