//! `ecaz corpus subset` — prepared TSV -> smaller canonical TSV fixture.

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::prepare::{resolve_profile, FileManifest, DEFAULT_DIM};

#[derive(Args, Debug)]
pub struct SubsetArgs {
    /// Canonical subset profile to emit, for example `ec_real_50k`.
    #[arg(long)]
    pub profile: String,
    /// Larger prepared corpus TSV whose row ids follow global sorted-row order.
    #[arg(long)]
    pub source_corpus_file: PathBuf,
    /// Optional larger prepared manifest to cite as provenance.
    #[arg(long)]
    pub source_manifest_file: Option<PathBuf>,
    /// Directory to write the subset TSVs and manifest into.
    #[arg(long)]
    pub output_dir: PathBuf,
    /// Expected embedding dimensionality recorded in the manifest.
    #[arg(long, default_value_t = DEFAULT_DIM)]
    pub dim: usize,
}

pub async fn run(args: SubsetArgs) -> Result<()> {
    let profile = resolve_profile(&args.profile).ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "unknown profile {:?}; use a profile known to `ecaz corpus prepare`",
            args.profile
        )
    })?;
    tokio::fs::create_dir_all(&args.output_dir)
        .await
        .wrap_err_with(|| format!("creating {}", args.output_dir.display()))?;

    let corpus_path = args
        .output_dir
        .join(format!("{}_corpus.tsv", profile.prefix));
    let queries_path = args
        .output_dir
        .join(format!("{}_queries.tsv", profile.prefix));
    let (corpus, queries) = subset_tsv(
        &args.source_corpus_file,
        &corpus_path,
        &queries_path,
        profile.corpus_rows,
        profile.query_rows,
    )?;

    let source_manifest = match &args.source_manifest_file {
        Some(path) => Some(read_manifest(path).await?),
        None => None,
    };
    let manifest = build_subset_manifest(
        profile.prefix,
        args.dim,
        &args.source_corpus_file,
        args.source_manifest_file.as_deref(),
        source_manifest.as_ref(),
        &corpus,
        &queries,
    );
    let manifest_path = args
        .output_dir
        .join(format!("{}_manifest.json", profile.prefix));
    tokio::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .await
        .wrap_err_with(|| format!("writing {}", manifest_path.display()))?;

    println!(
        "[subset] profile={} corpus_rows={} query_rows={} source={}",
        profile.prefix,
        corpus.rows,
        queries.rows,
        args.source_corpus_file.display()
    );
    println!("[subset] wrote {}", corpus_path.display());
    println!("[subset] wrote {}", queries_path.display());
    println!("[subset] wrote {}", manifest_path.display());
    Ok(())
}

fn subset_tsv(
    source_corpus_file: &Path,
    corpus_path: &Path,
    queries_path: &Path,
    corpus_rows: usize,
    query_rows: usize,
) -> Result<(FileManifest, FileManifest)> {
    let needed_rows = corpus_rows + query_rows;
    let source = File::open(source_corpus_file)
        .wrap_err_with(|| format!("opening {}", source_corpus_file.display()))?;
    let mut corpus_writer = manifesting_writer(corpus_path)?;
    let mut queries_writer = manifesting_writer(queries_path)?;

    for (ordinal, line) in BufReader::new(source).lines().enumerate() {
        if ordinal >= needed_rows {
            break;
        }
        let mut line = line.wrap_err_with(|| {
            format!(
                "reading row {} from {}",
                ordinal,
                source_corpus_file.display()
            )
        })?;
        line.push('\n');
        if ordinal < corpus_rows {
            corpus_writer.write_line(&line)?;
        } else {
            queries_writer.write_line(&line)?;
        }
    }

    let corpus = corpus_writer.finish()?;
    let queries = queries_writer.finish()?;
    if corpus.rows != corpus_rows {
        bail!(
            "source corpus had {} rows before query split; profile requires {}",
            corpus.rows,
            corpus_rows
        );
    }
    if queries.rows != query_rows {
        bail!(
            "source corpus had {} query rows after corpus split; profile requires {}",
            queries.rows,
            query_rows
        );
    }
    Ok((corpus, queries))
}

struct ManifestingWriter {
    file_name: String,
    writer: BufWriter<File>,
    hasher: Sha256,
    rows: usize,
    first_id: Option<i64>,
    last_id: Option<i64>,
}

