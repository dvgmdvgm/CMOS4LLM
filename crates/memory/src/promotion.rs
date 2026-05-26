use crate::l2l3::{EventStore, EventStoreError, EventType, Layer, MemoryEvent};
use crate::l4::{Fact, FactSource, ProjectMemory, ProjectMemoryError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromotionError {
    #[error("event store error: {0}")]
    EventStore(#[from] EventStoreError),
    #[error("project memory error: {0}")]
    ProjectMemory(#[from] ProjectMemoryError),
}

#[derive(Debug, Clone)]
pub struct PromotionConfig {
    pub l2_to_l3_min_access: u32,
    pub l2_to_l3_min_importance: f32,
    pub l3_to_l4_min_access: u32,
    pub l3_to_l4_min_importance: f32,
    pub l3_to_l4_event_types: Vec<EventType>,
}

impl Default for PromotionConfig {
    fn default() -> Self {
        Self {
            l2_to_l3_min_access: 3,
            l2_to_l3_min_importance: 0.6,
            l3_to_l4_min_access: 5,
            l3_to_l4_min_importance: 0.8,
            l3_to_l4_event_types: vec![
                EventType::Decision,
                EventType::Lesson,
                EventType::RejectedApproach,
            ],
        }
    }
}

pub struct PromotionEngine {
    config: PromotionConfig,
}

impl PromotionEngine {
    pub fn new(config: PromotionConfig) -> Self {
        Self { config }
    }

    pub fn run_l2_to_l3(
        &self,
        store: &EventStore,
        project_id: &str,
    ) -> Result<Vec<i64>, PromotionError> {
        let candidates = store.candidates_for_promotion(
            project_id,
            self.config.l2_to_l3_min_access,
            self.config.l2_to_l3_min_importance,
        )?;

        let mut promoted = Vec::new();
        for event in &candidates {
            if let Some(id) = event.id {
                store.promote_to_l3(id)?;
                promoted.push(id);
            }
        }
        Ok(promoted)
    }

    pub fn run_l3_to_l4(
        &self,
        store: &EventStore,
        project_memory: &ProjectMemory,
        project_id: &str,
    ) -> Result<Vec<i64>, PromotionError> {
        let l3_events = store.query_by_layer(project_id, Layer::L3)?;

        let mut promoted = Vec::new();
        for event in &l3_events {
            if !self.qualifies_for_l4(event) {
                continue;
            }

            let fact = self.event_to_fact(event, project_id);
            project_memory.insert_fact(&fact)?;

            if let Some(id) = event.id {
                promoted.push(id);
            }
        }
        Ok(promoted)
    }

    fn qualifies_for_l4(&self, event: &MemoryEvent) -> bool {
        event.access_count >= self.config.l3_to_l4_min_access
            && event.importance >= self.config.l3_to_l4_min_importance
            && self.config.l3_to_l4_event_types.contains(&event.event_type)
    }

    fn event_to_fact(&self, event: &MemoryEvent, project_id: &str) -> Fact {
        let kind = match event.event_type {
            EventType::Decision => "decision",
            EventType::Lesson => "lesson",
            EventType::RejectedApproach => "rejected_approach",
            _ => "fact",
        };

        let label = event.payload.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .to_string();

        let description = event.payload.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Fact {
            id: None,
            project_id: project_id.to_string(),
            kind: kind.to_string(),
            label,
            description,
            source: FactSource::Promotion {
                from_event_id: event.id.unwrap_or(0),
            },
            confidence: event.importance,
            access_count: event.access_count,
        }
    }
}
