use crate::embedding::EmbeddingClient;
use crate::error::RetrievalError;
use crate::scoring::{estimate_tokens, RelevanceScorer};
use crate::vector::{VectorIndex, VectorSearchResult};

use cmos_memory::l2l3::{EventStore, Layer, MemoryEvent};
use cmos_memory::l4::{Fact, ProjectMemory};

#[derive(Debug, Clone)]
pub struct HybridResult {
    pub id: String,
    pub content: String,
    pub layer: String,
    pub score: f64,
    pub token_estimate: usize,
}

#[derive(Debug, Clone)]
pub struct HybridConfig {
    pub vector_weight: f64,
    pub keyword_weight: f64,
    pub vector_candidates: usize,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.6,
            keyword_weight: 0.4,
            vector_candidates: 50,
        }
    }
}

pub struct HybridRetriever {
    config: HybridConfig,
    scorer: RelevanceScorer,
}

impl HybridRetriever {
    pub fn new(config: HybridConfig) -> Self {
        Self {
            config,
            scorer: RelevanceScorer::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn retrieve_l4(
        &self,
        query: &str,
        project_id: &str,
        vector_index: &VectorIndex,
        embedding_client: &EmbeddingClient,
        project_memory: &ProjectMemory,
        budget_tokens: usize,
    ) -> Result<Vec<HybridResult>, RetrievalError> {
        let query_embedding = embedding_client.embed_single(query).await?;
        self.retrieve_l4_with_embedding(query, project_id, vector_index, &query_embedding, project_memory, budget_tokens)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn retrieve_l4_with_embedding(
        &self,
        _query: &str,
        project_id: &str,
        vector_index: &VectorIndex,
        query_embedding: &[f32],
        project_memory: &ProjectMemory,
        budget_tokens: usize,
    ) -> Result<Vec<HybridResult>, RetrievalError> {
        let vector_results = vector_index
            .search(query_embedding, self.config.vector_candidates, Some("L4"))?;

        let all_facts = {
            let kinds = ["decision", "policy", "convention", "lesson", "constraint"];
            let mut facts = Vec::new();
            for kind in &kinds {
                facts.extend(project_memory.query_by_kind(project_id, kind)?);
            }
            facts
        };

        let mut scored = self.merge_l4_scores(&vector_results, &all_facts);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Self::apply_budget(scored, budget_tokens)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn retrieve_l3(
        &self,
        query: &str,
        project_id: &str,
        session_id: Option<&str>,
        vector_index: &VectorIndex,
        embedding_client: &EmbeddingClient,
        event_store: &EventStore,
        budget_tokens: usize,
    ) -> Result<Vec<HybridResult>, RetrievalError> {
        let query_embedding = embedding_client.embed_single(query).await?;
        self.retrieve_l3_with_embedding(query, project_id, session_id, vector_index, &query_embedding, event_store, budget_tokens)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn retrieve_l3_with_embedding(
        &self,
        _query: &str,
        project_id: &str,
        session_id: Option<&str>,
        vector_index: &VectorIndex,
        query_embedding: &[f32],
        event_store: &EventStore,
        budget_tokens: usize,
    ) -> Result<Vec<HybridResult>, RetrievalError> {
        let vector_results = vector_index
            .search(query_embedding, self.config.vector_candidates, Some("L3"))?;

        let events = if let Some(sid) = session_id {
            event_store.query_by_session(project_id, sid)?
        } else {
            event_store.query_by_layer(project_id, Layer::L3)?
        };

        let mut scored = self.merge_l3_scores(&vector_results, &events);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Self::apply_budget(scored, budget_tokens)
    }

    fn merge_l4_scores(
        &self,
        vector_results: &[VectorSearchResult],
        facts: &[Fact],
    ) -> Vec<HybridResult> {
        let max_distance = vector_results
            .iter()
            .map(|r| r.distance)
            .fold(0.0_f32, f32::max)
            .max(0.001);

        let mut results: Vec<HybridResult> = Vec::new();

        for fact in facts {
            let fact_id = fact.id.unwrap_or(0);
            let keyword_score = self.scorer.score_fact(fact);

            let vector_score = vector_results
                .iter()
                .find(|vr| vr.source_id == fact_id)
                .map(|vr| 1.0 - (vr.distance / max_distance) as f64)
                .unwrap_or(0.0);

            let combined = self.config.vector_weight * vector_score
                + self.config.keyword_weight * keyword_score;

            let content = format!("[{}] {}: {}", fact.kind, fact.label, fact.description);

            results.push(HybridResult {
                id: format!("l4-{}", fact_id),
                content,
                layer: "L4".to_string(),
                score: combined,
                token_estimate: estimate_tokens(&fact.description) + estimate_tokens(&fact.label) + 10,
            });
        }

        results
    }

    fn merge_l3_scores(
        &self,
        vector_results: &[VectorSearchResult],
        events: &[MemoryEvent],
    ) -> Vec<HybridResult> {
        let max_distance = vector_results
            .iter()
            .map(|r| r.distance)
            .fold(0.0_f32, f32::max)
            .max(0.001);

        let (newest_ts, oldest_ts) = timestamp_range(events);

        let mut results: Vec<HybridResult> = Vec::new();

        for event in events {
            let keyword_score = self.scorer.score_event(event, newest_ts, oldest_ts);

            let vector_score = vector_results
                .iter()
                .find(|vr| vr.source_id == event.id.unwrap_or(0))
                .map(|vr| 1.0 - (vr.distance / max_distance) as f64)
                .unwrap_or(0.0);

            let combined = self.config.vector_weight * vector_score
                + self.config.keyword_weight * keyword_score;

            let content = extract_event_content(event);

            results.push(HybridResult {
                id: format!("l3-{}", event.id.unwrap_or(0)),
                content: content.clone(),
                layer: "L3".to_string(),
                score: combined,
                token_estimate: estimate_tokens(&content),
            });
        }

        results
    }

    fn apply_budget(
        scored: Vec<HybridResult>,
        budget_tokens: usize,
    ) -> Result<Vec<HybridResult>, RetrievalError> {
        let mut results = Vec::new();
        let mut tokens_used = 0;

        for item in scored {
            if tokens_used + item.token_estimate > budget_tokens {
                break;
            }
            tokens_used += item.token_estimate;
            results.push(item);
        }

        Ok(results)
    }
}

fn extract_event_content(event: &MemoryEvent) -> String {
    if let Some(summary) = event.payload.get("summary").and_then(|v| v.as_str()) {
        summary.to_string()
    } else if let Some(content) = event.payload.get("content").and_then(|v| v.as_str()) {
        content.to_string()
    } else {
        event.payload.to_string()
    }
}

fn timestamp_range(events: &[MemoryEvent]) -> (f64, f64) {
    if events.is_empty() {
        return (0.0, 0.0);
    }
    let parse = |ts: &str| -> f64 {
        ts.replace(['-', ':', 'T', ' '], "")
            .chars()
            .take(14)
            .collect::<String>()
            .parse::<f64>()
            .unwrap_or(0.0)
    };
    let newest = events.iter().map(|e| parse(&e.timestamp)).fold(0.0_f64, f64::max);
    let oldest = events.iter().map(|e| parse(&e.timestamp)).fold(f64::MAX, f64::min);
    (newest, oldest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_config_default() {
        let config = HybridConfig::default();
        assert!((config.vector_weight + config.keyword_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_budget() {
        let items = vec![
            HybridResult {
                id: "1".into(),
                content: "short".into(),
                layer: "L4".into(),
                score: 0.9,
                token_estimate: 100,
            },
            HybridResult {
                id: "2".into(),
                content: "medium".into(),
                layer: "L4".into(),
                score: 0.8,
                token_estimate: 200,
            },
            HybridResult {
                id: "3".into(),
                content: "long".into(),
                layer: "L4".into(),
                score: 0.7,
                token_estimate: 300,
            },
        ];

        let result = HybridRetriever::apply_budget(items, 250).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "1");
    }

    #[test]
    fn test_merge_l4_scores_no_vector_match() {
        let retriever = HybridRetriever::new(HybridConfig::default());
        let vector_results = vec![];
        let facts = vec![Fact {
            id: Some(1),
            project_id: "proj".into(),
            kind: "decision".into(),
            label: "Use Postgres".into(),
            description: "Chose Postgres for ACID".into(),
            source: cmos_memory::l4::FactSource::Bootstrap,
            confidence: 0.9,
            access_count: 3,
        }];

        let results = retriever.merge_l4_scores(&vector_results, &facts);
        assert_eq!(results.len(), 1);
        // With no vector match, score comes entirely from keyword scoring
        assert!(results[0].score > 0.0);
    }
}
