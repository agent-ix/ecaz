use clap::{Args, Subcommand};
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{
    aws,
    datasets::{self, Dataset, Format},
    profiles::Profile,
    ssm,
    terraform::Terraform,
};

#[derive(Subcommand, Debug)]
pub enum CorpusCommand {
    /// List datasets known to the registry (FR-046).
    ListDatasets {
        #[arg(long)]
        json: bool,
    },
    /// Upload parquet (or converted BIGANN) shards for a dataset to the
    /// profile's S3 bucket. Defaults to running on the loader EC2 so the
    /// large download stays inside AWS.
    Stage(StageArgs),
    /// Fan out parquet → COPY workers on the loader EC2.
    Load(LoadArgs),
}

#[derive(Args, Debug)]
pub struct StageArgs {
    #[arg(long)]
    pub profile: Profile,
    #[arg(long)]
    pub dataset: String,
    /// Plan only — print what would be staged.
    #[arg(long)]
    pub dry_run: bool,
    /// Run the download + upload locally instead of on the loader EC2.
    /// Slower and more expensive (egress) but useful for offline-mirror
    /// debugging.
    #[arg(long)]
    pub local: bool,
    #[arg(long, default_value = "10800")]
    pub timeout: u64,
}

#[derive(Args, Debug)]
pub struct LoadArgs {
    #[arg(long)]
    pub profile: Profile,
    #[arg(long)]
    pub dataset: String,
    /// Skip shards whose receipts already exist in S3.
    #[arg(long)]
    pub resume: bool,
    /// Number of parallel workers on the loader EC2.
    #[arg(long, default_value = "8")]
    pub workers: usize,
    /// Target table name. Defaults to the dataset short name (with `-`
    /// rewritten to `_`).
    #[arg(long)]
    pub table: Option<String>,
    #[arg(long, default_value = "21600")]
    pub timeout: u64,
}

impl CorpusCommand {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        match self {
            CorpusCommand::ListDatasets { json } => list(json).await,
            CorpusCommand::Stage(args) => args.run(repo_root).await,
            CorpusCommand::Load(args) => args.run(repo_root).await,
        }
    }
}

async fn list(json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(datasets::REGISTRY)?);
    } else {
        println!(
            "{:<18} {:>13} {:>5} {:<6} {:<10} comparable",
            "name", "rows", "dim", "dist", "format"
        );
        for d in datasets::REGISTRY {
            println!(
                "{:<18} {:>13} {:>5} {:<6} {:<10} {}",
                d.name,
                d.rows,
                d.dim,
                format!("{:?}", d.distance),
                format!("{:?}", d.format),
                d.comparable_to.join(", ")
            );
        }
    }
    Ok(())
}

impl StageArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        let dataset = datasets::lookup(&self.dataset)
            .ok_or_else(|| eyre!("unknown dataset {:?}; try `ecaz cloud corpus list-datasets`", self.dataset))?;

        if self.dry_run {
            println!(
                "stage (plan): dataset={} rows={} dim={} format={:?} -> s3://<bucket>/parquet/{}/",
                dataset.name, dataset.rows, dataset.dim, dataset.format, dataset.name
            );
            return Ok(());
        }

        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack for profile {}", self.profile));
        }
        let out = tf.outputs().await?;

        if self.local {
            return Err(eyre!(
                "--local staging is not implemented yet; rerun without --local to stage via the loader EC2"
            ));
        }

        let script = stage_script(dataset, &out.s3_bucket);
        tracing::info!(dataset = dataset.name, "ssm: stage from loader to s3");
        ssm::run_shell(&out.region, &out.loader_instance_id, &script, self.timeout).await?;
        println!(
            "stage: dataset={} -> s3://{}/parquet/{}/",
            dataset.name, out.s3_bucket, dataset.name
        );
        Ok(())
    }
}

fn stage_script(d: &Dataset, bucket: &str) -> String {
    match d.format {
        Format::Parquet => parquet_stage_script(d, bucket),
        Format::BigAnnFbin => bigann_stage_script(d, bucket),
    }
}

fn parquet_stage_script(d: &Dataset, bucket: &str) -> String {
    // Hugging Face parquet repos resolve to
    // https://huggingface.co/datasets/<repo>/resolve/main/<path>. We rely
    // on `huggingface-hub` snapshot-download for glob support; fall back
    // to a `git lfs clone` if that pip is missing.
    let bucket = shell_escape(bucket);
    let name = shell_escape(d.name);
    let repo = shell_escape(d.source);
    let glob = shell_escape(d.source_path);
    format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
sudo -u loader bash -lc '
  set -eux
  pip install --user --quiet huggingface_hub
  python3 - <<PY
from huggingface_hub import snapshot_download
snapshot_download(
    repo_id={repo},
    repo_type="dataset",
    allow_patterns=[{glob}],
    local_dir="/home/loader/stage/{name}",
    local_dir_use_symlinks=False,
)
PY
  aws s3 sync /home/loader/stage/{name} s3://{bucket}/parquet/{name}/ \
    --exact-timestamps --only-show-errors
  aws s3 cp - s3://{bucket}/parquet/{name}/_manifest.json <<MANIFEST
{{"name": {name}, "format": "parquet", "rows": {rows}, "dim": {dim}}}
MANIFEST
  rm -rf /home/loader/stage/{name}
'
"#,
        rows = d.rows,
        dim = d.dim,
    )
}