fn manifesting_writer(path: &Path) -> Result<ManifestingWriter> {
    let file = File::create(path).wrap_err_with(|| format!("creating {}", path.display()))?;
    Ok(ManifestingWriter {
        file_name: path
            .file_name()
            .expect("output path should have file name")
            .to_string_lossy()
            .into_owned(),
        writer: BufWriter::new(file),
        hasher: Sha256::new(),
        rows: 0,
        first_id: None,
        last_id: None,
    })
}

impl ManifestingWriter {
    fn write_line(&mut self, line: &str) -> Result<()> {
        let row_id = parse_row_id(line)?;
        if self.first_id.is_none() {
            self.first_id = Some(row_id);
        }
        self.last_id = Some(row_id);
        self.rows += 1;
        self.hasher.update(line.as_bytes());
        self.writer.write_all(line.as_bytes())?;
        Ok(())
    }

    fn finish(mut self) -> Result<FileManifest> {
        self.writer.flush()?;
        Ok(FileManifest {
            file: self.file_name,
            rows: self.rows,
            sha256: hex::encode(self.hasher.finalize()),
            first_id: self.first_id,
            last_id: self.last_id,
            first_source_id: None,
            last_source_id: None,
        })
    }
}

fn parse_row_id(line: &str) -> Result<i64> {
    let Some((id, _)) = line.split_once('\t') else {
        bail!("TSV row is missing tab separator")
    };
    id.parse::<i64>()
        .wrap_err_with(|| format!("parsing row id {id:?}"))
}

async fn read_manifest(path: &Path) -> Result<Value> {
    let raw = tokio::fs::read_to_string(path)
        .await
        .wrap_err_with(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).wrap_err_with(|| format!("parsing {}", path.display()))
}

fn build_subset_manifest(
    prefix: &str,
    dim: usize,
    source_corpus_file: &Path,
    source_manifest_file: Option<&Path>,
    source_manifest: Option<&Value>,
    corpus: &FileManifest,
    queries: &FileManifest,
) -> Value {
    json!({
        "manifest_version": 1,
        "artifact_layout": "single_tsv",
        "prefix": prefix,
        "source_dataset": source_manifest
            .and_then(|m| m.get("source_dataset"))
            .cloned()
            .unwrap_or(Value::Null),
        "source_corpus_file": source_corpus_file.display().to_string(),
        "source_manifest_file": source_manifest_file.map(|p| p.display().to_string()),
        "dimension": dim,
        "selection_rule": {
            "sort_key": "global_sorted_row_index ascending",
            "corpus_start": 0,
            "corpus_rows": corpus.rows,
            "query_start": corpus.rows,
            "query_rows": queries.rows,
            "output_id_mode": "preserve_global_sorted_row_index",
        },
        "corpus": file_manifest_json(corpus),
        "queries": file_manifest_json(queries),
        "generated_at_utc": Utc::now().to_rfc3339(),
        "generated_by": "ecaz corpus subset",
    })
}

fn file_manifest_json(m: &FileManifest) -> Value {
    json!({
        "file": m.file,
        "rows": m.rows,
        "sha256": m.sha256,
        "first_id": m.first_id,
        "last_id": m.last_id,
        "first_source_id": m.first_source_id,
        "last_source_id": m.last_source_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn subset_tsv_splits_source_corpus_by_profile_rows() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.tsv");
        std::fs::write(
            &source,
            "0\t[1.0]\n1\t[2.0]\n2\t[3.0]\n3\t[4.0]\n4\t[5.0]\n",
        )
        .unwrap();

        let corpus_path = tmp.path().join("subset_corpus.tsv");
        let queries_path = tmp.path().join("subset_queries.tsv");
        let (corpus, queries) = subset_tsv(&source, &corpus_path, &queries_path, 3, 2).unwrap();

        assert_eq!(std::fs::read_to_string(corpus_path).unwrap(), "0\t[1.0]\n1\t[2.0]\n2\t[3.0]\n");
        assert_eq!(std::fs::read_to_string(queries_path).unwrap(), "3\t[4.0]\n4\t[5.0]\n");
        assert_eq!(corpus.rows, 3);
        assert_eq!(corpus.first_id, Some(0));
        assert_eq!(corpus.last_id, Some(2));
        assert_eq!(queries.rows, 2);
        assert_eq!(queries.first_id, Some(3));
        assert_eq!(queries.last_id, Some(4));
    }
}
