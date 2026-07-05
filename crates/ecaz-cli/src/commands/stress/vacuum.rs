//! `ecaz stress vacuum` — concurrent insert/delete/scan/VACUUM harness.
//!
//! Drives four concurrent
//! workers against a synthetic `ec_hnsw`-indexed table for a fixed wall
//! duration, then asserts that VACUUM left the index structurally sound.
//!
//! # Prerequisites
//!
//! A `pg_test` build of the extension so the following debug functions
//! are available in the target database:
//!
//! - `tests.ec_hnsw_debug_scan_result_count(oid, real[]) -> bigint`
//! - `tests.ec_hnsw_debug_reachable_live_element_count(oid) -> bigint`
//! - `ec_hnsw_index_admin_snapshot(regclass) -> record` (non-test build
//!   also exposes this).
//!
//! # Invariants
//!
//! After the run winds down we rebuild a fresh reference index on the
//! same surviving rows and verify:
//!
//! 1. `final_live_elements == reference_live_elements` (no ghost nodes).
//! 2. `final_scan_result_count > 0` (index is still queryable).
//! 3. `final_reachable_live_elements >= 90% * reference` (connectivity
//!    did not collapse).
//!
//! # Purity boundary
//!
//! SQL builders and `check_reachability_floor` are pure functions with
//! unit tests. The orchestration is a thin tokio::spawn shell.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use crate::psql::{self, ConnectionOptions};

#[derive(Args, Debug)]
pub struct VacuumArgs {
    /// Wall-clock duration for the concurrent phase (seconds).
    #[arg(long, default_value_t = 60)]
    pub duration_seconds: u64,
    /// Override the default synthetic table name.
    #[arg(long, default_value = "ec_hnsw_vacuum_concurrency")]
    pub table: String,
    /// Seed corpus size (rows inserted before workers start).
    #[arg(long, default_value_t = 2000)]
    pub seed_rows: i64,
    /// Reachability floor: final reachable live elements must be at least
    /// `reachability_floor_pct` percent of the rebuilt reference.
    #[arg(long, default_value_t = 90)]
    pub reachability_floor_pct: u32,
}

