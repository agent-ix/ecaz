//! `ecaz stress ivf-vacuum-scale` — scale-oriented IVF VACUUM harness.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Args, Debug)]
pub struct IvfVacuumScaleArgs {
    /// Synthetic table prefix. One table/index is created per nlists value.
    #[arg(long, default_value = "ec_ivf_vacuum_scale")]
    pub table_prefix: String,
    /// Rows inserted before building each IVF index.
    #[arg(long, default_value_t = 1_000_000)]
    pub rows: i64,
    /// Comma-separated IVF centroid counts to measure.
    #[arg(long, value_delimiter = ',', default_value = "8,32,64")]
    pub nlists: Vec<i64>,
    /// Persisted IVF nprobe reloption.
    #[arg(long, default_value_t = 8)]
    pub nprobe: i64,
    /// Training sample rows reloption.
    #[arg(long, default_value_t = 10_000)]
    pub training_sample_rows: i64,
    /// Synthetic vector dimensions.
    #[arg(long, default_value_t = 4)]
    pub dimensions: i64,
    /// Quantizer profile reloption.
    #[arg(long, default_value = "turboquant")]
    pub quantizer: String,
    /// Number of delete/VACUUM cycles to run.
    #[arg(long, default_value_t = 1)]
    pub cycles: i64,
    /// Rows deleted per cycle. Defaults to half of --rows.
    #[arg(long)]
    pub churn_rows: Option<i64>,
    /// Refill the deleted range after each VACUUM to keep live rows steady.
    #[arg(long, default_value_t = false)]
    pub refill_after_vacuum: bool,
    /// Milliseconds between backend RSS samples during VACUUM.
    #[arg(long, default_value_t = 25)]
    pub sample_interval_ms: u64,
    /// Write a copy of the summary to this path.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
}

