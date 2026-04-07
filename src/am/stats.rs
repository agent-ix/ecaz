#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatsSnapshot {
    pub function_name: &'static str,
    pub pg18_pgstat_kind_ready: bool,
    pub pg18_sql_function_ready: bool,
}

pub(crate) fn stats_snapshot() -> StatsSnapshot {
    StatsSnapshot {
        function_name: "tqvector_stats",
        pg18_pgstat_kind_ready: false,
        pg18_sql_function_ready: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{stats_snapshot, StatsSnapshot};

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
}
