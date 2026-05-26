use cmos_memory::l2l3::MemoryEvent;
use cmos_memory::l4::Fact;

#[derive(Debug, Clone)]
pub struct ScoredItem {
    pub content: String,
    pub score: f64,
    pub source: ItemSource,
    pub token_estimate: usize,
}

#[derive(Debug, Clone)]
pub enum ItemSource {
    L3Event { event_id: i64 },
    L4Fact { fact_id: i64 },
    L1Slot { slot_id: String },
}

pub struct RelevanceScorer {
    pub recency_weight: f64,
    pub importance_weight: f64,
    pub access_weight: f64,
}

impl Default for RelevanceScorer {
    fn default() -> Self {
        Self {
            recency_weight: 0.4,
            importance_weight: 0.4,
            access_weight: 0.2,
        }
    }
}

impl RelevanceScorer {
    pub fn score_event(&self, event: &MemoryEvent, newest_ts: f64, oldest_ts: f64) -> f64 {
        let recency = if newest_ts > oldest_ts {
            let event_ts = parse_timestamp_approx(&event.timestamp);
            (event_ts - oldest_ts) / (newest_ts - oldest_ts)
        } else {
            1.0
        };

        let importance = event.importance as f64;
        let access = (event.access_count as f64).ln_1p() / 5.0_f64.ln_1p();

        self.recency_weight * recency
            + self.importance_weight * importance
            + self.access_weight * access.min(1.0)
    }

    pub fn score_fact(&self, fact: &Fact) -> f64 {
        let confidence = fact.confidence as f64;
        let access = (fact.access_count as f64).ln_1p() / 5.0_f64.ln_1p();

        self.importance_weight * confidence + self.access_weight * access.min(1.0) + self.recency_weight * 0.5
    }
}

fn parse_timestamp_approx(ts: &str) -> f64 {
    ts.replace(['-', ':', 'T', ' '], "")
        .chars()
        .take(14)
        .collect::<String>()
        .parse::<f64>()
        .unwrap_or(0.0)
}

pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4 + 1
}
