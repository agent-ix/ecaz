//! `ecaz compare` — cross-engine comparison against external vector-search systems.
//!
//! `compare` is its own verb (not a `bench` subcommand) so future engines
//! (`compare faiss`, `compare weaviate`, `compare qdrant`) slot in cleanly.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod pgvector;

pub use pgvector::PgvectorArgs;

#[derive(Subcommand, Debug)]
pub enum CompareCommand {
    /// Compare Ecaz (HNSW/DiskANN/...) against pgvector on the same corpus.
    Pgvector(PgvectorArgs),
}

impl CompareCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            CompareCommand::Pgvector(a) => pgvector::run(database, a).await,
        }
    }
}
