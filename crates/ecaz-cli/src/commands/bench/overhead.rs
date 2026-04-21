//! `ecaz bench overhead` — decompose KNN wall time into internal
//! execution, query-vector encoding, and residual client/protocol
//! overhead.
//!
//! # Approach
//!
//! For each sweep value, per query:
//! - measure *full* wall-clock of `ORDER BY ... encode(...) LIMIT k`
//! - measure *encode-only* wall-clock of `SELECT encode_to_<embedding>(...)`
//!
//! Once per sweep value, run `EXPLAIN (ANALYZE, FORMAT JSON)` on the full
//! KNN and parse the planner-reported `Execution Time` out of the JSON.
//! That gives us a server-side number that excludes network + result
//! marshaling, so:
//!
//!   residual ≈ full_wall − internal_exec
//!
//! The residual bucket captures TCP round-trip + result decoding +
//! prepared-statement bookkeeping; it should shrink as rows-returned
//! shrinks (k small) and grow under network jitter.
//!
//! # Purity boundary
//!
//! `parse_execution_time_ms` and `compute_breakdown` are pure functions
//! with dedicated unit tests. The loop orchestration talks to Postgres
//! and reuses `build_knn_sql` + `summarize` + `percentile_sorted`.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::profiles::{self, IndexProfile};
use crate::psql;

use super::latency::summarize;
use super::recall::build_knn_sql;

#[derive(Args, Debug)]
pub struct OverheadArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
    /// Access-method profile to measure.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for KNN ORDER BY ... LIMIT k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Number of queries sampled per sweep value.
    #[arg(long, default_value_t = 100)]
    pub iterations: usize,
    /// Sweep values for the profile's tuning GUC.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Quantization bits used when encoding (must match loader).
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed (must match loader).
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
}

