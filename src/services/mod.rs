//! Business logic services

pub mod chunking;
pub mod ingestion;

pub use chunking::chunk_text;
pub use ingestion::IngestionService;
