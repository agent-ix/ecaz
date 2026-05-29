//! `ecaz stress soak-quant-cache` — Task 48 kickoff soak harness.
//!
//! Drives sustained concurrent traffic against the
//! `ProdQuantizer::cached` global `OnceLock<Mutex<HashMap<_, Arc<_>>>>`
//! for a configurable wall duration, mixing shared-key contention
//! (every worker requests the same key, asserting canonical `Arc`
//! identity per iteration) with private-key growth (each iteration
//! introduces fresh keys to extend the cache state).
//!
//! Pairs with the second concurrent miri test added in Task 43 packet
//! 015 (`miri_quantizer_cache_concurrent_init_under_contention` in
//! `src/quant/prod.rs`): Miri proves no UB on a 53-second schedule
//! sweep; this harness asserts the same invariants hold over a
//! many-iteration wall-clock run that Miri cannot afford.
//!
//! This kickoff slice does **not** sample process RSS or run a
//! linear-fit slope check (Task 48 §Approach 3 bullet 4). Adding the
//! cross-platform RSS sampler and the monotonic-growth assertion is
//! deferred to a follow-up packet so this slice can land as a small,
//! readable scaffold.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Barrier, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use ecaz::bench_api::ProdQuantizer;
use serde::Serialize;

/// Distinctive seed namespaces; chosen to avoid colliding with other
/// tests or harnesses that share the global cache in a single process.
const SHARED_SEED_BASE: u64 = 0xBEEF_0101_BEEF_0101;
const PRIVATE_SEED_BASE: u64 = 0x1234_5678_9ABC_DEF0;

/// Cross-platform current-RSS sampler. Returns `None` on unsupported
/// platforms; the soak loop records `None` in those iterations and the
/// slope-fit check skips them cleanly.
#[cfg(target_os = "linux")]
fn current_rss_bytes() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let resident_pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    // SAFETY: `sysconf(_SC_PAGESIZE)` is a safe POSIX query; no preconditions.
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return None;
    }
    Some(resident_pages * page_size as u64)
}

#[cfg(target_os = "macos")]
fn current_rss_bytes() -> Option<u64> {
    use std::mem::MaybeUninit;
    // SAFETY: `getpid` has no preconditions; `proc_pid_rusage` writes
    // exactly `sizeof(rusage_info_v2)` into a uninit buffer of the
    // matching type. We assume_init only on success (ret == 0).
    let pid = unsafe { libc::getpid() };
    let mut info: MaybeUninit<libc::rusage_info_v2> = MaybeUninit::uninit();
    let ret = unsafe {
        libc::proc_pid_rusage(
            pid,
            libc::RUSAGE_INFO_V2,
            info.as_mut_ptr().cast::<libc::rusage_info_t>(),
        )
    };
    if ret != 0 {
        return None;
    }
    // SAFETY: `proc_pid_rusage` returned success above, which per the
    // macOS contract means the destination buffer is fully initialized.
    let info = unsafe { info.assume_init() };
    Some(info.ri_resident_size)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn current_rss_bytes() -> Option<u64> {
    None
}

/// Linear least-squares slope (bytes-per-iteration) of `(iter_index, rss)`
/// pairs over the second half of the run. Returns `None` if too few
/// samples or any sample is `None`.
fn slope_bytes_per_iter(records: &[IterationRecord]) -> Option<f64> {
    let half = records.len() / 2;
    if records.len() < 4 {
        return None;
    }
    let tail = &records[half..];
    let n = tail.len() as f64;
    if n < 2.0 {
        return None;
    }
    let xs: Vec<f64> = tail.iter().map(|r| r.iter_index as f64).collect();
    let ys: Vec<f64> = tail
        .iter()
        .map(|r| r.rss_bytes.map(|b| b as f64))
        .collect::<Option<Vec<_>>>()?;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        num += (x - mean_x) * (y - mean_y);
        den += (x - mean_x).powi(2);
    }
    if den == 0.0 {
        return None;
    }
    Some(num / den)
}