pub async fn run(conn: &ConnectionOptions, args: VacuumArgs) -> Result<()> {
    if args.duration_seconds == 0 {
        return Err(eyre!("--duration-seconds must be >= 1"));
    }
    if args.reachability_floor_pct == 0 || args.reachability_floor_pct > 100 {
        return Err(eyre!(
            "--reachability-floor-pct must be in 1..=100 (got {})",
            args.reachability_floor_pct
        ));
    }
    crate::profiles::validate_ident(&args.table)
        .wrap_err_with(|| format!("invalid --table {:?}", args.table))?;
    let table = args.table.clone();
    let index_name = format!("{table}_idx");
    let ref_index_name = format!("{table}_ref_idx");

    let client = psql::connect(conn).await?;
    ensure_debug_functions(&client).await?;

    crate::ecaz_eprintln!("[stress] seeding {} rows into {table}", args.seed_rows);
    client
        .batch_execute(&build_seed_ddl(&table, &index_name, args.seed_rows))
        .await
        .wrap_err("seeding vacuum harness table")?;

    let stop = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_secs(args.duration_seconds);

    let insert = tokio::spawn(insert_worker(
        conn.clone(),
        table.clone(),
        Arc::clone(&stop),
        deadline,
    ));
    let vacuum = tokio::spawn(vacuum_worker(
        conn.clone(),
        table.clone(),
        Arc::clone(&stop),
        deadline,
    ));
    let scan_a = tokio::spawn(scan_worker(
        conn.clone(),
        index_name.clone(),
        vec![1.0, 0.0, 0.5, -1.0],
        Arc::clone(&stop),
        deadline,
    ));
    let scan_b = tokio::spawn(scan_worker(
        conn.clone(),
        index_name.clone(),
        vec![0.0, 1.0, -0.5, 0.25],
        Arc::clone(&stop),
        deadline,
    ));

    let deadline_watcher = tokio::spawn({
        let stop = Arc::clone(&stop);
        async move {
            tokio::time::sleep_until(deadline.into()).await;
            stop.store(true, Ordering::SeqCst);
        }
    });

    let ins = insert.await.map_err(|e| eyre!("insert worker: {e}"))??;
    let vac = vacuum.await.map_err(|e| eyre!("vacuum worker: {e}"))??;
    let sa = scan_a.await.map_err(|e| eyre!("scan_a worker: {e}"))??;
    let sb = scan_b.await.map_err(|e| eyre!("scan_b worker: {e}"))??;
    let _ = deadline_watcher.await;

    crate::ecaz_eprintln!("[stress] running final VACUUM (ANALYZE) and measuring invariants");
    client
        .batch_execute(&format!("VACUUM (ANALYZE) {table}"))
        .await
        .wrap_err("final VACUUM")?;

    let final_live_rows: i64 = client
        .query_one(&format!("SELECT count(*) FROM {table}"), &[])
        .await?
        .get(0);
    let final_live_elements: i64 = client
        .query_one(
            "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot($1::regclass)",
            &[&index_name],
        )
        .await?
        .get(0);
    let final_reachable: i64 = client
        .query_one(
            "SELECT tests.ec_hnsw_debug_reachable_live_element_count($1::regclass::oid)",
            &[&index_name],
        )
        .await?
        .get(0);
    let final_scan_count: i64 = client
        .query_one(
            "SELECT tests.ec_hnsw_debug_scan_result_count(\
                $1::regclass::oid, ARRAY[1.0, 0.0, 0.5, -1.0]::real[])",
            &[&index_name],
        )
        .await?
        .get(0);

    // Rebuild a reference index on the surviving rows and compare.
    client
        .batch_execute(&format!("DROP INDEX IF EXISTS {ref_index_name}"))
        .await?;
    client
        .batch_execute(&build_reference_index_sql(&table, &ref_index_name))
        .await
        .wrap_err("building reference index")?;
    let reference_elements: i64 = client
        .query_one(
            "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot($1::regclass)",
            &[&ref_index_name],
        )
        .await?
        .get(0);
    let reference_reachable: i64 = client
        .query_one(
            "SELECT tests.ec_hnsw_debug_reachable_live_element_count($1::regclass::oid)",
            &[&ref_index_name],
        )
        .await?
        .get(0);

    if final_live_elements != reference_elements {
        return Err(eyre!(
            "final_live_elements ({final_live_elements}) != reference ({reference_elements}) — VACUUM left ghost nodes"
        ));
    }
    if final_scan_count <= 0 {
        return Err(eyre!(
            "final scan returned {final_scan_count} rows — index is unqueryable"
        ));
    }
    if reference_reachable <= 0 {
        return Err(eyre!(
            "reference reachable count is {reference_reachable}; harness is broken"
        ));
    }
    check_reachability_floor(
        final_reachable,
        reference_reachable,
        args.reachability_floor_pct,
    )?;

    render_summary(VacuumSummary {
        duration_seconds: args.duration_seconds,
        insert_iterations: ins,
        vacuum_iterations: vac,
        scan_a_iterations: sa,
        scan_b_iterations: sb,
        final_live_rows,
        final_live_elements,
        final_reachable,
        reference_reachable,
        final_scan_count,
    });
    Ok(())
}

async fn ensure_debug_functions(client: &tokio_postgres::Client) -> Result<()> {
    let has_scan: bool = client
        .query_one(
            "SELECT to_regprocedure('tests.ec_hnsw_debug_scan_result_count(oid,real[])') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !has_scan {
        return Err(eyre!(
            "missing tests.ec_hnsw_debug_scan_result_count(oid, real[]); \
             install a pg_test build for the target PostgreSQL major version"
        ));
    }
    Ok(())
}

/// DDL that drops (if any), recreates, seeds `seed_rows` synthetic vectors,
/// and builds the measured ec_hnsw index.
pub fn build_seed_ddl(table: &str, index_name: &str, seed_rows: i64) -> String {
    format!(
        "DROP TABLE IF EXISTS {table} CASCADE;\n\
         CREATE TABLE {table} (\n    id bigserial PRIMARY KEY,\n    embedding ecvector NOT NULL\n);\n\
         INSERT INTO {table} (embedding)\n\
         SELECT encode_to_ecvector(\n    ARRAY[\n        sin((gs * 0.013)::double precision)::real,\n        cos((gs * 0.013)::double precision)::real,\n        sin((gs * 0.021)::double precision)::real,\n        cos((gs * 0.021)::double precision)::real\n    ]::real[],\n    4,\n    42\n)\nFROM generate_series(1, {seed_rows}) AS gs;\n\
         CREATE INDEX {index_name}\n    ON {table} USING ec_hnsw (embedding ecvector_ip_ops)\n    WITH (m = 8, ef_construction = 64);"
    )
}

