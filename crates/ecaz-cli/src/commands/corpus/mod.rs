//! `ecaz corpus` — load / inspect / list / generate corpora.
//!
//! A "corpus" is a named fixture identified by `--prefix`. Each corpus
//! occupies two tables (`<prefix>_corpus`, `<prefix>_queries`) and any
//! number of indexes built on `<prefix>_corpus.embedding`. Multiple
//! corpora coexist freely; many profiles can share a single corpus table
//! when their `embedding_type` matches.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod generate;
mod inspect;
mod list;
mod load;
mod prepare;

pub use generate::GenerateArgs;
pub use inspect::InspectArgs;
pub use load::LoadArgs;
pub use prepare::PrepareArgs;

#[derive(Subcommand, Debug)]
pub enum CorpusCommand {
    /// Load a local-file fixture (`<basename>_corpus.tsv` + `<basename>_queries.tsv`)
    /// into Postgres under the given prefix and build an index.
    Load(LoadArgs),
    /// Print row counts, dimension, and indexes for a loaded corpus.
    Inspect(InspectArgs),
    /// Enumerate all loaded corpora in the database.
    List,
    /// Generate a synthetic unit-sphere TSV dataset (no DB access) suitable
    /// for feeding into `ecaz corpus load`.
    Generate(GenerateArgs),
    /// Convert a Qdrant-DBpedia-style parquet release into canonical
    /// `<prefix>_corpus.tsv` + `<prefix>_queries.tsv` + manifest.
    Prepare(PrepareArgs),
}

impl CorpusCommand {
    pub async fn run(self, database: &str) -> Result<()> {
        match self {
            CorpusCommand::Load(args) => load::run(database, args).await,
            CorpusCommand::Inspect(args) => inspect::run(database, args).await,
            CorpusCommand::List => list::run(database).await,
            CorpusCommand::Generate(args) => generate::run(database, args).await,
            CorpusCommand::Prepare(args) => prepare::run(database, args).await,
        }
    }
}
