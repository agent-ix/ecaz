//! `ecaz stress ivf-insert` — live insert throughput harness for ec_ivf.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[derive(Args, Debug)]
pub struct IvfInsertArgs {
    /// Wall-clock duration for the concurrent insert phase (seconds).
    #[arg(long, default_value_t = 15)]
    pub duration_seconds: u64,
    /// Override the synthetic table name.
    #[arg(long, default_value = "ec_ivf_insert_stress")]
    pub table: String,
    /// Seed corpus size inserted before the IVF index is built.
    #[arg(long, default_value_t = 1000)]
    pub seed_rows: i64,
    /// Concurrent insert worker connections.
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,
    /// Rows inserted by each worker per INSERT statement.
    #[arg(long, default_value_t = 1)]
    pub batch_rows: i64,
    /// IVF centroid count for the measured index.
    #[arg(long, default_value_t = 16)]
    pub nlists: i64,
    /// Persisted IVF nprobe reloption.
    #[arg(long, default_value_t = 16)]
    pub nprobe: i64,
    /// Training sample rows reloption.
    #[arg(long, default_value_t = 1000)]
    pub training_sample_rows: i64,
    /// Write a copy of the summary to this path.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
    /// Fail instead of falling back to relation stats when IVF admin snapshot is unavailable.
    #[arg(long)]
    pub require_admin_snapshot: bool,
}

