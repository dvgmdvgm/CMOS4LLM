pub mod assembly;
pub mod embedding;
pub mod error;
pub mod hybrid;
pub mod scoring;
pub mod vector;

pub use assembly::{AssembledContext, ContextAssembler, ContextQuery};
pub use embedding::{EmbeddingClient, EmbeddingConfig};
pub use error::RetrievalError;
pub use hybrid::{HybridConfig, HybridResult, HybridRetriever};
pub use scoring::RelevanceScorer;
pub use vector::{VectorIndex, VectorRecord, VectorSearchResult};
