//! `ecaz compare` — cross-engine comparison against external vector-search systems.
//!
//! `compare` is its own verb (not a `bench` subcommand) so future engines
//! (`compare faiss`, `compare weaviate`, `compare qdrant`) slot in cleanly.

use clap::Subcommand;
use color_eyre::eyre::Result;

use crate::psql::ConnectionOptions;

mod pgvector;
mod vchord;
mod vectorscale;

pub use pgvector::PgvectorArgs;
pub use vchord::VchordArgs;
pub use vectorscale::VectorscaleArgs;

#[derive(Subcommand, Debug)]
pub enum CompareCommand {
    /// Compare Ecaz (HNSW/DiskANN/...) against pgvector on the same corpus.
    Pgvector(PgvectorArgs),
    /// Compare Ecaz DiskANN against pgvectorscale StreamingDiskANN.
    Vectorscale(VectorscaleArgs),
    /// Compare Ecaz IVF against vchord RaBitQ-on-IVF.
    Vchord(VchordArgs),
}

impl CompareCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            CompareCommand::Pgvector(a) => pgvector::run(conn, a).await,
            CompareCommand::Vectorscale(a) => vectorscale::run(conn, a).await,
            CompareCommand::Vchord(a) => vchord::run(conn, a).await,
        }
    }
}