pub async fn run(database: &str, args: IvfInsertArgs) -> Result<()> {
    validate_args(&args)?;
    crate::profiles::validate_ident(&args.table)
        .wrap_err_with(|| format!("invalid --table {:?}", args.table))?;
    let index_name = format!("{}_idx", args.table);

    let client = crate::psql::connect(database).await?;
    eprintln!(
        "[stress] seeding {} rows into {} and building {}",
        args.seed_rows, args.table, index_name
    );
    client
        .batch_execute(&build_seed_ddl(&args, &index_name))
        .await
        .wrap_err("seeding IVF insert stress table")?;

    let stop = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_secs(args.duration_seconds);
    let mut workers = Vec::with_capacity(args.concurrency);
    for worker_id in 0..args.concurrency {
        workers.push(tokio::spawn(insert_worker(
            database.to_owned(),
            args.table.clone(),
            args.batch_rows,
            worker_id,
            Arc::clone(&stop),
            deadline,
        )));
    }

    let deadline_watcher = tokio::spawn({
        let stop = Arc::clone(&stop);
        async move {
            tokio::time::sleep_until(deadline.into()).await;
            stop.store(true, Ordering::SeqCst);
        }
    });

    let mut total_rows = 0_i64;
    let mut total_iterations = 0_u64;
    for worker in workers {
        let result = worker.await.map_err(|e| eyre!("insert worker: {e}"))??;
        total_rows = total_rows
            .checked_add(result.rows)
            .ok_or_else(|| eyre!("inserted row count overflow"))?;
        total_iterations = total_iterations
            .checked_add(result.iterations)
            .ok_or_else(|| eyre!("insert iteration count overflow"))?;
    }
    let _ = deadline_watcher.await;

    let snapshot = fetch_ivf_snapshot(
        &client,
        &args.table,
        &index_name,
        args.require_admin_snapshot,
    )
    .await?;
    let summary = render_summary(&InsertSummary {
        duration_seconds: args.duration_seconds,
        concurrency: args.concurrency,
        batch_rows: args.batch_rows,
        seed_rows: args.seed_rows,
        total_inserted_rows: total_rows,
        total_insert_iterations: total_iterations,
        inserted_rows_per_second: total_rows as f64 / args.duration_seconds as f64,
        index_name,
        total_live_tuples: snapshot.total_live_tuples,
        inserted_since_build: snapshot.inserted_since_build,
        changed_row_fraction: snapshot.changed_row_fraction,
        average_list_live_count: snapshot.average_list_live_count,
        max_list_live_count: snapshot.max_list_live_count,
        list_imbalance_ratio: snapshot.list_imbalance_ratio,
        reindex_recommended: snapshot.reindex_recommended,
        reindex_reason: snapshot.reindex_reason,
        snapshot_source: snapshot.snapshot_source,
        index_bytes: snapshot.index_bytes,
    });
    println!("{summary}");
    if let Some(path) = args.log_output {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .wrap_err_with(|| format!("creating {}", parent.display()))?;
        }
        tokio::fs::write(&path, &summary)
            .await
            .wrap_err_with(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn validate_args(args: &IvfInsertArgs) -> Result<()> {
    if args.duration_seconds == 0 {
        return Err(eyre!("--duration-seconds must be >= 1"));
    }
    if args.seed_rows <= 0 {
        return Err(eyre!("--seed-rows must be >= 1"));
    }
    if args.concurrency == 0 {
        return Err(eyre!("--concurrency must be >= 1"));
    }
    if args.batch_rows <= 0 {
        return Err(eyre!("--batch-rows must be >= 1"));
    }
    if args.nlists <= 0 {
        return Err(eyre!("--nlists must be >= 1"));
    }
    if args.nprobe <= 0 {
        return Err(eyre!("--nprobe must be >= 1"));
    }
    if args.training_sample_rows < 0 {
        return Err(eyre!("--training-sample-rows must be >= 0"));
    }
    Ok(())
}

pub fn build_seed_ddl(args: &IvfInsertArgs, index_name: &str) -> String {
    format!(
        "DROP TABLE IF EXISTS {table} CASCADE;\n\
         CREATE TABLE {table} (\n    id bigserial PRIMARY KEY,\n    embedding ecvector NOT NULL\n);\n\
         INSERT INTO {table} (embedding)\n\
         SELECT encode_to_ecvector(\n    ARRAY[\n        sin((gs * 0.013)::double precision)::real,\n        cos((gs * 0.013)::double precision)::real,\n        sin((gs * 0.021)::double precision)::real,\n        cos((gs * 0.021)::double precision)::real\n    ]::real[],\n    4,\n    42\n)\n\
         FROM generate_series(1, {seed_rows}) AS gs;\n\
         CREATE INDEX {index_name}\n    ON {table} USING ec_ivf (embedding ecvector_ip_ops)\n    WITH (\n        nlists = {nlists},\n        nprobe = {nprobe},\n        training_sample_rows = {training_sample_rows},\n        storage_format = 'turboquant',\n        rerank = 'heap_f32',\n        rerank_width = 10\n    );",
        table = args.table,
        seed_rows = args.seed_rows,
        index_name = index_name,
        nlists = args.nlists,
        nprobe = args.nprobe,
        training_sample_rows = args.training_sample_rows,
    )
}

fn build_insert_sql(table: &str) -> String {
    format!(
        "INSERT INTO {table} (embedding)\n\
         SELECT encode_to_ecvector(\n    ARRAY[\n        sin((((extract(epoch FROM clock_timestamp()) * 1000000)::bigint + $1::bigint + gs)::double precision) * 0.017)::real,\n        cos((((extract(epoch FROM clock_timestamp()) * 1000000)::bigint + $1::bigint + gs)::double precision) * 0.017)::real,\n        sin((((extract(epoch FROM clock_timestamp()) * 1000000)::bigint + $1::bigint + gs)::double precision) * 0.029)::real,\n        cos((((extract(epoch FROM clock_timestamp()) * 1000000)::bigint + $1::bigint + gs)::double precision) * 0.029)::real\n    ]::real[],\n    4,\n    42\n)\n\
         FROM generate_series(1, $2::bigint) AS gs"
    )
}

async fn insert_worker(
    database: String,
    table: String,
    batch_rows: i64,
    worker_id: usize,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<WorkerResult> {
    let client = crate::psql::connect(&database).await?;
    let sql = build_insert_sql(&table);
    let stmt = client.prepare(&sql).await?;
    let mut rows = 0_i64;
    let mut iterations = 0_u64;
    let worker_offset = i64::try_from(worker_id)
        .map_err(|_| eyre!("worker id exceeds i64"))?
        .checked_mul(1_000_000_000)
        .ok_or_else(|| eyre!("worker offset overflow"))?;
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        let inserted = client
            .execute(&stmt, &[&worker_offset, &batch_rows])
            .await
            .wrap_err("insert iteration")?;
        rows = rows
            .checked_add(i64::try_from(inserted).map_err(|_| eyre!("insert count exceeds i64"))?)
            .ok_or_else(|| eyre!("inserted row count overflow"))?;
        iterations = iterations
            .checked_add(1)
            .ok_or_else(|| eyre!("insert iteration count overflow"))?;
    }
    Ok(WorkerResult { rows, iterations })
}

async fn fetch_ivf_snapshot(
    client: &tokio_postgres::Client,
    table_name: &str,
    index_name: &str,
    require_admin_snapshot: bool,
) -> Result<IvfSnapshot> {
    let has_admin_snapshot: bool = client
        .query_one(
            "SELECT to_regprocedure('ec_ivf_index_admin_snapshot(oid)') IS NOT NULL",
            &[],
        )
        .await
        .wrap_err("checking ec_ivf_index_admin_snapshot availability")?
        .get(0);
    let index_bytes = relation_size(client, index_name).await?;
    if !has_admin_snapshot {
        if require_admin_snapshot {
            return Err(eyre!(
                "ec_ivf_index_admin_snapshot(oid) is required but is not installed; use a fresh PG18 database with the current ecaz extension SQL"
            ));
        }
        let table_rows = relation_row_count(client, table_name).await?;
        return Ok(IvfSnapshot {
            total_live_tuples: table_rows.to_string(),
            inserted_since_build: "unavailable".to_owned(),
            changed_row_fraction: "unavailable".to_owned(),
            average_list_live_count: "unavailable".to_owned(),
            max_list_live_count: "unavailable".to_owned(),
            list_imbalance_ratio: "unavailable".to_owned(),
            reindex_recommended: "unavailable".to_owned(),
            reindex_reason: "ec_ivf_index_admin_snapshot(oid) not installed".to_owned(),
            snapshot_source: "fallback_relation_stats".to_owned(),
            index_bytes,
        });
    }

    let row = client
        .query_one(
            "SELECT total_live_tuples,
                    inserted_since_build,
                    changed_row_fraction,
                    average_list_live_count,
                    max_list_live_count,
                    list_imbalance_ratio,
                    reindex_recommended,
                    reindex_reason
             FROM ec_ivf_index_admin_snapshot($1::text::regclass::oid)",
            &[&index_name],
        )
        .await
        .wrap_err("fetching ec_ivf_index_admin_snapshot")?;
    Ok(IvfSnapshot {
        total_live_tuples: row.get::<_, i64>(0).to_string(),
        inserted_since_build: row.get::<_, i64>(1).to_string(),
        changed_row_fraction: format!("{:.6}", row.get::<_, f64>(2)),
        average_list_live_count: format!("{:.2}", row.get::<_, f64>(3)),
        max_list_live_count: row.get::<_, i64>(4).to_string(),
        list_imbalance_ratio: format!("{:.6}", row.get::<_, f64>(5)),
        reindex_recommended: row.get::<_, bool>(6).to_string(),
        reindex_reason: row.get(7),
        snapshot_source: "ec_ivf_index_admin_snapshot".to_owned(),
        index_bytes,
    })
}

async fn relation_row_count(client: &tokio_postgres::Client, table_name: &str) -> Result<i64> {
    let sql = format!("SELECT count(*) FROM {table_name}");
    let row = client
        .query_one(&sql, &[])
        .await
        .wrap_err_with(|| format!("counting rows in {table_name}"))?;
    Ok(row.get(0))
}

async fn relation_size(client: &tokio_postgres::Client, relation_name: &str) -> Result<i64> {
    let row = client
        .query_one(
            "SELECT pg_relation_size($1::text::regclass)",
            &[&relation_name],
        )
        .await
        .wrap_err_with(|| format!("measuring size for {relation_name}"))?;
    Ok(row.get(0))
}

#[derive(Debug, Clone, Copy)]
struct WorkerResult {
    rows: i64,
    iterations: u64,
}

#[derive(Debug)]
struct IvfSnapshot {
    total_live_tuples: String,
    inserted_since_build: String,
    changed_row_fraction: String,
    average_list_live_count: String,
    max_list_live_count: String,
    list_imbalance_ratio: String,
    reindex_recommended: String,
    reindex_reason: String,
    snapshot_source: String,
    index_bytes: i64,
}

#[derive(Debug)]
struct InsertSummary {
    duration_seconds: u64,
    concurrency: usize,
    batch_rows: i64,
    seed_rows: i64,
    total_inserted_rows: i64,
    total_insert_iterations: u64,
    inserted_rows_per_second: f64,
    index_name: String,
    total_live_tuples: String,
    inserted_since_build: String,
    changed_row_fraction: String,
    average_list_live_count: String,
    max_list_live_count: String,
    list_imbalance_ratio: String,
    reindex_recommended: String,
    reindex_reason: String,
    snapshot_source: String,
    index_bytes: i64,
}

fn render_summary(s: &InsertSummary) -> String {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["metric", "value"]);
    for (k, v) in [
        ("duration_seconds", s.duration_seconds.to_string()),
        ("concurrency", s.concurrency.to_string()),
        ("batch_rows", s.batch_rows.to_string()),
        ("seed_rows", s.seed_rows.to_string()),
        ("total_inserted_rows", s.total_inserted_rows.to_string()),
        (
            "total_insert_iterations",
            s.total_insert_iterations.to_string(),
        ),
        (
            "inserted_rows_per_second",
            format!("{:.2}", s.inserted_rows_per_second),
        ),
        ("index_name", s.index_name.clone()),
        ("snapshot_source", s.snapshot_source.clone()),
        ("index_bytes", s.index_bytes.to_string()),
        ("total_live_tuples", s.total_live_tuples.clone()),
        ("inserted_since_build", s.inserted_since_build.clone()),
        ("changed_row_fraction", s.changed_row_fraction.clone()),
        ("average_list_live_count", s.average_list_live_count.clone()),
        ("max_list_live_count", s.max_list_live_count.clone()),
        ("list_imbalance_ratio", s.list_imbalance_ratio.clone()),
        ("reindex_recommended", s.reindex_recommended.clone()),
        ("reindex_reason", s.reindex_reason.clone()),
    ] {
        t.add_row(vec![Cell::new(k), Cell::new(v)]);
    }
    format!("{t}\nivf insert stress harness passed\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> IvfInsertArgs {
        IvfInsertArgs {
            duration_seconds: 1,
            table: "ivf_insert_test".to_owned(),
            seed_rows: 64,
            concurrency: 2,
            batch_rows: 3,
            nlists: 8,
            nprobe: 4,
            training_sample_rows: 32,
            log_output: None,
            require_admin_snapshot: false,
        }
    }

    #[test]
    fn seed_ddl_pins_ivf_shape() {
        let sql = build_seed_ddl(&args(), "ivf_insert_test_idx");
        assert!(sql.contains("DROP TABLE IF EXISTS ivf_insert_test CASCADE"));
        assert!(sql.contains("USING ec_ivf (embedding ecvector_ip_ops)"));
        assert!(sql.contains("nlists = 8"));
        assert!(sql.contains("nprobe = 4"));
        assert!(sql.contains("training_sample_rows = 32"));
        assert!(sql.contains("storage_format = 'turboquant'"));
        assert!(sql.contains("rerank = 'heap_f32'"));
    }

    #[test]
    fn insert_sql_uses_parameterized_batch_size() {
        let sql = build_insert_sql("ivf_insert_test");
        assert!(sql.contains("INSERT INTO ivf_insert_test"));
        assert!(sql.contains("$1::bigint"));
        assert!(sql.contains("$2::bigint"));
    }

    #[test]
    fn rejects_zero_duration() {
        let mut args = args();
        args.duration_seconds = 0;
        assert!(validate_args(&args).is_err());
    }
}