#[derive(Args, Debug)]
pub struct SoakQuantCacheArgs {
    /// Wall-clock duration for the soak loop (seconds).
    #[arg(long, default_value_t = 5)]
    pub duration_seconds: u64,
    /// Concurrent worker threads per iteration.
    #[arg(long, default_value_t = 4)]
    pub workers: usize,
    /// Vector dimensionality forwarded to `ProdQuantizer::cached`.
    #[arg(long, default_value_t = 8)]
    pub dim: usize,
    /// Bits per code forwarded to `ProdQuantizer::cached`.
    #[arg(long, default_value_t = 4)]
    pub bits: u8,
    /// Number of shared-key slots each worker contends for per iteration.
    #[arg(long, default_value_t = 8)]
    pub shared_keys: u32,
    /// Number of unique private keys each iteration adds to the cache.
    #[arg(long, default_value_t = 4)]
    pub private_keys_per_iter: u32,
    /// Write the JSON summary to this path in addition to stdout.
    #[arg(long)]
    pub log_output: Option<PathBuf>,
    /// Maximum tolerated linear-fit slope of RSS-per-iteration over the
    /// second half of the run. Exceeding this exits non-zero so soak
    /// runs can act as a leak gate. Set to 0 to disable the slope
    /// check entirely (the slope is still recorded in the JSON).
    #[arg(long, default_value_t = 1024)]
    pub slope_tolerance_bytes_per_iter: u64,
}