pub async fn run(database: &str, args: IvfVacuumScaleArgs) -> Result<()> {
    validate_args(&args)?;
    crate::profiles::validate_ident(&args.table_prefix)
        .wrap_err_with(|| format!("invalid --table-prefix {:?}", args.table_prefix))?;

    let client = crate::psql::connect(database).await?;
    let mut summaries = Vec::with_capacity(args.nlists.len());
    for nlists in &args.nlists {
        let surface = VacuumSurface::new(&args.table_prefix, *nlists);
        eprintln!(
            "[stress] preparing {} rows for {}",
            args.rows, surface.table_name
        );
        client
            .batch_execute(&build_setup_sql(&args, &surface))
            .await
            .wrap_err_with(|| format!("preparing {}", surface.table_name))?;

        for cycle in 1..=args.cycles {
            let cycle_summary = run_vacuum_cycle(&client, &args, &surface, cycle)
                .await
                .wrap_err_with(|| format!("running cycle {cycle} for {}", surface.table_name))?;
            summaries.push(cycle_summary);
        }
    }

    let summary = render_summary(&summaries);
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

fn validate_args(args: &IvfVacuumScaleArgs) -> Result<()> {
    if args.rows < 2 {
        return Err(eyre!("--rows must be >= 2"));
    }
    if args.nlists.is_empty() {
        return Err(eyre!("--nlists must include at least one value"));
    }
    if args.nlists.iter().any(|value| *value <= 0) {
        return Err(eyre!("--nlists values must be >= 1"));
    }
    if args.nprobe <= 0 {
        return Err(eyre!("--nprobe must be >= 1"));
    }
    if args.training_sample_rows < 0 {
        return Err(eyre!("--training-sample-rows must be >= 0"));
    }
    if args.dimensions <= 0 {
        return Err(eyre!("--dimensions must be >= 1"));
    }
    if args.sample_interval_ms == 0 {
        return Err(eyre!("--sample-interval-ms must be >= 1"));
    }
    if args.cycles <= 0 {
        return Err(eyre!("--cycles must be >= 1"));
    }
    let churn_rows = resolved_churn_rows(args)?;
    if churn_rows <= 0 {
        return Err(eyre!("--churn-rows must be >= 1"));
    }
    if churn_rows > args.rows {
        return Err(eyre!("--churn-rows must be <= --rows"));
    }
    if args.cycles > 1 && !args.refill_after_vacuum {
        return Err(eyre!(
            "--cycles greater than 1 requires --refill-after-vacuum"
        ));
    }
    crate::profiles::validate_ident(&args.quantizer)
        .wrap_err_with(|| format!("invalid --quantizer {:?}", args.quantizer))?;
    Ok(())
}

fn resolved_churn_rows(args: &IvfVacuumScaleArgs) -> Result<i64> {
    Ok(args.churn_rows.unwrap_or(args.rows / 2))
}

#[derive(Debug)]
struct VacuumSurface {
    nlists: i64,
    table_name: String,
    index_name: String,
}

impl VacuumSurface {
    fn new(prefix: &str, nlists: i64) -> Self {
        let table_name = format!("{prefix}_n{nlists}");
        Self {
            nlists,
            index_name: format!("{table_name}_idx"),
            table_name,
        }
    }
}

fn build_setup_sql(args: &IvfVacuumScaleArgs, surface: &VacuumSurface) -> String {
    let vector_sql = synthetic_vector_sql("gs::double precision", args.dimensions, 0.013, 0.021);
    format!(
        "CREATE EXTENSION IF NOT EXISTS ecaz;\n\
         DROP TABLE IF EXISTS {table} CASCADE;\n\
         CREATE TABLE {table} (\n    id bigint PRIMARY KEY,\n    embedding ecvector NOT NULL\n);\n\
         INSERT INTO {table} (id, embedding)\n\
         SELECT gs, {vector_sql}\n\
         FROM generate_series(1, {rows}) AS gs;\n\
         CREATE INDEX {index}\n    ON {table} USING ec_ivf (embedding ecvector_ip_ops)\n    WITH (\n        nlists = {nlists},\n        nprobe = {nprobe},\n        training_sample_rows = {training_sample_rows},\n        quantizer = '{quantizer}',\n        rerank = 'heap_f32'\n    );\n\
         ANALYZE {table};",
        table = surface.table_name,
        index = surface.index_name,
        rows = args.rows,
        nlists = surface.nlists,
        nprobe = args.nprobe,
        training_sample_rows = args.training_sample_rows,
        quantizer = args.quantizer,
        vector_sql = vector_sql,
    )
}

async fn run_vacuum_cycle(
    client: &tokio_postgres::Client,
    args: &IvfVacuumScaleArgs,
    surface: &VacuumSurface,
    cycle: i64,
) -> Result<VacuumScaleSummary> {
    let churn_rows = resolved_churn_rows(args)?;
    let delete_start_id = (cycle - 1) * churn_rows + 1;
    let delete_end_id = cycle * churn_rows;
    let rows_before_delete = relation_row_count(client, &surface.table_name).await?;
    let before_delete_bytes = relation_size(client, &surface.index_name).await?;

    let delete_start = Instant::now();
    client
        .execute(
            &format!(
                "DELETE FROM {} WHERE id BETWEEN $1 AND $2;",
                surface.table_name
            ),
            &[&delete_start_id, &delete_end_id],
        )
        .await
        .wrap_err_with(|| format!("deleting churn range from {}", surface.table_name))?;
    let delete_elapsed_ms = elapsed_ms(delete_start);
    let after_delete_bytes = relation_size(client, &surface.index_name).await?;

    let vacuum_pid: i32 = client
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .wrap_err("fetching vacuum backend pid")?
        .get(0);
    let stop = Arc::new(AtomicBool::new(false));
    let peak = Arc::new(Mutex::new(MemorySample::default()));
    let monitor = tokio::spawn(monitor_backend_memory(
        vacuum_pid,
        args.sample_interval_ms,
        Arc::clone(&stop),
        Arc::clone(&peak),
    ));

    let vacuum_start = Instant::now();
    client
        .batch_execute(&format!("VACUUM (ANALYZE) {};", surface.table_name))
        .await
        .wrap_err_with(|| format!("vacuuming {}", surface.table_name))?;
    let vacuum_elapsed_ms = elapsed_ms(vacuum_start);
    stop.store(true, Ordering::SeqCst);
    monitor
        .await
        .map_err(|e| eyre!("memory monitor task failed: {e}"))??;

    let after_vacuum_bytes = relation_size(client, &surface.index_name).await?;
    let rows_after_vacuum = relation_row_count(client, &surface.table_name).await?;

    let (insert_elapsed_ms, after_refill_bytes, rows_after_refill) = if args.refill_after_vacuum {
        let insert_start_id = args.rows + (cycle - 1) * churn_rows + 1;
        let insert_end_id = args.rows + cycle * churn_rows;
        let insert_start = Instant::now();
        client
            .batch_execute(&refill_sql(
                &surface.table_name,
                insert_start_id,
                insert_end_id,
                args.dimensions,
            ))
            .await
            .wrap_err_with(|| format!("refilling churn range in {}", surface.table_name))?;
        let insert_elapsed_ms = elapsed_ms(insert_start);
        let after_refill_bytes = relation_size(client, &surface.index_name).await?;
        let rows_after_refill = relation_row_count(client, &surface.table_name).await?;
        (insert_elapsed_ms, after_refill_bytes, rows_after_refill)
    } else {
        (0, after_vacuum_bytes, rows_after_vacuum)
    };

    let memory = *peak.lock().await;
    Ok(VacuumScaleSummary {
        nlists: surface.nlists,
        cycle,
        table_name: surface.table_name.clone(),
        index_name: surface.index_name.clone(),
        rows_before_delete,
        rows_after_vacuum,
        rows_after_refill,
        delete_elapsed_ms,
        vacuum_elapsed_ms,
        insert_elapsed_ms,
        index_bytes_before_delete: before_delete_bytes,
        index_bytes_after_delete: after_delete_bytes,
        index_bytes_after_vacuum: after_vacuum_bytes,
        index_bytes_after_refill: after_refill_bytes,
        vacuum_backend_pid: vacuum_pid,
        rss_peak_kb: memory.rss_peak_kb,
        hwm_peak_kb: memory.hwm_peak_kb,
        memory_samples: memory.samples,
    })
}

fn refill_sql(table_name: &str, start_id: i64, end_id: i64, dimensions: i64) -> String {
    let vector_sql = synthetic_vector_sql("gs::double precision", dimensions, 0.013, 0.021);
    format!(
        "INSERT INTO {table_name} (id, embedding)\n\
         SELECT gs, {vector_sql}\n\
         FROM generate_series({start_id}, {end_id}) AS gs;"
    )
}

fn synthetic_vector_sql(base: &str, dimensions: i64, first_rate: f64, second_rate: f64) -> String {
    if dimensions == 4 {
        return format!(
            "encode_to_ecvector(\n    ARRAY[\n        sin(({base} * {first_rate})::double precision)::real,\n        cos(({base} * {first_rate})::double precision)::real,\n        sin(({base} * {second_rate})::double precision)::real,\n        cos(({base} * {second_rate})::double precision)::real\n    ]::real[],\n    4,\n    42\n)"
        );
    }
    format!(
        "encode_to_ecvector(\n    ARRAY(\n        SELECT CASE\n            WHEN d % 2 = 0 THEN cos(({base} * {first_rate} + d * 0.001)::double precision)::real\n            ELSE sin(({base} * {second_rate} + d * 0.001)::double precision)::real\n        END\n        FROM generate_series(1, {dimensions}) AS d\n    )::real[],\n    4,\n    42\n)"
    )
}

async fn monitor_backend_memory(
    pid: i32,
    sample_interval_ms: u64,
    stop: Arc<AtomicBool>,
    peak: Arc<Mutex<MemorySample>>,
) -> Result<()> {
    while !stop.load(Ordering::Relaxed) {
        if let Some(sample) = read_proc_status_memory(pid).await? {
            let mut peak = peak.lock().await;
            peak.samples += 1;
            peak.rss_peak_kb = peak.rss_peak_kb.max(sample.rss_peak_kb);
            peak.hwm_peak_kb = peak.hwm_peak_kb.max(sample.hwm_peak_kb);
        }
        tokio::time::sleep(Duration::from_millis(sample_interval_ms)).await;
    }
    Ok(())
}

async fn read_proc_status_memory(pid: i32) -> Result<Option<MemorySample>> {
    let path = format!("/proc/{pid}/status");
    let Ok(contents) = tokio::fs::read_to_string(&path).await else {
        return Ok(None);
    };
    let mut sample = MemorySample::default();
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            sample.rss_peak_kb = parse_status_kb(value)?;
        } else if let Some(value) = line.strip_prefix("VmHWM:") {
            sample.hwm_peak_kb = parse_status_kb(value)?;
        }
    }
    Ok(Some(sample))
}

