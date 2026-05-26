use thiserror::Error;

#[derive(Debug, Error)]
pub enum RetrievalError {
    #[error("event store error: {0}")]
    EventStore(#[from] cmos_memory::l2l3::EventStoreError),
    #[error("project memory error: {0}")]
    ProjectMemory(#[from] cmos_memory::l4::ProjectMemoryError),
    #[error("budget exceeded: requested {requested} tokens, max {max}")]
    BudgetExceeded { requested: usize, max: usize },
    #[error("embedding error: {0}")]
    Embedding(String),
    #[error("vector index error: {0}")]
    VectorIndex(String),
}
