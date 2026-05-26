use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySlot {
    pub id: String,
    pub content: String,
    pub token_estimate: usize,
    pub priority: SlotPriority,
    pub inserted_at: u64,
    pub access_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlotPriority {
    Scratch = 0,
    Context = 1,
    Policy = 2,
    System = 3,
}

#[derive(Debug, Clone)]
pub struct WorkingMemoryConfig {
    pub max_tokens: usize,
    pub eviction_headroom: usize,
}

impl Default for WorkingMemoryConfig {
    fn default() -> Self {
        Self {
            max_tokens: 16_000,
            eviction_headroom: 2_000,
        }
    }
}

#[derive(Clone)]
pub struct WorkingMemory {
    inner: Arc<RwLock<WorkingMemoryInner>>,
}

struct WorkingMemoryInner {
    slots: VecDeque<MemorySlot>,
    total_tokens: usize,
    config: WorkingMemoryConfig,
    clock: u64,
}

impl WorkingMemory {
    pub fn new(config: WorkingMemoryConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(WorkingMemoryInner {
                slots: VecDeque::new(),
                total_tokens: 0,
                config,
                clock: 0,
            })),
        }
    }

    pub fn insert(&self, id: impl Into<String>, content: impl Into<String>, priority: SlotPriority) -> Option<Vec<MemorySlot>> {
        let id = id.into();
        let content = content.into();
        let token_estimate = estimate_tokens(&content);

        let mut inner = self.inner.write().unwrap();
        inner.clock += 1;

        // Remove existing slot with same id
        if let Some(pos) = inner.slots.iter().position(|s| s.id == id) {
            let old = inner.slots.remove(pos).unwrap();
            inner.total_tokens -= old.token_estimate;
        }

        let slot = MemorySlot {
            id,
            content,
            token_estimate,
            priority,
            inserted_at: inner.clock,
            access_count: 0,
        };

        let evicted = inner.evict_to_fit(token_estimate);
        inner.total_tokens += token_estimate;
        inner.slots.push_back(slot);

        if evicted.is_empty() { None } else { Some(evicted) }
    }

    pub fn get(&self, id: &str) -> Option<MemorySlot> {
        let mut inner = self.inner.write().unwrap();
        if let Some(slot) = inner.slots.iter_mut().find(|s| s.id == id) {
            slot.access_count += 1;
            Some(slot.clone())
        } else {
            None
        }
    }

    pub fn remove(&self, id: &str) -> Option<MemorySlot> {
        let mut inner = self.inner.write().unwrap();
        if let Some(pos) = inner.slots.iter().position(|s| s.id == id) {
            let slot = inner.slots.remove(pos).unwrap();
            inner.total_tokens -= slot.token_estimate;
            Some(slot)
        } else {
            None
        }
    }

    pub fn assemble(&self) -> String {
        let inner = self.inner.read().unwrap();
        let mut sorted: Vec<&MemorySlot> = inner.slots.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.inserted_at.cmp(&a.inserted_at)));
        sorted.iter().map(|s| s.content.as_str()).collect::<Vec<_>>().join("\n\n")
    }

    pub fn assemble_within_budget(&self, token_budget: usize) -> String {
        let inner = self.inner.read().unwrap();
        let mut sorted: Vec<&MemorySlot> = inner.slots.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.inserted_at.cmp(&a.inserted_at)));

        let mut result = Vec::new();
        let mut used = 0;
        for slot in sorted {
            if used + slot.token_estimate > token_budget {
                continue;
            }
            result.push(slot.content.as_str());
            used += slot.token_estimate;
        }
        result.join("\n\n")
    }

    pub fn total_tokens(&self) -> usize {
        self.inner.read().unwrap().total_tokens
    }

    pub fn slot_count(&self) -> usize {
        self.inner.read().unwrap().slots.len()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.slots.clear();
        inner.total_tokens = 0;
    }
}

impl WorkingMemoryInner {
    fn evict_to_fit(&mut self, needed: usize) -> Vec<MemorySlot> {
        let mut evicted = Vec::new();
        let target = self.config.max_tokens.saturating_sub(self.config.eviction_headroom);

        while self.total_tokens + needed > target && !self.slots.is_empty() {
            let victim_idx = self.find_eviction_candidate();
            let victim = self.slots.remove(victim_idx).unwrap();
            self.total_tokens -= victim.token_estimate;
            evicted.push(victim);
        }
        evicted
    }

    fn find_eviction_candidate(&self) -> usize {
        let mut best_idx = 0;
        let mut best_score = u64::MAX;

        for (i, slot) in self.slots.iter().enumerate() {
            let priority_weight = (slot.priority as u64) * 10_000;
            let recency_weight = slot.inserted_at;
            let access_weight = slot.access_count as u64 * 1_000;
            let score = priority_weight + recency_weight + access_weight;

            if score < best_score {
                best_score = score;
                best_idx = i;
            }
        }
        best_idx
    }
}

fn estimate_tokens(text: &str) -> usize {
    // ~4 chars per token for English text
    text.len().div_ceil(4)
}