fn parse_status_kb(value: &str) -> Result<i64> {
    value
        .split_whitespace()
        .next()
        .ok_or_else(|| eyre!("missing /proc status memory value"))?
        .parse::<i64>()
        .wrap_err("parsing /proc status memory value")
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

async fn relation_row_count(client: &tokio_postgres::Client, table_name: &str) -> Result<i64> {
    let row = client
        .query_one(&format!("SELECT count(*) FROM {table_name}"), &[])
        .await
        .wrap_err_with(|| format!("counting rows in {table_name}"))?;
    Ok(row.get(0))
}

fn elapsed_ms(start: Instant) -> i64 {
    i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX)
}

#[derive(Debug, Default, Clone, Copy)]
struct MemorySample {
    rss_peak_kb: i64,
    hwm_peak_kb: i64,
    samples: i64,
}

#[derive(Debug)]
struct VacuumScaleSummary {
    nlists: i64,
    cycle: i64,
    table_name: String,
    index_name: String,
    rows_before_delete: i64,
    rows_after_vacuum: i64,
    rows_after_refill: i64,
    delete_elapsed_ms: i64,
    vacuum_elapsed_ms: i64,
    insert_elapsed_ms: i64,
    index_bytes_before_delete: i64,
    index_bytes_after_delete: i64,
    index_bytes_after_vacuum: i64,
    index_bytes_after_refill: i64,
    vacuum_backend_pid: i32,
    rss_peak_kb: i64,
    hwm_peak_kb: i64,
    memory_samples: i64,
}

