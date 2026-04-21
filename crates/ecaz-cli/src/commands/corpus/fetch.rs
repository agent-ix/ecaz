//! `ecaz corpus fetch` — first-class remote parquet fetch for canonical corpora.
//!
//! The current operator need is narrow: fetch the canonical DBpedia/OpenAI
//! parquet release used by the real-corpus recall contract into a local
//! `output_dir/data/` tree that `ecaz corpus prepare --parquet ...` can
//! consume directly.
//!
//! The implementation keeps the moving parts explicit and testable:
//! dataset resolution, shard naming, and resolve-URL construction are pure
//! helpers; network I/O is a thin async shell.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use futures::StreamExt;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::psql::ConnectionOptions;

pub const DEFAULT_DATASET: &str = "qdrant-dbpedia-openai3-large-1536-1m";
pub const FETCH_MANIFEST_FILE: &str = "ecaz_fetch_manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteDataset {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub repo: &'static str,
    pub revision: &'static str,
    pub remote_data_dir: &'static str,
    pub shard_prefix: &'static str,
    pub shard_count: usize,
    pub source_dataset: &'static str,
}

pub const QDRANT_DBPEDIA_OPENAI3_1M: RemoteDataset = RemoteDataset {
    name: DEFAULT_DATASET,
    aliases: &[
        "dbpedia-openai3-large-1536-1m",
        "dbpedia-entities-openai3-text-embedding-3-large-1536-1m",
        "Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M",
    ],
    repo: "Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M",
    revision: "main",
    remote_data_dir: "data",
    shard_prefix: "train",
    shard_count: 26,
    source_dataset: super::prepare::DEFAULT_SOURCE_DATASET,
};

pub const DATASETS: &[RemoteDataset] = &[QDRANT_DBPEDIA_OPENAI3_1M];

#[derive(Args, Debug)]
pub struct FetchArgs {
    /// Named remote dataset to fetch.
    #[arg(long, default_value = DEFAULT_DATASET)]
    pub dataset: String,

    /// Directory to populate. Parquet shards land under `<output-dir>/data/`.
    #[arg(long)]
    pub output_dir: PathBuf,

    /// Override the remote revision/tag/commit. Defaults to the dataset registry revision.
    #[arg(long)]
    pub revision: Option<String>,

    /// Re-download shards even if the destination file already exists.
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

pub async fn run(_conn: &ConnectionOptions, args: FetchArgs) -> Result<()> {
    let dataset = resolve_dataset(&args.dataset).ok_or_else(|| {
        let known: Vec<&str> = DATASETS.iter().map(|dataset| dataset.name).collect();
        eyre!(
            "unknown dataset {:?}; try {}",
            args.dataset,
            known.join(", ")
        )
    })?;
    let revision = args
        .revision
        .clone()
        .unwrap_or_else(|| dataset.revision.to_owned());
    let data_dir = args.output_dir.join(dataset.remote_data_dir);
    tokio::fs::create_dir_all(&data_dir)
        .await
        .wrap_err_with(|| format!("creating {}", data_dir.display()))?;

    let client = reqwest::Client::builder()
        .user_agent(format!("ecaz-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .wrap_err("building HTTP client")?;
    let shards = shard_filenames(dataset);
    let mut downloaded = 0usize;
    let mut skipped = 0usize;

    for (idx, shard) in shards.iter().enumerate() {
        let destination = data_dir.join(shard);
        if destination.exists() && !args.force {
            skipped += 1;
            eprintln!(
                "[fetch] [{}/{}] skipping existing {}",
                idx + 1,
                shards.len(),
                destination.display()
            );
            continue;
        }

        let url = build_download_url(dataset, &revision, shard);
        eprintln!(
            "[fetch] [{}/{}] downloading {}",
            idx + 1,
            shards.len(),
            shard
        );
        download_file(&client, &url, &destination).await?;
        downloaded += 1;
    }

    let manifest_path = args.output_dir.join(FETCH_MANIFEST_FILE);
    let manifest = build_fetch_manifest(
        dataset,
        &revision,
        dataset.remote_data_dir,
        &shards,
        &chrono::Utc::now().to_rfc3339(),
    );
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    tokio::fs::write(&manifest_path, bytes)
        .await
        .wrap_err_with(|| format!("writing {}", manifest_path.display()))?;

    eprintln!(
        "[fetch] done. downloaded={} skipped={} parquet_dir={}",
        downloaded,
        skipped,
        data_dir.display()
    );
    eprintln!(
        "[fetch] next: ecaz corpus prepare --profile ec_hnsw_real_10k --parquet {} --output-dir /path/to/staged",
        data_dir.display()
    );
    Ok(())
}

pub fn resolve_dataset(name: &str) -> Option<&'static RemoteDataset> {
    DATASETS.iter().find(|dataset| {
        dataset.name == name
            || dataset
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(name))
    })
}

pub fn shard_filename(prefix: &str, index: usize, total: usize) -> String {
    format!("{prefix}-{index:05}-of-{total:05}.parquet")
}

pub fn shard_filenames(dataset: &RemoteDataset) -> Vec<String> {
    (0..dataset.shard_count)
        .map(|idx| shard_filename(dataset.shard_prefix, idx, dataset.shard_count))
        .collect()
}

pub fn build_download_url(dataset: &RemoteDataset, revision: &str, shard: &str) -> String {
    format!(
        "https://huggingface.co/datasets/{repo}/resolve/{revision}/{remote_data_dir}/{shard}",
        repo = dataset.repo,
        remote_data_dir = dataset.remote_data_dir,
    )
}

pub fn build_fetch_manifest(
    dataset: &RemoteDataset,
    revision: &str,
    parquet_dir: &str,
    shards: &[String],
    fetched_at: &str,
) -> Value {
    json!({
        "manifest_version": 1,
        "dataset": dataset.name,
        "source_dataset": dataset.source_dataset,
        "repo": dataset.repo,
        "revision": revision,
        "parquet_dir": parquet_dir,
        "shards": shards,
        "fetched_at": fetched_at,
    })
}

async fn download_file(client: &reqwest::Client, url: &str, destination: &Path) -> Result<()> {
    let temp_path = temp_download_path(destination)?;
    if temp_path.exists() {
        tokio::fs::remove_file(&temp_path)
            .await
            .wrap_err_with(|| format!("removing stale {}", temp_path.display()))?;
    }
    let result: Result<()> = async {
        let response = client
            .get(url)
            .send()
            .await
            .wrap_err_with(|| format!("requesting {url}"))?
            .error_for_status()
            .wrap_err_with(|| format!("downloading {url}"))?;
        let mut stream = response.bytes_stream();
        let mut handle = tokio::fs::File::create(&temp_path)
            .await
            .wrap_err_with(|| format!("creating {}", temp_path.display()))?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.wrap_err_with(|| format!("reading response stream from {url}"))?;
            handle
                .write_all(&chunk)
                .await
                .wrap_err_with(|| format!("writing {}", temp_path.display()))?;
        }
        handle
            .flush()
            .await
            .wrap_err_with(|| format!("flushing {}", temp_path.display()))?;
        drop(handle);
        tokio::fs::rename(&temp_path, destination)
            .await
            .wrap_err_with(|| {
                format!(
                    "moving completed download {} -> {}",
                    temp_path.display(),
                    destination.display()
                )
            })?;
        Ok(())
    }
    .await;
    if result.is_err() && temp_path.exists() {
        let _ = tokio::fs::remove_file(&temp_path).await;
    }
    result
}

