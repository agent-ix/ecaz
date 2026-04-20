//! `ecaz compare` — cross-engine comparison against external vector-search systems.
//!
//! `compare` is its own verb (not a `bench` subcommand) so future engines
//! (`compare faiss`, `compare weaviate`, `compare qdrant`) slot in cleanly.
//!
//! v1 status: command tree declared; implementations land in v2.

use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};

#[derive(Subcommand, Debug)]
pub enum CompareCommand {
    /// Compare Ecaz (HNSW/DiskANN/...) against pgvector on the same corpus.
    Pgvector(PgvectorArgs),
}

#[derive(Args, Debug)]
pub struct PgvectorArgs {
    /// Loaded Ecaz corpus prefix (data is shared between engines via a
    /// pgvector-format sidecar column or separate pgvector table).
    #[arg(long)]
    pub prefix: String,

    /// Ecaz profile to compare against pgvector.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,

    /// pgvector index type to build (ivfflat, hnsw, ...).
    #[arg(long, default_value = "hnsw")]
    pub pgvector_index: String,

    /// k for recall@k comparison.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
}

impl CompareCommand {
    pub async fn run(self, _database: &str) -> Result<()> {
        Err(eyre!(
            "ecaz compare pgvector: not yet implemented (ported in a v2 PR)"
        ))
    }
}