fn render_summary(summaries: &[VacuumScaleSummary]) -> String {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec![
        "nlists",
        "cycle",
        "table",
        "index",
        "rows_before",
        "rows_after_vacuum",
        "rows_after_refill",
        "delete_ms",
        "vacuum_ms",
        "insert_ms",
        "idx_before",
        "idx_after_delete",
        "idx_after_vacuum",
        "idx_after_refill",
        "backend_pid",
        "rss_peak_kb",
        "hwm_peak_kb",
        "memory_samples",
    ]);
    for s in summaries {
        t.add_row(vec![
            Cell::new(s.nlists),
            Cell::new(s.cycle),
            Cell::new(&s.table_name),
            Cell::new(&s.index_name),
            Cell::new(s.rows_before_delete),
            Cell::new(s.rows_after_vacuum),
            Cell::new(s.rows_after_refill),
            Cell::new(s.delete_elapsed_ms),
            Cell::new(s.vacuum_elapsed_ms),
            Cell::new(s.insert_elapsed_ms),
            Cell::new(s.index_bytes_before_delete),
            Cell::new(s.index_bytes_after_delete),
            Cell::new(s.index_bytes_after_vacuum),
            Cell::new(s.index_bytes_after_refill),
            Cell::new(s.vacuum_backend_pid),
            Cell::new(s.rss_peak_kb),
            Cell::new(s.hwm_peak_kb),
            Cell::new(s.memory_samples),
        ]);
    }
    format!("{t}\nivf vacuum scale harness passed\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> IvfVacuumScaleArgs {
        IvfVacuumScaleArgs {
            table_prefix: "ivf_vacuum_test".to_owned(),
            rows: 100,
            nlists: vec![8, 32],
            nprobe: 8,
            training_sample_rows: 50,
            dimensions: 4,
            quantizer: "turboquant".to_owned(),
            cycles: 1,
            churn_rows: None,
            refill_after_vacuum: false,
            sample_interval_ms: 10,
            log_output: None,
        }
    }

    #[test]
    fn setup_sql_pins_ivf_vacuum_shape() {
        let surface = VacuumSurface::new("ivf_vacuum_test", 8);
        let sql = build_setup_sql(&args(), &surface);

        assert!(sql.contains("DROP TABLE IF EXISTS ivf_vacuum_test_n8 CASCADE"));
        assert!(sql.contains("USING ec_ivf (embedding ecvector_ip_ops)"));
        assert!(sql.contains("nlists = 8"));
        assert!(sql.contains("nprobe = 8"));
        assert!(sql.contains("training_sample_rows = 50"));
        assert!(sql.contains("quantizer = 'turboquant'"));
        assert!(sql.contains("generate_series(1, 100)"));
    }

    #[test]
    fn rejects_empty_nlists() {
        let mut args = args();
        args.nlists.clear();
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn rejects_multi_cycle_without_refill() {
        let mut args = args();
        args.cycles = 2;
        assert!(validate_args(&args).is_err());
    }

    #[test]
    fn refill_sql_uses_insert_range() {
        let sql = refill_sql("ivf_vacuum_test_n8", 101, 125, 4);
        assert!(sql.contains("INSERT INTO ivf_vacuum_test_n8"));
        assert!(sql.contains("generate_series(101, 125)"));
    }

    #[test]
    fn parses_proc_status_kb() {
        assert_eq!(parse_status_kb("   12345 kB").unwrap(), 12345);
    }
}
