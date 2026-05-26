use serde::{Deserialize, Serialize};

use cmos_memory::l1::WorkingMemory;
use cmos_memory::l2l3::{EventStore, EventType, Layer, MemoryEvent};
use cmos_memory::l4::{Fact, ProjectMemory};

use crate::embedding::EmbeddingClient;
use crate::error::RetrievalError;
use crate::hybrid::{HybridConfig, HybridRetriever};
use crate::scoring::{estimate_tokens, RelevanceScorer};
use crate::vector::VectorIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuery {
    pub project_id: String,
    pub task_description: String,
    pub session_id: Option<String>,
    pub max_tokens: usize,
    pub include_l1: bool,
    pub include_l3: bool,
    pub include_l4: bool,
}

impl ContextQuery {
    pub fn new(project_id: &str, task: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            task_description: task.to_string(),
            session_id: None,
            max_tokens: 32_000,
            include_l1: true,
            include_l3: true,
            include_l4: true,
        }
    }

    pub fn with_budget(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    pub label: String,
    pub content: String,
    pub token_count: usize,
    pub source_layer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    pub sections: Vec<ContextSection>,
    pub total_tokens: usize,
    pub budget: usize,
    pub items_considered: usize,
    pub items_included: usize,
}

impl AssembledContext {
    pub fn render(&self) -> String {
        let mut output = String::new();
        for section in &self.sections {
            output.push_str(&format!("## {}\n\n", section.label));
            output.push_str(&section.content);
            output.push_str("\n\n");
        }
        output
    }

    pub fn render_with_header(&self, task: &str) -> String {
        let mut output = format!(
            "# Context for task\n\n> {}\n\n---\n\n",
            task
        );
        output.push_str(&self.render());
        output.push_str(&format!(
            "---\n_Context: {} tokens used / {} budget, {} items from {} considered_\n",
            self.total_tokens, self.budget, self.items_included, self.items_considered
        ));
        output
    }
}

pub struct ContextAssembler {
    scorer: RelevanceScorer,
    l4_budget_ratio: f64,
    l3_budget_ratio: f64,
    l1_budget_ratio: f64,
}

impl Default for ContextAssembler {
    fn default() -> Self {
        Self {
            scorer: RelevanceScorer::default(),
            l4_budget_ratio: 0.4,
            l3_budget_ratio: 0.4,
            l1_budget_ratio: 0.2,
        }
    }
}