fn temp_download_path(destination: &Path) -> Result<PathBuf> {
    let file_name = destination.file_name().ok_or_else(|| {
        eyre!(
            "download destination {} has no terminal file name",
            destination.display()
        )
    })?;
    Ok(destination.with_file_name(format!("{}.part", file_name.to_string_lossy())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_dataset_accepts_canonical_name_and_aliases() {
        assert_eq!(
            resolve_dataset(DEFAULT_DATASET).map(|d| d.repo),
            Some(QDRANT_DBPEDIA_OPENAI3_1M.repo)
        );
        assert_eq!(
            resolve_dataset("dbpedia-openai3-large-1536-1m").map(|d| d.repo),
            Some(QDRANT_DBPEDIA_OPENAI3_1M.repo)
        );
        assert_eq!(
            resolve_dataset("Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M")
                .map(|d| d.repo),
            Some(QDRANT_DBPEDIA_OPENAI3_1M.repo)
        );
    }

    #[test]
    fn shard_filename_matches_hugging_face_layout() {
        assert_eq!(
            shard_filename("train", 0, 26),
            "train-00000-of-00026.parquet"
        );
        assert_eq!(
            shard_filename("train", 25, 26),
            "train-00025-of-00026.parquet"
        );
    }

    #[test]
    fn shard_filenames_cover_all_shards() {
        let shards = shard_filenames(&QDRANT_DBPEDIA_OPENAI3_1M);
        assert_eq!(shards.len(), 26);
        assert_eq!(shards.first().unwrap(), "train-00000-of-00026.parquet");
        assert_eq!(shards.last().unwrap(), "train-00025-of-00026.parquet");
    }

    #[test]
    fn build_download_url_points_at_dataset_resolve_path() {
        let url = build_download_url(
            &QDRANT_DBPEDIA_OPENAI3_1M,
            "main",
            "train-00000-of-00026.parquet",
        );
        assert_eq!(
            url,
            "https://huggingface.co/datasets/Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M/resolve/main/data/train-00000-of-00026.parquet"
        );
    }

    #[test]
    fn build_fetch_manifest_records_repo_revision_and_shards() {
        let manifest = build_fetch_manifest(
            &QDRANT_DBPEDIA_OPENAI3_1M,
            "main",
            "data",
            &["train-00000-of-00026.parquet".to_owned()],
            "2026-04-21T00:00:00+00:00",
        );
        assert_eq!(manifest["manifest_version"], 1);
        assert_eq!(manifest["dataset"], DEFAULT_DATASET);
        assert_eq!(manifest["repo"], QDRANT_DBPEDIA_OPENAI3_1M.repo);
        assert_eq!(manifest["revision"], "main");
        assert_eq!(manifest["parquet_dir"], "data");
        assert_eq!(manifest["shards"][0], "train-00000-of-00026.parquet");
    }

    #[test]
    fn temp_download_path_appends_part_suffix() {
        let path = temp_download_path(Path::new("/tmp/data/train-00000-of-00026.parquet"))
            .expect("temp path");
        assert_eq!(
            path,
            PathBuf::from("/tmp/data/train-00000-of-00026.parquet.part")
        );
    }
}
