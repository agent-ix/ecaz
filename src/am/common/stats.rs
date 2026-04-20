#[cfg(feature = "pg18")]
use crate::pg18_pgstat_shim;
#[cfg(feature = "pg18")]
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatsSnapshot {
    pub function_name: &'static str,
    pub pg18_pgstat_kind_ready: bool,
    pub pg18_sql_function_ready: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TqStatsCounters {
    pub total_distance_calcs: u64,
    pub total_graph_hops: u64,
    pub total_linear_pages: u64,
    pub total_scans_started: u64,
    pub total_scans_bootstrap_only: u64,
    pub quantizer_cache_hits: u64,
    pub quantizer_cache_misses: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TqStatsSummary {
    pub total_distance_calcs: u64,
    pub total_graph_hops: u64,
    pub total_linear_pages: u64,
    pub total_scans_started: u64,
    pub total_scans_bootstrap_only: u64,
    pub quantizer_cache_hits: u64,
    pub quantizer_cache_misses: u64,
    pub bootstrap_hit_rate: f64,
    pub quantizer_cache_rate: f64,
}

pub(crate) fn stats_snapshot() -> StatsSnapshot {
    StatsSnapshot {
        function_name: "tqvector_stats",
        pg18_pgstat_kind_ready: pgstat_kind_ready(),
        pg18_sql_function_ready: cfg!(feature = "pg18"),
    }
}

#[cfg(feature = "pg18")]
static TOTAL_DISTANCE_CALCS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static TOTAL_GRAPH_HOPS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static TOTAL_LINEAR_PAGES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static TOTAL_SCANS_STARTED: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static TOTAL_SCANS_BOOTSTRAP_ONLY: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static QUANTIZER_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "pg18")]
static QUANTIZER_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

pub(crate) fn pgstat_kind_blocker() -> Option<&'static str> {
    #[cfg(feature = "pg18")]
    {
        Some(
            "custom pgstat kind registration requires loading tqvector via shared_preload_libraries on PG18 and restarting PostgreSQL",
        )
    }

    #[cfg(not(feature = "pg18"))]
    {
        None
    }
}