/// Reference-index rebuild SQL used for the post-run comparison.
pub fn build_reference_index_sql(table: &str, ref_index: &str) -> String {
    format!(
        "CREATE INDEX {ref_index}\n    ON {table} USING ec_hnsw (embedding ecvector_ip_ops)\n    WITH (m = 8, ef_construction = 64)"
    )
}

/// INSERT iteration executed repeatedly by the insert worker.
pub fn build_insert_iteration_sql(table: &str) -> String {
    format!(
        "INSERT INTO {table} (embedding)\n\
         SELECT encode_to_ecvector(\n    ARRAY[\n        (random() * 2.0 - 1.0)::real,\n        (random() * 2.0 - 1.0)::real,\n        (random() * 2.0 - 1.0)::real,\n        (random() * 2.0 - 1.0)::real\n    ]::real[],\n    4,\n    42\n)\nFROM generate_series(1, 4)"
    )
}

/// DELETE + VACUUM pair executed repeatedly by the vacuum worker.
pub fn build_vacuum_iteration_sql(table: &str) -> String {
    format!(
        "DELETE FROM {table}\n    WHERE id IN (SELECT id FROM {table} ORDER BY id LIMIT 2);\n\
         VACUUM {table};"
    )
}

async fn connect_worker(conn: &ConnectionOptions) -> Result<tokio_postgres::Client> {
    psql::connect(conn).await
}

async fn insert_worker(
    conn: ConnectionOptions,
    table: String,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<u64> {
    let client = connect_worker(&conn).await?;
    let sql = build_insert_iteration_sql(&table);
    let mut iterations = 0_u64;
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        client
            .batch_execute(&sql)
            .await
            .wrap_err("insert iteration")?;
        iterations += 1;
    }
    Ok(iterations)
}

async fn vacuum_worker(
    conn: ConnectionOptions,
    table: String,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<u64> {
    let client = connect_worker(&conn).await?;
    let sql = build_vacuum_iteration_sql(&table);
    let count_sql = format!("SELECT count(*) FROM {table}");
    let mut iterations = 0_u64;
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        client.batch_execute(&sql).await.wrap_err("vacuum iter")?;
        let live: i64 = client.query_one(count_sql.as_str(), &[]).await?.get(0);
        if live <= 0 {
            return Err(eyre!("vacuum worker: live row count dropped to {live}"));
        }
        iterations += 1;
    }
    Ok(iterations)
}

async fn scan_worker(
    conn: ConnectionOptions,
    index_name: String,
    query: Vec<f32>,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<u64> {
    let client = connect_worker(&conn).await?;
    let stmt = client
        .prepare("SELECT tests.ec_hnsw_debug_scan_result_count($1::regclass::oid, $2::real[])")
        .await?;
    let mut iterations = 0_u64;
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        let count: i64 = client
            .query_one(&stmt, &[&index_name, &query])
            .await?
            .get(0);
        if count <= 0 {
            return Err(eyre!(
                "scan worker saw ec_hnsw scan return {count} rows (expected > 0)"
            ));
        }
        iterations += 1;
    }
    Ok(iterations)
}