impl ContextAssembler {
    pub fn new(scorer: RelevanceScorer) -> Self {
        Self {
            scorer,
            ..Default::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn assemble_hybrid(
        &self,
        query: &ContextQuery,
        l1: Option<&WorkingMemory>,
        event_store: Option<&EventStore>,
        project_memory: Option<&ProjectMemory>,
        vector_index: Option<&VectorIndex>,
        embedding_client: Option<&EmbeddingClient>,
    ) -> Result<AssembledContext, RetrievalError> {
        let use_hybrid = vector_index.is_some() && embedding_client.is_some();

        if !use_hybrid {
            return self.assemble(query, l1, event_store, project_memory);
        }

        let vi = vector_index.unwrap();
        let ec = embedding_client.unwrap();
        let hybrid = HybridRetriever::new(HybridConfig::default());

        let mut sections = Vec::new();
        let mut total_tokens = 0;
        let mut items_considered = 0;
        let mut items_included = 0;

        let l4_budget = (query.max_tokens as f64 * self.l4_budget_ratio) as usize;
        let l3_budget = (query.max_tokens as f64 * self.l3_budget_ratio) as usize;
        let l1_budget = (query.max_tokens as f64 * self.l1_budget_ratio) as usize;

        if query.include_l4 && let Some(pm) = project_memory {
            let results = hybrid
                .retrieve_l4(&query.task_description, &query.project_id, vi, ec, pm, l4_budget)
                .await?;
            items_considered += results.len();
            if !results.is_empty() {
                let mut content = String::new();
                let mut tokens_used = 0;
                for r in &results {
                    let line = format!("- **{}**\n", r.content);
                    let line_tokens = estimate_tokens(&line);
                    content.push_str(&line);
                    tokens_used += line_tokens;
                }
                items_included += results.len();
                total_tokens += tokens_used;
                sections.push(ContextSection {
                    label: "Project Knowledge (L4, hybrid)".to_string(),
                    content,
                    token_count: tokens_used,
                    source_layer: "L4".to_string(),
                });
            }
        }

        if query.include_l3 && let Some(es) = event_store {
            let results = hybrid
                .retrieve_l3(
                    &query.task_description,
                    &query.project_id,
                    query.session_id.as_deref(),
                    vi,
                    ec,
                    es,
                    l3_budget,
                )
                .await?;
            items_considered += results.len();
            if !results.is_empty() {
                let mut content = String::new();
                let mut tokens_used = 0;
                for r in &results {
                    let line = format!("- {}\n", r.content);
                    let line_tokens = estimate_tokens(&line);
                    content.push_str(&line);
                    tokens_used += line_tokens;
                }
                items_included += results.len();
                total_tokens += tokens_used;
                sections.push(ContextSection {
                    label: "Recent Episodes (L3, hybrid)".to_string(),
                    content,
                    token_count: tokens_used,
                    source_layer: "L3".to_string(),
                });
            }
        }

        if query.include_l1 && let Some(wm) = l1 {
            let (section, considered, included) = self.assemble_l1(wm, l1_budget);
            items_considered += considered;
            items_included += included;
            if let Some(s) = section {
                total_tokens += s.token_count;
                sections.push(s);
            }
        }

        Ok(AssembledContext {
            sections,
            total_tokens,
            budget: query.max_tokens,
            items_considered,
            items_included,
        })
    }

    pub fn assemble(
        &self,
        query: &ContextQuery,
        l1: Option<&WorkingMemory>,
        event_store: Option<&EventStore>,
        project_memory: Option<&ProjectMemory>,
    ) -> Result<AssembledContext, RetrievalError> {
        let mut sections = Vec::new();
        let mut total_tokens = 0;
        let mut items_considered = 0;
        let mut items_included = 0;

        let l4_budget = (query.max_tokens as f64 * self.l4_budget_ratio) as usize;
        let l3_budget = (query.max_tokens as f64 * self.l3_budget_ratio) as usize;
        let l1_budget = (query.max_tokens as f64 * self.l1_budget_ratio) as usize;

        if query.include_l4 && let Some(pm) = project_memory {
            let (section, considered, included) =
                self.assemble_l4(pm, &query.project_id, l4_budget)?;
            items_considered += considered;
            items_included += included;
            if let Some(s) = section {
                total_tokens += s.token_count;
                sections.push(s);
            }
        }

        if query.include_l3 && let Some(es) = event_store {
            let (section, considered, included) =
                self.assemble_l3(es, &query.project_id, query.session_id.as_deref(), l3_budget)?;
            items_considered += considered;
            items_included += included;
            if let Some(s) = section {
                total_tokens += s.token_count;
                sections.push(s);
            }
        }

        if query.include_l1 && let Some(wm) = l1 {
            let (section, considered, included) = self.assemble_l1(wm, l1_budget);
            items_considered += considered;
            items_included += included;
            if let Some(s) = section {
                total_tokens += s.token_count;
                sections.push(s);
            }
        }

        Ok(AssembledContext {
            sections,
            total_tokens,
            budget: query.max_tokens,
            items_considered,
            items_included,
        })
    }

    fn assemble_l4(
        &self,
        pm: &ProjectMemory,
        project_id: &str,
        budget: usize,
    ) -> Result<(Option<ContextSection>, usize, usize), RetrievalError> {
        let kinds = ["decision", "policy", "convention", "lesson", "constraint"];
        let mut all_facts: Vec<Fact> = Vec::new();

        for kind in &kinds {
            let facts = pm.query_by_kind(project_id, kind)?;
            all_facts.extend(facts);
        }

        let items_considered = all_facts.len();
        if all_facts.is_empty() {
            return Ok((None, 0, 0));
        }

        let mut scored: Vec<(Fact, f64)> = all_facts
            .into_iter()
            .map(|f| {
                let score = self.scorer.score_fact(&f);
                (f, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut content = String::new();
        let mut tokens_used = 0;
        let mut included = 0;

        for (fact, _score) in &scored {
            let line = format!("- **[{}]** {}: {}\n", fact.kind, fact.label, fact.description);
            let line_tokens = estimate_tokens(&line);
            if tokens_used + line_tokens > budget {
                break;
            }
            content.push_str(&line);
            tokens_used += line_tokens;
            included += 1;
        }

        if content.is_empty() {
            return Ok((None, items_considered, 0));
        }

        Ok((
            Some(ContextSection {
                label: "Project Knowledge (L4)".to_string(),
                content,
                token_count: tokens_used,
                source_layer: "L4".to_string(),
            }),
            items_considered,
            included,
        ))
    }

    fn assemble_l3(
        &self,
        es: &EventStore,
        project_id: &str,
        session_id: Option<&str>,
        budget: usize,
    ) -> Result<(Option<ContextSection>, usize, usize), RetrievalError> {
        let events = if let Some(sid) = session_id {
            es.query_by_session(project_id, sid)?
        } else {
            es.query_by_layer(project_id, Layer::L3)?
        };

        let items_considered = events.len();
        if events.is_empty() {
            return Ok((None, 0, 0));
        }

        let (newest_ts, oldest_ts) = timestamp_range(&events);

        let mut scored: Vec<(&MemoryEvent, f64)> = events
            .iter()
            .map(|e| {
                let score = self.scorer.score_event(e, newest_ts, oldest_ts);
                (e, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut content = String::new();
        let mut tokens_used = 0;
        let mut included = 0;

        for (event, _score) in &scored {
            let line = format_event(event);
            let line_tokens = estimate_tokens(&line);
            if tokens_used + line_tokens > budget {
                break;
            }
            content.push_str(&line);
            tokens_used += line_tokens;
            included += 1;
        }

        if content.is_empty() {
            return Ok((None, items_considered, 0));
        }

        Ok((
            Some(ContextSection {
                label: "Recent Episodes (L3)".to_string(),
                content,
                token_count: tokens_used,
                source_layer: "L3".to_string(),
            }),
            items_considered,
            included,
        ))
    }

    fn assemble_l1(&self, wm: &WorkingMemory, budget: usize) -> (Option<ContextSection>, usize, usize) {
        let items_considered = wm.slot_count();
        let content = wm.assemble_within_budget(budget);

        if content.is_empty() {
            return (None, items_considered, 0);
        }

        let tokens_used = crate::scoring::estimate_tokens(&content);

        (
            Some(ContextSection {
                label: "Working Memory (L1)".to_string(),
                content,
                token_count: tokens_used,
                source_layer: "L1".to_string(),
            }),
            items_considered,
            items_considered, // all slots that fit within budget are included
        )
    }
}

fn format_event(event: &MemoryEvent) -> String {
    let type_label = match event.event_type {
        EventType::Decision => "DECISION",
        EventType::Lesson => "LESSON",
        EventType::RejectedApproach => "REJECTED",
        EventType::Extraction => "EXTRACTION",
        EventType::TaskStart => "TASK_START",
        EventType::TaskEnd => "TASK_END",
        EventType::Turn => "TURN",
        EventType::ScratchFact => "FACT",
        EventType::Promotion => "PROMOTED",
    };

    let payload_str = if let Some(summary) = event.payload.get("summary").and_then(|v| v.as_str()) {
        summary.to_string()
    } else if let Some(content) = event.payload.get("content").and_then(|v| v.as_str()) {
        content.to_string()
    } else {
        event.payload.to_string()
    };

    format!("- [{}] {}: {}\n", type_label, event.timestamp, payload_str)
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
    use cmos_memory::l1::WorkingMemoryConfig;
    use cmos_memory::l4::FactSource;

    #[test]
    fn test_assemble_empty() {
        let assembler = ContextAssembler::default();
        let query = ContextQuery::new("test-project", "fix the login bug");

        let result = assembler.assemble(&query, None, None, None).unwrap();
        assert_eq!(result.total_tokens, 0);
        assert_eq!(result.sections.len(), 0);
    }

    #[test]
    fn test_assemble_l4_only() {
        let pm = ProjectMemory::open_in_memory().unwrap();
        pm.insert_fact(&Fact {
            id: None,
            project_id: "proj1".to_string(),
            kind: "decision".to_string(),
            label: "Use PostgreSQL".to_string(),
            description: "Chose PostgreSQL for ACID compliance".to_string(),
            source: FactSource::UserDeclared,
            confidence: 0.9,
            access_count: 5,
        }).unwrap();
        pm.insert_fact(&Fact {
            id: None,
            project_id: "proj1".to_string(),
            kind: "convention".to_string(),
            label: "snake_case".to_string(),
            description: "All Python code uses snake_case naming".to_string(),
            source: FactSource::Bootstrap,
            confidence: 0.8,
            access_count: 2,
        }).unwrap();

        let assembler = ContextAssembler::default();
        let query = ContextQuery::new("proj1", "add new endpoint");

        let result = assembler.assemble(&query, None, None, Some(&pm)).unwrap();
        assert!(result.total_tokens > 0);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].source_layer, "L4");
        assert_eq!(result.items_included, 2);
    }

    #[test]
    fn test_assemble_l1_only() {
        let wm = WorkingMemory::new(WorkingMemoryConfig::default());
        wm.insert("slot1", "System policy: always validate input", cmos_memory::l1::SlotPriority::Policy);
        wm.insert("slot2", "Current task context", cmos_memory::l1::SlotPriority::Context);

        let assembler = ContextAssembler::default();
        let mut query = ContextQuery::new("proj1", "test task");
        query.include_l3 = false;
        query.include_l4 = false;

        let result = assembler.assemble(&query, Some(&wm), None, None).unwrap();
        assert!(result.total_tokens > 0);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].source_layer, "L1");
    }

    #[test]
    fn test_budget_enforcement() {
        let pm = ProjectMemory::open_in_memory().unwrap();
        for i in 0..100 {
            pm.insert_fact(&Fact {
                id: None,
                project_id: "proj1".to_string(),
                kind: "decision".to_string(),
                label: format!("Decision {}", i),
                description: format!("A very important decision number {} that takes up space in the context window and should be truncated by budget", i),
                source: FactSource::Bootstrap,
                confidence: 0.7,
                access_count: 1,
            }).unwrap();
        }

        let assembler = ContextAssembler::default();
        let query = ContextQuery::new("proj1", "test").with_budget(500);

        let result = assembler.assemble(&query, None, None, Some(&pm)).unwrap();
        assert!(result.total_tokens <= 500);
        assert!(result.items_included < 100);
        assert_eq!(result.items_considered, 100);
    }

    #[test]
    fn test_render_with_header() {
        let ctx = AssembledContext {
            sections: vec![
                ContextSection {
                    label: "Test Section".to_string(),
                    content: "Some content here".to_string(),
                    token_count: 10,
                    source_layer: "L4".to_string(),
                },
            ],
            total_tokens: 10,
            budget: 32000,
            items_considered: 5,
            items_included: 1,
        };

        let rendered = ctx.render_with_header("fix the bug");
        assert!(rendered.contains("fix the bug"));
        assert!(rendered.contains("Test Section"));
        assert!(rendered.contains("Some content here"));
        assert!(rendered.contains("10 tokens used"));
    }
}
