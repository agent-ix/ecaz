//! `ecaz bench latency` — wall-clock p50/p95/p99 for KNN SQL at k.
//!
//! # Flow
//!
//! 1. Connect, validate profile + prefix + tuning GUC.
//! 2. Load `--iterations` query vectors from `<prefix>_queries.source`
//!    (round-robined if iterations > queries).
//! 3. Spawn `--concurrency` workers, each pulling from a shared counter
//!    and running the same prepared KNN statement.
//! 4. Merge per-worker duration buffers, emit one comfy-table row per
//!    sweep value: count, mean, stddev, min, p50, p95, p99, max.
//!
//! # Purity boundary
//!
//! `percentile` and `summarize` are pure functions over `&[Duration]`.
//! The orchestration (`run`) is a thin DB shell on top; live-Postgres
//! coverage lands with the integration suite.

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio_postgres::NoTls;

use crate::profiles;
use crate::psql;

use super::recall::build_knn_sql;

#[derive(Args, Debug)]
pub struct LatencyArgs {
    /// Prefix identifying the corpus.
    #[arg(long)]
    pub prefix: String,
    /// Access-method profile to measure.
    #[arg(long, default_value = "ec_hnsw")]
    pub profile: String,
    /// k for KNN ORDER BY ... LIMIT k.
    #[arg(long, default_value_t = 10)]
    pub k: usize,
    /// Number of concurrent worker connections.
    #[arg(long, default_value_t = 1)]
    pub concurrency: usize,
    /// Total number of queries to run per sweep value.
    #[arg(long, default_value_t = 1000)]
    pub iterations: usize,
    /// Sweep values for the profile's tuning GUC. Accepts `--sweep 100,200`
    /// or repeated `--sweep 100 --sweep 200`.
    #[arg(long, value_delimiter = ',')]
    pub sweep: Vec<i32>,
    /// Quantization bits used when encoding query vectors (must match loader).
    #[arg(long, default_value_t = 4)]
    pub bits: i32,
    /// Quantizer seed (must match loader).
    #[arg(long, default_value_t = 42)]
    pub seed: i64,
}

pub async fn run(database: &str, args: LatencyArgs) -> Result<()> {
    profiles::validate_ident(&args.prefix)
        .wrap_err_with(|| format!("invalid prefix {:?}", args.prefix))?;
    if args.k == 0 || args.iterations == 0 || args.concurrency == 0 {
        return Err(eyre!("--k, --iterations, --concurrency must all be >= 1"));
    }
    let profile = profiles::resolve(&args.profile).ok_or_else(|| {
        eyre!(
            "unknown profile {:?}; try {}",
            args.profile,
            profiles::names().join(", ")
        )
    })?;
    let guc = profile
        .ef_search_guc
        .ok_or_else(|| eyre!("profile {:?} has no tuning GUC to sweep", profile.name))?;
    let sweep_values: Vec<i32> = if args.sweep.is_empty() {
        if profile.default_sweep.is_empty() {
            return Err(eyre!(
                "--sweep is required for profile {:?} (no default sweep registered)",
                profile.name
            ));
        }
        eprintln!(
            "[latency] no --sweep provided; using profile default {:?}",
            profile.default_sweep
        );
        profile.default_sweep.to_vec()
    } else {
        args.sweep.clone()
    };

    let corpus_table = format!("{}_corpus", args.prefix);
    let queries_table = format!("{}_queries", args.prefix);
    let sql = build_knn_sql(profile, &corpus_table);

    // Pull query vectors once into memory. Iterations > n_queries wraps.
    let bootstrap = psql::connect(database).await?;
    if psql::index_count_with_am(&bootstrap, &corpus_table, profile.access_method).await? == 0 {
        return Err(eyre!(
            "{} on {:?}",
            super::missing_am_error(profile, profile.access_method),
            corpus_table
        ));
    }
    let rows = bootstrap
        .query(
            &format!("SELECT source FROM {queries_table} ORDER BY id"),
            &[],
        )
        .await
        .wrap_err_with(|| format!("reading {queries_table}"))?;
    if rows.is_empty() {
        return Err(eyre!("{queries_table} is empty"));
    }
    let queries: Arc<Vec<Vec<f32>>> =
        Arc::new(rows.iter().map(|r| r.get::<_, Vec<f32>>(0)).collect());
    drop(bootstrap);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        profile.sweep_axis_label(),
        "count",
        "mean",
        "stddev",
        "min",
        "p50",
        "p95",
        "p99",
        "max",
    ]);

    for value in &sweep_values {
        let durations = run_sweep_point(
            database,
            guc,
            *value,
            &sql,
            Arc::clone(&queries),
            args.concurrency,
            args.iterations,
            args.bits,
            args.seed,
            args.k,
        )
        .await?;
        let stats = summarize(&durations);
        table.add_row(vec![
            Cell::new(value),
            Cell::new(stats.count),
            Cell::new(format_ms(stats.mean)),
            Cell::new(format_ms(stats.stddev)),
            Cell::new(format_ms(stats.min)),
            Cell::new(format_ms(stats.p50)),
            Cell::new(format_ms(stats.p95)),
            Cell::new(format_ms(stats.p99)),
            Cell::new(format_ms(stats.max)),
        ]);
    }
    println!("{table}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_sweep_point(
    database: &str,
    guc: &str,
    value: i32,
    sql: &str,
    queries: Arc<Vec<Vec<f32>>>,
    concurrency: usize,
    iterations: usize,
    bits: i32,
    seed: i64,
    k: usize,
) -> Result<Vec<Duration>> {
    let bar = ProgressBar::new(iterations as u64);
    bar.set_style(
        ProgressStyle::with_template("[latency {msg}] {wide_bar} {pos}/{len} ({per_sec})").unwrap(),
    );
    bar.set_message(format!("{guc}={value}"));
    bar.enable_steady_tick(Duration::from_millis(250));
    let bar = Arc::new(bar);

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let database = database.to_owned();
        let guc = guc.to_owned();
        let sql = sql.to_owned();
        let queries = Arc::clone(&queries);
        let counter = Arc::clone(&counter);
        let bar = Arc::clone(&bar);
        handles.push(tokio::spawn(async move {
            worker(
                database, guc, value, sql, queries, counter, iterations, bits, seed, k, bar,
            )
            .await
        }));
    }

    let mut merged: Vec<Duration> = Vec::with_capacity(iterations);
    for h in handles {
        let durs = h.await.map_err(|e| eyre!("worker panicked: {e}"))??;
        merged.extend(durs);
    }
    bar.finish_and_clear();
    Ok(merged)
}

