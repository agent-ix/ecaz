//! `ecaz corpus load` — port of the legacy `scripts/load_real_corpus.py`.
//!
//! See the module-level doc in `super` for the corpus model. This command
//! is the only way new data enters Postgres; everything downstream assumes
//! the `<prefix>_corpus` / `<prefix>_queries` contract it establishes.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use std::path::PathBuf;

use crate::profiles;
use crate::psql;

#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Fixture prefix used for table and index names. Must match
    /// [a-zA-Z_][a-zA-Z0-9_]*.
    #[arg(long)]
    pub prefix: String,

    /// Path to <basename>_corpus.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub corpus_file: PathBuf,

    /// Path to <basename>_queries.tsv (one `id\t<json_array>` per line).
    #[arg(long)]
    pub queries_file: PathBuf,

    /// Vector dimensionality.
    #[arg(long, default_value_t = 1536)]
    pub dim: usize,

    /// Access-method profile (drives embedding type, encoder, opclass).
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,

    /// Quantization bits passed to the profile's encoder.
    #[arg(long, default_value_t = 4)]
    pub bits: i32,

    /// Quantizer seed passed to the profile's encoder.
    #[arg(long, default_value_t = 42)]
    pub seed: i64,

    /// HNSW-only: m values to sweep. Accepts `--m 8,16` or repeated `--m 8 --m 16`.
    #[arg(long, value_delimiter = ',')]
    pub m: Vec<i32>,

    /// HNSW-only: ef_construction passed to CREATE INDEX.
    #[arg(long, default_value_t = 128)]
    pub ef_construction: i32,

    /// Optional storage format (turboquant / pq_fastscan).
    #[arg(long)]
    pub storage_format: Option<String>,

    /// AM-specific reloption passthrough. Repeatable.
    /// Example: `--reloption graph_degree=48 --reloption alpha=1.2`.
    #[arg(long = "reloption", value_parser = crate::reloptions::parse_cli)]
    pub reloptions: Vec<(String, String)>,

    /// Optional manifest file path (auto-discovered when corpus/queries files
    /// follow the `<basename>_{corpus,queries}.tsv` convention).
    #[arg(long)]
    pub manifest_file: Option<PathBuf>,

    /// Continue past manifest verification failures with a warning.
    #[arg(long)]
    pub allow_manifest_mismatch: bool,
}

pub async fn run(database: &str, args: LoadArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    let profile = profiles::resolve(&args.profile)
        .ok_or_else(|| eyre!("unknown profile {:?}; try {}", args.profile, profiles::names().join(", ")))?;

    if !profile.sweep_axis_is_m() && !args.m.is_empty() {
        return Err(eyre!(
            "--m is not supported by profile {:?}; use --reloption for AM-specific tunables",
            profile.name
        ));
    }

    let _client = psql::connect(database).await.wrap_err("connecting to postgres")?;

    // TODO(ecaz-cli v1): port the full loader flow:
    //   1. optional manifest verification (`<basename>_manifest.json`)
    //   2. CREATE TABLE <prefix>_corpus   (id, source real[], embedding <profile.embedding_type>)
    //   3. COPY corpus rows, then UPDATE embedding = <profile.encoder_function>(source, bits, seed)
    //   4. CREATE TABLE <prefix>_queries  (id, source real[])
    //   5. COPY queries rows
    //   6. CREATE INDEX per sweep value, using profile.access_method + profile.operator_class
    //      + reloptions assembled from profile-specific knobs plus --reloption passthrough
    //   7. print a Rich-style summary table (profile, rows, indexes, sha256)
    //
    // For now this stub validates inputs and confirms Postgres is reachable so
    // the command tree compiles end-to-end.

    tracing::info!(
        profile = profile.name,
        prefix = %args.prefix,
        corpus = %args.corpus_file.display(),
        queries = %args.queries_file.display(),
        dim = args.dim,
        "corpus load: inputs validated; full flow implementation pending"
    );
    Err(eyre!(
        "ecaz corpus load: not yet implemented (v1 stub; full port in progress)"
    ))
}
