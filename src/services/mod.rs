//! Business logic services

pub mod chunking;
pub mod ingestion;
pub mod search;

pub use chunking::chunk_text;
pub use ingestion::IngestionService;
pub use search::SearchService;
