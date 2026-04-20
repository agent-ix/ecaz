//! `ecaz corpus list` — enumerate loaded corpora.

use color_eyre::eyre::Result;

pub async fn run(_database: &str) -> Result<()> {
    // TODO(ecaz-cli v1): implement.
    // Query for pairs of tables matching `(<prefix>_corpus, <prefix>_queries)`
    // and print a comfy-table of (prefix, corpus rows, queries rows, embedding type,
    // indexes built). Uses `psql::client` once wired.
    Err(color_eyre::eyre::eyre!(
        "ecaz corpus list: not yet implemented"
    ))
}