#[allow(clippy::too_many_arguments)]
async fn worker(
    database: String,
    guc: String,
    value: i32,
    sql: String,
    queries: Arc<Vec<Vec<f32>>>,
    counter: Arc<AtomicUsize>,
    iterations: usize,
    bits: i32,
    seed: i64,
    k: usize,
    bar: Arc<ProgressBar>,
) -> Result<Vec<Duration>> {
    // Each worker needs its own connection so the session-local GUC sticks.
    let mut config = tokio_postgres::Config::new();
    config.dbname(&database);
    if let Ok(v) = std::env::var("PGHOST") {
        config.host(&v);
    }
    if let Ok(v) = std::env::var("PGPORT") {
        if let Ok(p) = v.parse() {
            config.port(p);
        }
    }
    if let Ok(v) = std::env::var("PGUSER") {
        config.user(&v);
    }
    if let Ok(v) = std::env::var("PGPASSWORD") {
        config.password(&v);
    }
    let (client, connection) = config.connect(NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error = %e, "latency worker connection failed");
        }
    });
    client
        .batch_execute(&format!("SET {guc} = {value}"))
        .await?;
    let stmt = client.prepare(&sql).await?;
    let k_i64 = k as i64;

    let mut durations = Vec::new();
    loop {
        let idx = counter.fetch_add(1, Ordering::Relaxed);
        if idx >= iterations {
            return Ok(durations);
        }
        let q = &queries[idx % queries.len()];
        let t0 = Instant::now();
        let _ = client.query(&stmt, &[q, &bits, &seed, &k_i64]).await?;
        durations.push(t0.elapsed());
        bar.inc(1);
    }
}

/// Fixed-field summary of a latency sample. All durations are in the
/// Duration type so the caller decides how to format; `summarize` never
/// looks at wall time on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyStats {
    pub count: usize,
    pub mean: Duration,
    pub stddev: Duration,
    pub min: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
}

impl LatencyStats {
    pub const ZERO: Self = Self {
        count: 0,
        mean: Duration::ZERO,
        stddev: Duration::ZERO,
        min: Duration::ZERO,
        p50: Duration::ZERO,
        p95: Duration::ZERO,
        p99: Duration::ZERO,
        max: Duration::ZERO,
    };
}

/// Summarise a sample of latencies. Percentiles use linear interpolation
/// between the two nearest ranks (numpy's default). An empty input returns
/// `LatencyStats::ZERO` — the caller decides whether to render that.
pub fn summarize(durations: &[Duration]) -> LatencyStats {
    if durations.is_empty() {
        return LatencyStats::ZERO;
    }
    let mut sorted_ns: Vec<f64> = durations.iter().map(|d| d.as_nanos() as f64).collect();
    sorted_ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let count = sorted_ns.len();
    let mean_ns = sorted_ns.iter().sum::<f64>() / count as f64;
    let var = sorted_ns
        .iter()
        .map(|x| {
            let d = x - mean_ns;
            d * d
        })
        .sum::<f64>()
        / count as f64;
    let stddev_ns = var.sqrt();
    LatencyStats {
        count,
        mean: ns_to_duration(mean_ns),
        stddev: ns_to_duration(stddev_ns),
        min: ns_to_duration(sorted_ns[0]),
        p50: ns_to_duration(percentile_sorted(&sorted_ns, 0.50)),
        p95: ns_to_duration(percentile_sorted(&sorted_ns, 0.95)),
        p99: ns_to_duration(percentile_sorted(&sorted_ns, 0.99)),
        max: ns_to_duration(sorted_ns[count - 1]),
    }
}