/// Enforce the reachability floor: `final * 100 >= reference * floor_pct`.
/// Returns `Ok(())` when the floor holds, otherwise a descriptive error.
///
/// Multiplication-based comparison avoids a floating-point threshold so the
/// result is byte-identical to the legacy bash check.
pub fn check_reachability_floor(
    final_reachable: i64,
    reference_reachable: i64,
    floor_pct: u32,
) -> Result<()> {
    if reference_reachable <= 0 {
        return Err(eyre!(
            "reference reachable count must be > 0 (got {reference_reachable})"
        ));
    }
    let lhs = (final_reachable as i128) * 100;
    let rhs = (reference_reachable as i128) * floor_pct as i128;
    if lhs < rhs {
        return Err(eyre!(
            "reachable live elements ({final_reachable}) fell below \
             {floor_pct}% of rebuilt reference ({reference_reachable})"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct VacuumSummary {
    duration_seconds: u64,
    insert_iterations: u64,
    vacuum_iterations: u64,
    scan_a_iterations: u64,
    scan_b_iterations: u64,
    final_live_rows: i64,
    final_live_elements: i64,
    final_reachable: i64,
    reference_reachable: i64,
    final_scan_count: i64,
}

fn render_summary(s: VacuumSummary) {
    let pct = if s.reference_reachable > 0 {
        s.final_reachable * 100 / s.reference_reachable
    } else {
        0
    };
    let mut t = Table::new();
    t.load_preset(UTF8_FULL);
    t.set_header(vec!["metric", "value"]);
    for (k, v) in [
        ("duration_seconds", s.duration_seconds as i64),
        ("insert_worker_iterations", s.insert_iterations as i64),
        ("vacuum_worker_iterations", s.vacuum_iterations as i64),
        ("scan_a_worker_iterations", s.scan_a_iterations as i64),
        ("scan_b_worker_iterations", s.scan_b_iterations as i64),
        ("final_live_rows", s.final_live_rows),
        ("final_live_elements", s.final_live_elements),
        ("final_reachable_live_elements", s.final_reachable),
        ("reference_reachable_live_elements", s.reference_reachable),
        ("reachable_vs_reference_percent", pct),
        ("final_scan_result_count", s.final_scan_count),
    ] {
        t.add_row(vec![Cell::new(k), Cell::new(v)]);
    }
    crate::ecaz_println!("{t}");
    crate::ecaz_println!("vacuum concurrency harness passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SQL builders ---

    #[test]
    fn seed_ddl_pins_ec_hnsw_opclass_and_encoder() {
        let sql = build_seed_ddl("t", "t_idx", 2000);
        assert!(sql.contains("DROP TABLE IF EXISTS t CASCADE"));
        assert!(sql.contains("embedding ecvector NOT NULL"));
        assert!(sql.contains("encode_to_ecvector"));
        assert!(sql.contains("generate_series(1, 2000)"));
        assert!(sql.contains("USING ec_hnsw (embedding ecvector_ip_ops)"));
        assert!(sql.contains("m = 8"));
        assert!(sql.contains("ef_construction = 64"));
    }

    #[test]
    fn reference_index_sql_matches_measured_index_shape() {
        let sql = build_reference_index_sql("t", "t_ref_idx");
        assert!(sql.contains("CREATE INDEX t_ref_idx"));
        assert!(sql.contains("USING ec_hnsw (embedding ecvector_ip_ops)"));
        assert!(sql.contains("m = 8"));
        assert!(sql.contains("ef_construction = 64"));
    }

    #[test]
    fn insert_iteration_adds_four_rows_per_call() {
        let sql = build_insert_iteration_sql("t");
        assert!(sql.contains("INSERT INTO t"));
        assert!(sql.contains("generate_series(1, 4)"));
        assert!(sql.contains("encode_to_ecvector"));
    }

    #[test]
    fn vacuum_iteration_deletes_oldest_two_then_vacuums() {
        let sql = build_vacuum_iteration_sql("t");
        assert!(sql.contains("DELETE FROM t"));
        assert!(sql.contains("ORDER BY id LIMIT 2"));
        assert!(sql.contains("VACUUM t"));
    }

    // --- check_reachability_floor ---

    #[test]
    fn reachability_floor_ok_at_exact_threshold() {
        // 90 of 100 at 90% floor is exactly the boundary — must pass.
        check_reachability_floor(90, 100, 90).unwrap();
    }

    #[test]
    fn reachability_floor_ok_above_threshold() {
        check_reachability_floor(95, 100, 90).unwrap();
        check_reachability_floor(100, 100, 90).unwrap();
    }

    #[test]
    fn reachability_floor_fails_below_threshold() {
        let err = check_reachability_floor(80, 100, 90)
            .unwrap_err()
            .to_string();
        assert!(err.contains("fell below"), "got {err}");
    }

    #[test]
    fn reachability_floor_rejects_zero_reference() {
        assert!(check_reachability_floor(5, 0, 90).is_err());
        assert!(check_reachability_floor(5, -1, 90).is_err());
    }

    #[test]
    fn reachability_floor_large_values_do_not_overflow_i64_multiply() {
        // Without i128 widening, final * 100 overflows for counts in the
        // hundreds of millions. Both sides in i128 must stay sane.
        let big = 500_000_000_i64;
        check_reachability_floor(big, big, 90).unwrap();
    }

    #[test]
    fn reachability_floor_100_percent_requires_exact_match() {
        assert!(check_reachability_floor(99, 100, 100).is_err());
        check_reachability_floor(100, 100, 100).unwrap();
    }
}