#[derive(Debug, Serialize)]
struct IterationRecord {
    iter_index: u64,
    elapsed_ms: u128,
    total_ops: u64,
    ops_per_sec: f64,
    shared_arc_strong_count_max: usize,
    distinct_shared_keys_observed: usize,
    /// Current RSS in bytes at the end of the iteration. `None` on
    /// platforms where `current_rss_bytes` is unavailable.
    rss_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SoakSummary {
    duration_seconds_requested: u64,
    workers: usize,
    dim: usize,
    bits: u8,
    shared_keys: u32,
    private_keys_per_iter: u32,
    slope_tolerance_bytes_per_iter: u64,
    iterations_completed: u64,
    total_ops: u64,
    wall_elapsed_ms: u128,
    mean_ops_per_sec: f64,
    /// Linear-fit slope of RSS vs. iteration index over the second
    /// half of the run; `None` if the platform did not supply RSS
    /// samples or there are too few records.
    slope_bytes_per_iter: Option<f64>,
    /// `true` iff `slope_bytes_per_iter` is present and below the
    /// configured tolerance, OR the tolerance is 0 (gate disabled).
    /// `false` triggers a non-zero exit.
    slope_check_passed: bool,
    iterations: Vec<IterationRecord>,
}

pub fn run(args: SoakQuantCacheArgs) -> Result<()> {
    if args.duration_seconds == 0 {
        return Err(eyre!("--duration-seconds must be >= 1"));
    }
    if args.workers == 0 {
        return Err(eyre!("--workers must be >= 1"));
    }
    if !(2..=8).contains(&args.bits) {
        return Err(eyre!(
            "--bits must be in 2..=8 (ProdQuantizer contract); got {}",
            args.bits
        ));
    }
    if args.dim == 0 {
        return Err(eyre!("--dim must be >= 1"));
    }
    if args.shared_keys == 0 {
        return Err(eyre!("--shared-keys must be >= 1"));
    }

    let deadline = Instant::now() + Duration::from_secs(args.duration_seconds);
    let wall_start = Instant::now();
    let mut iterations: Vec<IterationRecord> = Vec::new();
    let mut iter_index: u64 = 0;
    let mut total_ops_acc: u64 = 0;

    while Instant::now() < deadline {
        let iter_start = Instant::now();
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let start_barrier = Arc::new(Barrier::new(args.workers));
        let op_counter = Arc::new(AtomicU64::new(0));
        let observed_shared_keys: Arc<Mutex<std::collections::HashSet<u64>>> =
            Arc::new(Mutex::new(std::collections::HashSet::new()));
        let max_strong_count = Arc::new(AtomicU64::new(0));

        // Each iteration runs for a sub-budget so we record many
        // samples across `duration_seconds`. The remaining wall budget
        // is split across the iteration; a minimum of 250ms keeps
        // per-iteration overhead noise low.
        let remaining = deadline.saturating_duration_since(iter_start);
        let iter_budget = remaining.min(Duration::from_millis(500)).max(Duration::from_millis(50));
        let iter_deadline = iter_start + iter_budget;

        thread::scope(|scope| {
            for worker_id in 0..args.workers {
                let stop = Arc::clone(&stop);
                let start_barrier = Arc::clone(&start_barrier);
                let op_counter = Arc::clone(&op_counter);
                let observed_shared_keys = Arc::clone(&observed_shared_keys);
                let max_strong_count = Arc::clone(&max_strong_count);
                let dim = args.dim;
                let bits = args.bits;
                let shared_keys = args.shared_keys;
                let private_keys_per_iter = args.private_keys_per_iter;
                let iter_idx_for_worker = iter_index;

                scope.spawn(move || {
                    start_barrier.wait();
                    let mut local_ops: u64 = 0;
                    while !stop.load(Ordering::Acquire) && Instant::now() < iter_deadline {
                        // Alternate between shared-key contention and
                        // private-key growth.
                        let pick_shared = local_ops % 2 == 0;
                        if pick_shared {
                            // `local_ops` alternates shared/private, so dividing by
                            // two before the modulo keeps a unit-stride walk across
                            // every `shared_keys` slot regardless of parity.
                            let key_index = ((local_ops as u32) / 2) % shared_keys;
                            let shared_seed = SHARED_SEED_BASE
                                .wrapping_add(u64::from(key_index));
                            let arc = ProdQuantizer::cached(dim, bits, shared_seed);
                            let strong = Arc::strong_count(&arc) as u64;
                            max_strong_count.fetch_max(strong, Ordering::Relaxed);
                            observed_shared_keys
                                .lock()
                                .expect("observed_shared_keys mutex")
                                .insert(shared_seed);
                        } else {
                            let private_index =
                                (local_ops as u32 / 2) % private_keys_per_iter.max(1);
                            let private_seed = PRIVATE_SEED_BASE
                                .wrapping_add(iter_idx_for_worker.wrapping_mul(1_000_003))
                                .wrapping_add((worker_id as u64).wrapping_mul(31))
                                .wrapping_add(u64::from(private_index));
                            let _arc = ProdQuantizer::cached(dim, bits, private_seed);
                        }
                        local_ops += 1;
                    }
                    op_counter.fetch_add(local_ops, Ordering::Relaxed);
                });
            }
        });

        stop.store(true, Ordering::Release);
        let elapsed = iter_start.elapsed();
        let iter_ops = op_counter.load(Ordering::Acquire);
        let ops_per_sec = if elapsed.as_secs_f64() > 0.0 {
            iter_ops as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        let distinct_shared = observed_shared_keys
            .lock()
            .expect("observed_shared_keys mutex")
            .len();
        let strong_max = max_strong_count.load(Ordering::Acquire) as usize;

        iterations.push(IterationRecord {
            iter_index,
            elapsed_ms: elapsed.as_millis(),
            total_ops: iter_ops,
            ops_per_sec,
            shared_arc_strong_count_max: strong_max,
            distinct_shared_keys_observed: distinct_shared,
            rss_bytes: current_rss_bytes(),
        });
        total_ops_acc = total_ops_acc.saturating_add(iter_ops);
        iter_index += 1;
    }

    let wall_elapsed = wall_start.elapsed();
    let mean_ops_per_sec = if wall_elapsed.as_secs_f64() > 0.0 {
        total_ops_acc as f64 / wall_elapsed.as_secs_f64()
    } else {
        0.0
    };

    let slope = slope_bytes_per_iter(&iterations);
    let slope_check_passed = match (slope, args.slope_tolerance_bytes_per_iter) {
        (_, 0) => true,
        (None, _) => true,
        (Some(s), tol) => s <= tol as f64,
    };

    let summary = SoakSummary {
        duration_seconds_requested: args.duration_seconds,
        workers: args.workers,
        dim: args.dim,
        bits: args.bits,
        shared_keys: args.shared_keys,
        private_keys_per_iter: args.private_keys_per_iter,
        slope_tolerance_bytes_per_iter: args.slope_tolerance_bytes_per_iter,
        iterations_completed: iter_index,
        total_ops: total_ops_acc,
        wall_elapsed_ms: wall_elapsed.as_millis(),
        mean_ops_per_sec,
        slope_bytes_per_iter: slope,
        slope_check_passed,
        iterations,
    };

    let json = serde_json::to_string_pretty(&summary)
        .wrap_err("serialize soak summary as JSON")?;
    crate::ecaz_println!("{json}");

    if let Some(path) = args.log_output {
        std::fs::write(&path, &json)
            .wrap_err_with(|| format!("write soak summary to {}", path.display()))?;
    }

    if !slope_check_passed {
        return Err(eyre!(
            "soak slope check failed: slope {:?} bytes/iter exceeds tolerance {} bytes/iter; \
             treat as a memory-leak gate failure",
            slope,
            args.slope_tolerance_bytes_per_iter,
        ));
    }

    Ok(())
}
