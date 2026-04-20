//! `ecaz corpus inspect` — show row counts, dimension, and indexes for a corpus.

use clap::Args;
use color_eyre::eyre::Result;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
}

pub async fn run(_database: &str, _args: InspectArgs) -> Result<()> {
    // TODO(ecaz-cli v1): implement.
    // Print a two-part report:
    //   1. header: prefix, row count, dimension, embedding column type, first/last id
    //   2. table: indexes on `<prefix>_corpus`, each with access method, operator class,
    //      reloptions, and on-disk size from pg_relation_size.
    Err(color_eyre::eyre::eyre!(
        "ecaz corpus inspect: not yet implemented"
    ))
}