#[cfg(feature = "pg18")]
fn load_counter(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

#[cfg(feature = "pg18")]
fn increment(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

#[cfg(feature = "pg18")]
fn record_shared_delta(delta: TqStatsCounters) {
    unsafe {
        let _ = pg18_pgstat_shim::record(&delta);
    }
}

#[cfg(feature = "pg18")]
fn pgstat_kind_ready() -> bool {
    unsafe { pg18_pgstat_shim::is_registered() }
}

#[cfg(not(feature = "pg18"))]
fn pgstat_kind_ready() -> bool {
    false
}

#[cfg(feature = "pg18")]
pub(crate) fn current_backend_stats_counters() -> TqStatsCounters {
    TqStatsCounters {
        total_distance_calcs: load_counter(&TOTAL_DISTANCE_CALCS),
        total_graph_hops: load_counter(&TOTAL_GRAPH_HOPS),
        total_linear_pages: load_counter(&TOTAL_LINEAR_PAGES),
        total_scans_started: load_counter(&TOTAL_SCANS_STARTED),
        total_scans_bootstrap_only: load_counter(&TOTAL_SCANS_BOOTSTRAP_ONLY),
        quantizer_cache_hits: load_counter(&QUANTIZER_CACHE_HITS),
        quantizer_cache_misses: load_counter(&QUANTIZER_CACHE_MISSES),
    }
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn current_backend_stats_counters() -> TqStatsCounters {
    TqStatsCounters::default()
}

#[cfg(feature = "pg18")]
pub(crate) fn current_stats_counters() -> TqStatsCounters {
    unsafe { pg18_pgstat_shim::snapshot().unwrap_or_else(current_backend_stats_counters) }
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn current_stats_counters() -> TqStatsCounters {
    current_backend_stats_counters()
}

#[cfg(feature = "pg18")]
pub(crate) fn record_distance_calc() {
    increment(&TOTAL_DISTANCE_CALCS);
    record_shared_delta(TqStatsCounters {
        total_distance_calcs: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_distance_calc() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_graph_hop() {
    increment(&TOTAL_GRAPH_HOPS);
    record_shared_delta(TqStatsCounters {
        total_graph_hops: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_graph_hop() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_linear_page() {
    increment(&TOTAL_LINEAR_PAGES);
    record_shared_delta(TqStatsCounters {
        total_linear_pages: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_linear_page() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_scan_started() {
    increment(&TOTAL_SCANS_STARTED);
    record_shared_delta(TqStatsCounters {
        total_scans_started: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_scan_started() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_bootstrap_only_scan() {
    increment(&TOTAL_SCANS_BOOTSTRAP_ONLY);
    record_shared_delta(TqStatsCounters {
        total_scans_bootstrap_only: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_bootstrap_only_scan() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_quantizer_cache_hit() {
    increment(&QUANTIZER_CACHE_HITS);
    record_shared_delta(TqStatsCounters {
        quantizer_cache_hits: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_quantizer_cache_hit() {}

#[cfg(feature = "pg18")]
pub(crate) fn record_quantizer_cache_miss() {
    increment(&QUANTIZER_CACHE_MISSES);
    record_shared_delta(TqStatsCounters {
        quantizer_cache_misses: 1,
        ..TqStatsCounters::default()
    });
}

#[cfg(not(feature = "pg18"))]
pub(crate) fn record_quantizer_cache_miss() {}

#[cfg(feature = "pg18")]
pub(crate) unsafe fn register_pg18_stats() {
    unsafe {
        let _ = pg18_pgstat_shim::register_kind();
    }
}

impl TqStatsCounters {
    pub(crate) fn record_distance_calc(&mut self) {
        self.total_distance_calcs += 1;
    }

    pub(crate) fn record_graph_hop(&mut self) {
        self.total_graph_hops += 1;
    }

    pub(crate) fn record_linear_page(&mut self) {
        self.total_linear_pages += 1;
    }

    pub(crate) fn record_scan_started(&mut self) {
        self.total_scans_started += 1;
    }

    pub(crate) fn record_bootstrap_only_scan(&mut self) {
        self.total_scans_bootstrap_only += 1;
    }

    pub(crate) fn record_quantizer_cache_hit(&mut self) {
        self.quantizer_cache_hits += 1;
    }

    pub(crate) fn record_quantizer_cache_miss(&mut self) {
        self.quantizer_cache_misses += 1;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn summary(self) -> TqStatsSummary {
        let bootstrap_hit_rate = if self.total_scans_started == 0 {
            0.0
        } else {
            self.total_scans_bootstrap_only as f64 / self.total_scans_started as f64
        };
        let cache_events = self.quantizer_cache_hits + self.quantizer_cache_misses;
        let quantizer_cache_rate = if cache_events == 0 {
            0.0
        } else {
            self.quantizer_cache_hits as f64 / cache_events as f64
        };

        TqStatsSummary {
            total_distance_calcs: self.total_distance_calcs,
            total_graph_hops: self.total_graph_hops,
            total_linear_pages: self.total_linear_pages,
            total_scans_started: self.total_scans_started,
            total_scans_bootstrap_only: self.total_scans_bootstrap_only,
            quantizer_cache_hits: self.quantizer_cache_hits,
            quantizer_cache_misses: self.quantizer_cache_misses,
            bootstrap_hit_rate,
            quantizer_cache_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{stats_snapshot, StatsSnapshot, TqStatsCounters, TqStatsSummary};

    #[test]
    fn stats_snapshot_matches_build_target() {
        assert_eq!(
            stats_snapshot(),
            StatsSnapshot {
                function_name: "tqvector_stats",
                pg18_pgstat_kind_ready: false,
                pg18_sql_function_ready: cfg!(feature = "pg18"),
            }
        );
    }

    #[test]
    fn stats_counters_record_each_staged_metric() {
        let mut counters = TqStatsCounters::default();

        counters.record_distance_calc();
        counters.record_graph_hop();
        counters.record_linear_page();
        counters.record_scan_started();
        counters.record_bootstrap_only_scan();
        counters.record_quantizer_cache_hit();
        counters.record_quantizer_cache_miss();

        assert_eq!(
            counters,
            TqStatsCounters {
                total_distance_calcs: 1,
                total_graph_hops: 1,
                total_linear_pages: 1,
                total_scans_started: 1,
                total_scans_bootstrap_only: 1,
                quantizer_cache_hits: 1,
                quantizer_cache_misses: 1,
            }
        );
    }

    #[test]
    fn stats_counters_reset_to_zero_state() {
        let mut counters = TqStatsCounters {
            total_distance_calcs: 2,
            total_graph_hops: 3,
            total_linear_pages: 5,
            total_scans_started: 7,
            total_scans_bootstrap_only: 11,
            quantizer_cache_hits: 13,
            quantizer_cache_misses: 17,
        };

        counters.reset();

        assert_eq!(counters, TqStatsCounters::default());
    }

    #[test]
    fn stats_summary_reports_derived_rates() {
        let counters = TqStatsCounters {
            total_distance_calcs: 100,
            total_graph_hops: 20,
            total_linear_pages: 5,
            total_scans_started: 10,
            total_scans_bootstrap_only: 8,
            quantizer_cache_hits: 99,
            quantizer_cache_misses: 1,
        };

        assert_eq!(
            counters.summary(),
            TqStatsSummary {
                total_distance_calcs: 100,
                total_graph_hops: 20,
                total_linear_pages: 5,
                total_scans_started: 10,
                total_scans_bootstrap_only: 8,
                quantizer_cache_hits: 99,
                quantizer_cache_misses: 1,
                bootstrap_hit_rate: 0.8,
                quantizer_cache_rate: 0.99,
            }
        );
    }

    #[test]
    fn stats_summary_handles_zero_denominators() {
        assert_eq!(
            TqStatsCounters::default().summary(),
            TqStatsSummary {
                total_distance_calcs: 0,
                total_graph_hops: 0,
                total_linear_pages: 0,
                total_scans_started: 0,
                total_scans_bootstrap_only: 0,
                quantizer_cache_hits: 0,
                quantizer_cache_misses: 0,
                bootstrap_hit_rate: 0.0,
                quantizer_cache_rate: 0.0,
            }
        );
    }
}