/// Linear-interpolated percentile from a pre-sorted ascending sample.
/// `p` is in [0, 1]; out-of-range values are clamped so a caller passing
/// `0.95` vs `95.0` never produces a panic.
pub fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 1.0);
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let rank = p * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = rank - lo as f64;
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }
}

fn ns_to_duration(ns: f64) -> Duration {
    if !ns.is_finite() || ns < 0.0 {
        return Duration::ZERO;
    }
    Duration::from_nanos(ns.round() as u64)
}

fn format_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 10.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{ms:.2} ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    // --- percentile_sorted ---

    #[test]
    fn percentile_sorted_empty_is_zero() {
        assert_eq!(percentile_sorted(&[], 0.5), 0.0);
    }

    #[test]
    fn percentile_sorted_single_value_is_that_value_for_any_p() {
        assert_eq!(percentile_sorted(&[42.0], 0.0), 42.0);
        assert_eq!(percentile_sorted(&[42.0], 0.5), 42.0);
        assert_eq!(percentile_sorted(&[42.0], 1.0), 42.0);
    }

    #[test]
    fn percentile_sorted_endpoints_hit_extremes() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile_sorted(&v, 0.0), 1.0);
        assert_eq!(percentile_sorted(&v, 1.0), 5.0);
    }

    #[test]
    fn percentile_sorted_linear_interpolates_between_ranks() {
        // Matches numpy.percentile([1,2,3,4], [50, 95]) = [2.5, 3.85]
        let v = vec![1.0, 2.0, 3.0, 4.0];
        assert!((percentile_sorted(&v, 0.50) - 2.5).abs() < 1e-9);
        assert!((percentile_sorted(&v, 0.95) - 3.85).abs() < 1e-9);
    }

    #[test]
    fn percentile_sorted_clamps_out_of_range_p() {
        let v = vec![1.0, 2.0, 3.0];
        assert_eq!(percentile_sorted(&v, -0.5), 1.0);
        assert_eq!(percentile_sorted(&v, 95.0), 3.0);
    }

    // --- summarize ---

    #[test]
    fn summarize_empty_returns_zero_stats() {
        assert_eq!(summarize(&[]), LatencyStats::ZERO);
    }

    #[test]
    fn summarize_single_value_has_zero_stddev_and_equal_percentiles() {
        let s = summarize(&[ms(5)]);
        assert_eq!(s.count, 1);
        assert_eq!(s.mean, ms(5));
        assert_eq!(s.stddev, Duration::ZERO);
        assert_eq!(s.min, ms(5));
        assert_eq!(s.max, ms(5));
        assert_eq!(s.p50, ms(5));
        assert_eq!(s.p99, ms(5));
    }

    #[test]
    fn summarize_is_independent_of_input_order() {
        let asc = [ms(1), ms(2), ms(3), ms(4), ms(5)];
        let desc = [ms(5), ms(4), ms(3), ms(2), ms(1)];
        assert_eq!(summarize(&asc), summarize(&desc));
    }

    #[test]
    fn summarize_mean_and_min_max_match_raw_sample() {
        let sample: Vec<Duration> = (1..=100).map(ms).collect();
        let s = summarize(&sample);
        assert_eq!(s.count, 100);
        assert_eq!(s.min, ms(1));
        assert_eq!(s.max, ms(100));
        // Mean of 1..=100 = 50.5 ms — allow slight rounding into whole ns.
        let mean_ms = s.mean.as_secs_f64() * 1000.0;
        assert!((mean_ms - 50.5).abs() < 0.001, "mean={mean_ms}");
    }

    #[test]
    fn summarize_stddev_matches_population_formula() {
        // sample = [1, 2, 3, 4, 5] ms → pop variance = 2.0, stddev = sqrt(2)
        let s = summarize(&[ms(1), ms(2), ms(3), ms(4), ms(5)]);
        let stddev_ms = s.stddev.as_secs_f64() * 1000.0;
        assert!(
            (stddev_ms - (2.0_f64).sqrt()).abs() < 1e-6,
            "stddev={stddev_ms}"
        );
    }

    #[test]
    fn summarize_p50_is_the_median() {
        let s = summarize(&[ms(1), ms(2), ms(3), ms(4), ms(5)]);
        assert_eq!(s.p50, ms(3));
    }

    // --- format_ms / ns_to_duration ---

    #[test]
    fn format_ms_switches_precision_at_10ms_boundary() {
        assert_eq!(format_ms(Duration::from_micros(4_567)), "4.57 ms");
        assert_eq!(format_ms(Duration::from_millis(150)), "150.0 ms");
    }

    #[test]
    fn ns_to_duration_rejects_nan_and_negative() {
        assert_eq!(ns_to_duration(f64::NAN), Duration::ZERO);
        assert_eq!(ns_to_duration(-1.0), Duration::ZERO);
        assert_eq!(ns_to_duration(f64::INFINITY), Duration::ZERO);
    }
}
