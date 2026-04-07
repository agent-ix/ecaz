#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatsSnapshot {
    pub function_name: &'static str,
    pub pg18_pgstat_kind_ready: bool,
    pub pg18_sql_function_ready: bool,
}

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

pub(crate) fn stats_snapshot() -> StatsSnapshot {
    StatsSnapshot {
        function_name: "tqvector_stats",
        pg18_pgstat_kind_ready: false,
        pg18_sql_function_ready: false,
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
}

#[cfg(test)]
mod tests {
    use super::{stats_snapshot, StatsSnapshot, TqStatsCounters};

    #[test]
    fn stats_snapshot_stays_explicitly_unwired_until_pg18_support_exists() {
        assert_eq!(
            stats_snapshot(),
            StatsSnapshot {
                function_name: "tqvector_stats",
                pg18_pgstat_kind_ready: false,
                pg18_sql_function_ready: false,
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
}