pub async fn run(database: &str, args: OverheadArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 || args.iterations == 0 {
        return Err(eyre!("--k and --iterations must be >= 1"));
    }
    let profile = profiles::resolve(&args.profile)
        .ok_or_else(|| eyre!("unknown profile {:?}", args.profile))?;
    let guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC", profile.name))?;
    let sweep_values: Vec<i32> = if args.sweep.is_empty() {
        if profile.default_sweep.is_empty() {
            return Err(eyre!(
                "--sweep is required for profile {:?} (no default sweep registered)",
                profile.name
            ));
        }
        eprintln!(
            "[overhead] no --sweep provided; using profile default {:?}",
            profile.default_sweep
        );
        profile.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let full_sql = build_knn_sql(profile, &corpus_table);
    let encode_sql = build_encode_only_sql(profile);
    // EXPLAIN's result columns are `text`, even with FORMAT JSON — so we
    // read them as String and parse client-side with serde_json rather
    // than enabling the tokio-postgres `with-serde_json-1` feature.
    let explain_sql = format!("EXPLAIN (ANALYZE, FORMAT JSON) {full_sql}");

    let client = psql::connect(database).await?;
    if psql::index_count_with_am(&client, &corpus_table, profile.access_method).await? == 0 {
        return Err(eyre!(
            "{} on {:?}",
            super::missing_am_error(profile, profile.access_method),
            corpus_table
        ));
    }
    let query_rows = client
        .query(
            &format!("SELECT source FROM {queries_table} ORDER BY id"),
            &[],
        )
        .await
        .wrap_err_with(|| format!("reading {queries_table}"))?;
    if query_rows.is_empty() {
        return Err(eyre!("{queries_table} is empty"));
    }
    let queries: Vec<Vec<f32>> = query_rows.iter().map(|r| r.get::<_, Vec<f32>>(0)).collect();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        profile.sweep_axis_label(),
        "full p50",
        "full p95",
        "encode p50",
        "internal exec",
        "residual",
        "residual %",
    ]);

    for value in &sweep_values {
        client
            .batch_execute(&format!("SET {guc} = {value}"))
            .await
            .wrap_err_with(|| format!("SET {guc} = {value}"))?;

        // One EXPLAIN per sweep point. Use query 0 as the representative.
        let explain_stmt = client
            .prepare(&explain_sql)
            .await
            .wrap_err("preparing EXPLAIN")?;
        let q0 = &queries[0];
        let explain_rows = client
            .query(
                &explain_stmt,
                &[q0, &args.bits, &args.seed, &(args.k as i64)],
            )
            .await
            .wrap_err("running EXPLAIN")?;
        // EXPLAIN (FORMAT JSON) emits one `text` row per top-level JSON
        // fragment. Join them before parsing in case a future PG splits
        // the output across rows.
        let plan_text: String = explain_rows
            .iter()
            .map(|r| r.get::<_, String>(0))
            .collect::<Vec<_>>()
            .join("\n");
        let plan_json: Value = serde_json::from_str(&plan_text).wrap_err("parsing EXPLAIN JSON")?;
        let internal_ms = parse_execution_time_ms(&plan_json)
            .ok_or_else(|| eyre!("EXPLAIN JSON did not contain Execution Time"))?;

        let full_stmt = client.prepare(&full_sql).await?;
        let encode_stmt = client.prepare(&encode_sql).await?;
        let bar = ProgressBar::new(args.iterations as u64);
        bar.set_style(
            ProgressStyle::with_template("[overhead {msg}] {wide_bar} {pos}/{len}").unwrap(),
        );
        bar.set_message(format!("{guc}={value}"));
        bar.enable_steady_tick(Duration::from_millis(250));

        let mut full_durs = Vec::with_capacity(args.iterations);
        let mut encode_durs = Vec::with_capacity(args.iterations);
        let k_i64 = args.k as i64;
        for i in 0..args.iterations {
            let q = &queries[i % queries.len()];
            let t0 = Instant::now();
            let _ = client
                .query(&full_stmt, &[q, &args.bits, &args.seed, &k_i64])
                .await
                .wrap_err("full KNN query")?;
            full_durs.push(t0.elapsed());
            let t1 = Instant::now();
            let _ = client
                .query(&encode_stmt, &[q, &args.bits, &args.seed])
                .await
                .wrap_err("encode-only query")?;
            encode_durs.push(t1.elapsed());
            bar.inc(1);
        }
        bar.finish_and_clear();

        let full = summarize(&full_durs);
        let encode = summarize(&encode_durs);
        let b = compute_breakdown(full.p50, internal_ms);
        table.add_row(vec![
            Cell::new(value),
            Cell::new(format_ms_dur(full.p50)),
            Cell::new(format_ms_dur(full.p95)),
            Cell::new(format_ms_dur(encode.p50)),
            Cell::new(format_ms(internal_ms)),
            Cell::new(format_ms_dur(b.residual)),
            Cell::new(format!("{:.1}%", b.residual_fraction * 100.0)),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn build_encode_only_sql(profile: &IndexProfile) -> String {
    format!(
        "SELECT {enc}($1::real[], $2::integer, $3::bigint)",
        enc = profile.encoder_function
    )
}

/// Decomposition of a single wall-clock measurement. `full` is the
/// observed client-side duration, `internal` is the planner-reported
/// server-side execution time, `residual` is the leftover attributed to
/// network + result marshaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Breakdown {
    pub full: Duration,
    pub internal: Duration,
    pub residual: Duration,
    pub residual_fraction: f64,
}

/// Compute a `Breakdown` from a client-observed full duration and a
/// planner-reported internal execution time in ms.
///
/// Clamps `residual` to zero when the planner reports a longer time than
/// the client observed (can happen under jitter when the EXPLAIN-based
/// internal time is measured on a different query than the sampled
/// iterations). Clamps `residual_fraction` to [0, 1] accordingly.
pub fn compute_breakdown(full: Duration, internal_ms: f64) -> Breakdown {
    let internal = if internal_ms.is_finite() && internal_ms > 0.0 {
        Duration::from_secs_f64(internal_ms / 1000.0)
    } else {
        Duration::ZERO
    };
    let residual = full.saturating_sub(internal);
    let full_ns = full.as_nanos() as f64;
    let residual_fraction = if full_ns > 0.0 {
        (residual.as_nanos() as f64 / full_ns).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Breakdown {
        full,
        internal,
        residual,
        residual_fraction,
    }
}

/// Walk an EXPLAIN (ANALYZE, FORMAT JSON) result and return the
/// "Execution Time" (milliseconds) from the outermost plan envelope.
///
/// The tokio-postgres driver returns the JSON as either a single object
/// or a one-element array — both shapes are accepted.
pub fn parse_execution_time_ms(plan: &Value) -> Option<f64> {
    let envelope = match plan {
        Value::Array(arr) => arr.first()?,
        other => other,
    };
    envelope.get("Execution Time").and_then(Value::as_f64)
}

fn format_ms_dur(d: Duration) -> String {
    format_ms(d.as_secs_f64() * 1000.0)
}

fn format_ms(ms: f64) -> String {
    if ms >= 10.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{ms:.2} ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{EC_DISKANN, EC_HNSW};
    use serde_json::json;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    // --- parse_execution_time_ms ---

    #[test]
    fn parse_execution_time_from_array_envelope() {
        let plan = json!([{ "Plan": {}, "Execution Time": 12.345 }]);
        assert_eq!(parse_execution_time_ms(&plan), Some(12.345));
    }

    #[test]
    fn parse_execution_time_from_object_envelope() {
        let plan = json!({ "Plan": {}, "Execution Time": 99.0 });
        assert_eq!(parse_execution_time_ms(&plan), Some(99.0));
    }

    #[test]
    fn parse_execution_time_missing_returns_none() {
        let plan = json!({ "Plan": {} });
        assert_eq!(parse_execution_time_ms(&plan), None);
    }

    #[test]
    fn parse_execution_time_non_numeric_returns_none() {
        let plan = json!([{ "Execution Time": "not-a-number" }]);
        assert_eq!(parse_execution_time_ms(&plan), None);
    }

    #[test]
    fn parse_execution_time_accepts_integer_value() {
        // Postgres sometimes emits an integer when the time is exact ms.
        let plan = json!([{ "Execution Time": 7 }]);
        assert_eq!(parse_execution_time_ms(&plan), Some(7.0));
    }

    // --- compute_breakdown ---

    #[test]
    fn breakdown_residual_is_full_minus_internal() {
        let b = compute_breakdown(ms(10), 4.0);
        assert_eq!(b.full, ms(10));
        assert_eq!(b.internal, ms(4));
        assert_eq!(b.residual, ms(6));
        assert!((b.residual_fraction - 0.6).abs() < 1e-9);
    }

    #[test]
    fn breakdown_clamps_negative_residual_to_zero() {
        // internal > full (jitter or measuring different queries)
        let b = compute_breakdown(ms(2), 5.0);
        assert_eq!(b.residual, Duration::ZERO);
        assert_eq!(b.residual_fraction, 0.0);
    }

    #[test]
    fn breakdown_handles_zero_full_without_nan() {
        let b = compute_breakdown(Duration::ZERO, 0.0);
        assert!(b.residual_fraction.is_finite());
        assert_eq!(b.residual, Duration::ZERO);
    }

    #[test]
    fn breakdown_rejects_nan_internal_as_zero() {
        let b = compute_breakdown(ms(10), f64::NAN);
        assert_eq!(b.internal, Duration::ZERO);
        assert_eq!(b.residual, ms(10));
    }

    #[test]
    fn breakdown_rejects_negative_internal_as_zero() {
        let b = compute_breakdown(ms(10), -3.0);
        assert_eq!(b.internal, Duration::ZERO);
    }

    // --- build_encode_only_sql ---

    #[test]
    fn encode_only_sql_uses_profile_encoder() {
        assert!(build_encode_only_sql(&EC_HNSW).contains("encode_to_ecvector"));
        assert!(build_encode_only_sql(&EC_DISKANN).contains("encode_to_ecvector"));
    }
}