fn bigann_stage_script(d: &Dataset, bucket: &str) -> String {
    // BIGANN is hosted as opaque .u8bin / .fbin blobs. We download the
    // base file, convert to parquet shards (10M rows each) using a small
    // python helper, and upload. This is the only non-parquet adapter.
    let bucket = shell_escape(bucket);
    let name = shell_escape(d.name);
    let url = shell_escape(&format!("{}/{}", d.source, d.source_path));
    format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
sudo -u loader bash -lc '
  set -eux
  pip install --user --quiet pyarrow numpy
  mkdir -p /home/loader/stage/{name}
  curl -L --fail -o /home/loader/stage/{name}/base.bin {url}
  python3 - <<PY
import os, struct, numpy as np, pyarrow as pa, pyarrow.parquet as pq
src = "/home/loader/stage/{name}/base.bin"
out = "/home/loader/stage/{name}"
with open(src, "rb") as f:
    n, d = struct.unpack("<II", f.read(8))
    SHARD = 10_000_000
    rest = n
    shard = 0
    while rest > 0:
        take = min(SHARD, rest)
        arr = np.fromfile(f, dtype=np.uint8, count=take * d).reshape(take, d)
        ids = np.arange(n - rest, n - rest + take, dtype=np.int64)
        tbl = pa.table({{
            "id": pa.array(ids),
            "vector": pa.array([row.tolist() for row in arr],
                                type=pa.list_(pa.uint8())),
        }})
        pq.write_table(tbl, f"{{out}}/shard-{{shard:05d}}.parquet")
        rest -= take
        shard += 1
PY
  rm -f /home/loader/stage/{name}/base.bin
  aws s3 sync /home/loader/stage/{name} s3://{bucket}/parquet/{name}/ \
    --exact-timestamps --only-show-errors
  aws s3 cp - s3://{bucket}/parquet/{name}/_manifest.json <<MANIFEST
{{"name": {name}, "format": "bigann-fbin", "rows": {rows}, "dim": {dim}}}
MANIFEST
  rm -rf /home/loader/stage/{name}
'
"#,
        rows = d.rows,
        dim = d.dim,
    )
}

impl LoadArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        let dataset = datasets::lookup(&self.dataset)
            .ok_or_else(|| eyre!("unknown dataset {:?}", self.dataset))?;
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack for profile {}", self.profile));
        }
        let out = tf.outputs().await?;
        let table = self
            .table
            .clone()
            .unwrap_or_else(|| dataset.name.replace('-', "_"));

        let script = load_script(LoadInputs {
            dataset,
            bucket: &out.s3_bucket,
            db_host: &out.db_private_ip,
            workers: self.workers,
            table: &table,
            resume: self.resume,
        });

        tracing::info!(
            dataset = dataset.name,
            workers = self.workers,
            "ssm: corpus load fan-out"
        );
        ssm::run_shell(&out.region, &out.loader_instance_id, &script, self.timeout).await?;
        println!(
            "load: dataset={} table={} workers={} ok",
            dataset.name, table, self.workers
        );
        Ok(())
    }
}

struct LoadInputs<'a> {
    dataset: &'a Dataset,
    bucket: &'a str,
    db_host: &'a str,
    workers: usize,
    table: &'a str,
    resume: bool,
}

fn load_script(i: LoadInputs<'_>) -> String {
    // The loader EC2 has the `ecaz` binary baked in by cloud-init.
    // Each worker runs `ecaz corpus prepare` + `ecaz corpus load` on
    // one parquet shard. Receipts are dropped under
    // s3://bucket/state/load/<dataset>/<shard>.done so `--resume`
    // re-runs skip already-done shards without re-counting rows.
    let bucket = shell_escape(i.bucket);
    let name = shell_escape(i.dataset.name);
    let host = shell_escape(i.db_host);
    let table = shell_escape(i.table);
    let workers = i.workers;
    let resume_guard = if i.resume {
        format!(
            r#"if aws s3api head-object --bucket {bucket_raw} --key state/load/{name_raw}/$(basename "$shard").done >/dev/null 2>&1; then
        echo "skip $shard"
        exit 0
    fi"#,
            bucket_raw = i.bucket,
            name_raw = i.dataset.name,
        )
    } else {
        String::new()
    };

    format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
sudo -u loader bash -lc '
  set -eux
  export PGHOST={host}
  export PGPORT=5432
  export PGUSER=postgres
  export PGDATABASE=postgres

  workdir=$(mktemp -d)
  trap "rm -rf $workdir" EXIT

  aws s3 ls s3://{bucket}/parquet/{name}/ \
    | awk "{{print \$NF}}" \
    | grep -E "\\.parquet$" > "$workdir/shards.txt"

  load_shard() {{
    shard="$1"
    local file="$workdir/$(basename "$shard")"
    {resume_guard}
    aws s3 cp s3://{bucket}/parquet/{name}/$shard "$file" --only-show-errors
    /usr/local/bin/ecaz corpus prepare \
      --input "$file" --output "$file.tsv" --table {table}
    /usr/local/bin/ecaz corpus load \
      --input "$file.tsv" --table {table}
    aws s3 cp - s3://{bucket}/state/load/{name}/$(basename "$shard").done <<<"$(date -Is)"
    rm -f "$file" "$file.tsv"
  }}
  export -f load_shard
  export workdir host=$PGHOST

  cat "$workdir/shards.txt" \
    | xargs -n1 -P{workers} -I{{}} bash -c "load_shard {{}}"
'
"#
    )
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
