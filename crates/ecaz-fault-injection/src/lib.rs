//! PG-level fault-injection matrix for ECAZ operator smoke lanes.
//!
//! This crate is intentionally free of PostgreSQL client dependencies. It
//! defines the fault model, required coverage, and post-condition probes used by
//! the `ecaz dev fault` CLI and Makefile smoke targets.

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProviderMode {
    EioRead,
    EnospcWrite,
    SlowDisk,
}

impl ProviderMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderMode::EioRead => "eio-read",
            ProviderMode::EnospcWrite => "enospc-write",
            ProviderMode::SlowDisk => "slow-disk",
        }
    }
}

impl fmt::Display for ProviderMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn provider_library_path() -> Option<&'static str> {
    option_env!("ECAZ_FAULT_PROVIDER_SO")
}

pub fn provider_environment(
    mode: ProviderMode,
    path_match: &str,
    after: u64,
    latency_ms: Option<u64>,
    marker: Option<&str>,
) -> Vec<(String, String)> {
    let mut env = vec![
        (
            "LD_PRELOAD".to_string(),
            provider_library_path()
                .unwrap_or("<linux-only provider not built>")
                .to_string(),
        ),
        ("ECAZ_FAULT_PROVIDER_ENABLE".to_string(), "1".to_string()),
        (
            "ECAZ_FAULT_PROVIDER_MODE".to_string(),
            mode.as_str().to_string(),
        ),
        (
            "ECAZ_FAULT_PROVIDER_MATCH".to_string(),
            path_match.to_string(),
        ),
        (
            "ECAZ_FAULT_PROVIDER_AFTER".to_string(),
            after.max(1).to_string(),
        ),
    ];
    if let Some(latency_ms) = latency_ms {
        env.push((
            "ECAZ_FAULT_PROVIDER_LATENCY_MS".to_string(),
            latency_ms.to_string(),
        ));
    }
    if let Some(marker) = marker {
        env.push(("ECAZ_FAULT_PROVIDER_MARKER".to_string(), marker.to_string()));
    }
    env
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FaultLane {
    Io,
    Memory,
    Cancel,
    Timeout,
    LockTimeout,
    Resource,
    SlowDisk,
}

impl FaultLane {
    pub const ALL: [FaultLane; 7] = [
        FaultLane::Io,
        FaultLane::Memory,
        FaultLane::Cancel,
        FaultLane::Timeout,
        FaultLane::LockTimeout,
        FaultLane::Resource,
        FaultLane::SlowDisk,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FaultLane::Io => "io",
            FaultLane::Memory => "memory",
            FaultLane::Cancel => "cancel",
            FaultLane::Timeout => "timeout",
            FaultLane::LockTimeout => "lock-timeout",
            FaultLane::Resource => "resource",
            FaultLane::SlowDisk => "slow-disk",
        }
    }
}

impl fmt::Display for FaultLane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FaultLane {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "io" => Ok(FaultLane::Io),
            "memory" => Ok(FaultLane::Memory),
            "cancel" => Ok(FaultLane::Cancel),
            "timeout" => Ok(FaultLane::Timeout),
            "lock-timeout" | "lock_timeout" => Ok(FaultLane::LockTimeout),
            "resource" => Ok(FaultLane::Resource),
            "slow-disk" | "slow_disk" => Ok(FaultLane::SlowDisk),
            other => Err(format!("unknown fault lane {other:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FaultAm {
    Hnsw,
    Ivf,
    DiskAnn,
    Spire,
}

impl FaultAm {
    pub const ALL: [FaultAm; 4] = [
        FaultAm::Hnsw,
        FaultAm::Ivf,
        FaultAm::DiskAnn,
        FaultAm::Spire,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FaultAm::Hnsw => "ec_hnsw",
            FaultAm::Ivf => "ec_ivf",
            FaultAm::DiskAnn => "ec_diskann",
            FaultAm::Spire => "ec_spire",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaultCase {
    pub id: String,
    pub lane: FaultLane,
    pub access_method: FaultAm,
    pub fault: &'static str,
    pub trigger: &'static str,
    pub expected: &'static str,
    pub postconditions: &'static [&'static str],
}

pub fn required_smoke_cases(lane: FaultLane) -> Vec<FaultCase> {
    FaultAm::ALL
        .into_iter()
        .flat_map(|am| lane_cases(lane, am))
        .collect()
}

pub fn all_smoke_cases() -> Vec<FaultCase> {
    FaultLane::ALL
        .into_iter()
        .flat_map(required_smoke_cases)
        .collect()
}

fn lane_cases(lane: FaultLane, access_method: FaultAm) -> Vec<FaultCase> {
    match lane {
        FaultLane::Io => vec![
            case(
                lane,
                access_method,
                "eio-read",
                "inject EIO on relation read path",
                "clean ERROR; backend remains connected",
            ),
            case(
                lane,
                access_method,
                "enospc-write",
                "inject ENOSPC on page extension or WAL write",
                "clean ERROR; no partial AM-visible tuple",
            ),
        ],
        FaultLane::Memory => vec![case(
            lane,
            access_method,
            "palloc-nth-failure",
            "fail the Nth allocation while the AM callback is active",
            "clean ERROR; Rust guards release PG resources",
        )],
        FaultLane::Cancel => vec![case(
            lane,
            access_method,
            "pg-cancel-backend",
            "cancel the backend while build/scan/insert/vacuum is in progress",
            "query cancels promptly; no leaked pins or locks",
        )],
        FaultLane::Timeout => vec![case(
            lane,
            access_method,
            "statement-timeout",
            "SET statement_timeout low enough to interrupt active AM work",
            "timeout ERROR; all retained AM state is dropped",
        )],
        FaultLane::LockTimeout => vec![case(
            lane,
            access_method,
            "lock-timeout",
            "SET lock_timeout while contended DDL waits on AM relations",
            "lock timeout ERROR; no relation lock survives the session",
        )],
        FaultLane::Resource => vec![case(
            lane,
            access_method,
            "tiny-work-mem",
            "run build/scan with tiny work_mem and maintenance_work_mem",
            "clean ERROR or successful bounded execution; no negative counters",
        )],
        FaultLane::SlowDisk => vec![case(
            lane,
            access_method,
            "latency-injection",
            "delay relation and remote-object reads",
            "operation remains cancellable and timeout-governed",
        )],
    }
}

fn case(
    lane: FaultLane,
    access_method: FaultAm,
    fault: &'static str,
    trigger: &'static str,
    expected: &'static str,
) -> FaultCase {
    FaultCase {
        id: format!("{}-{}-{fault}", access_method.as_str(), lane.as_str()),
        lane,
        access_method,
        fault,
        trigger,
        expected,
        postconditions: &[
            "no pinned buffers retained by the test backend",
            "no surviving relation or advisory locks",
            "no PANIC or FATAL in postmaster log",
            "pgstat counters remain monotonic",
        ],
    }
}

pub fn leak_probe_sql() -> &'static [&'static str] {
    &[
        "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database() AND application_name LIKE 'ecaz-fault-%' AND pid <> pg_backend_pid()",
        "SELECT count(*) FROM pg_locks l JOIN pg_stat_activity a USING (pid) WHERE a.datname = current_database() AND a.application_name LIKE 'ecaz-fault-%' AND a.pid <> pg_backend_pid()",
        "SELECT count(*) FROM pg_prepared_xacts WHERE database = current_database()",
    ]
}

pub fn workload_table(access_method: FaultAm) -> &'static str {
    match access_method {
        FaultAm::Hnsw => "ecaz_fault_hnsw",
        FaultAm::Ivf => "ecaz_fault_ivf",
        FaultAm::DiskAnn => "ecaz_fault_diskann",
        FaultAm::Spire => "ecaz_fault_spire",
    }
}

pub fn workload_index(access_method: FaultAm) -> &'static str {
    match access_method {
        FaultAm::Hnsw => "ecaz_fault_hnsw_idx",
        FaultAm::Ivf => "ecaz_fault_ivf_idx",
        FaultAm::DiskAnn => "ecaz_fault_diskann_idx",
        FaultAm::Spire => "ecaz_fault_spire_idx",
    }
}

pub fn workload_setup_sql(access_method: FaultAm, rows: i64) -> String {
    let table = workload_table(access_method);
    let index = workload_index(access_method);
    let index_sql = match access_method {
        FaultAm::Hnsw => format!(
            "CREATE INDEX {index} ON {table} USING ec_hnsw (embedding ecvector_ip_ops) \
             WITH (m = 8, ef_construction = 16)"
        ),
        FaultAm::Ivf => format!(
            "CREATE INDEX {index} ON {table} USING ec_ivf (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 4, training_sample_rows = {rows}, storage_format = 'turboquant', rerank = 'heap_f32', rerank_width = 10)"
        ),
        FaultAm::DiskAnn => format!(
            "CREATE INDEX {index} ON {table} USING ec_diskann (embedding ecvector_diskann_ip_ops) \
             WITH (graph_degree = 8, build_list_size = 20, list_size = 20, rerank_budget = 8)"
        ),
        FaultAm::Spire => format!(
            "CREATE INDEX {index} ON {table} USING ec_spire (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 4, nprobe = 4, storage_format = 'rabitq')"
        ),
    };
    format!(
        "DROP TABLE IF EXISTS {table} CASCADE;
         CREATE TABLE {table} (
             id bigserial PRIMARY KEY,
             embedding ecvector NOT NULL
         );
         INSERT INTO {table} (embedding)
         SELECT encode_to_ecvector(
             ARRAY[
                 cos((gs * 0.013)::double precision)::real,
                 sin((gs * 0.013)::double precision)::real,
                 0.0::real,
                 0.0::real
             ]::real[],
             4,
             42
         )
         FROM generate_series(1, {rows}) AS gs;
         {index_sql};"
    )
}

pub fn workload_scan_sql(access_method: FaultAm) -> String {
    let table = workload_table(access_method);
    format!(
        "SELECT id FROM {table}
         ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[]
         LIMIT 5"
    )
}

pub fn workload_insert_sql(access_method: FaultAm) -> String {
    let table = workload_table(access_method);
    format!(
        "INSERT INTO {table} (embedding)
         VALUES (encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0]::real[], 4, 42))"
    )
}

pub fn workload_vacuum_sql(access_method: FaultAm) -> String {
    format!("VACUUM (ANALYZE) {}", workload_table(access_method))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_lanes_cover_every_access_method() {
        for lane in FaultLane::ALL {
            let cases = required_smoke_cases(lane);
            for access_method in FaultAm::ALL {
                assert!(
                    cases.iter().any(|case| case.access_method == access_method),
                    "lane {lane} missing {}",
                    access_method.as_str()
                );
            }
        }
    }

    #[test]
    fn io_lane_covers_eio_and_enospc() {
        let cases = required_smoke_cases(FaultLane::Io);
        assert!(cases.iter().any(|case| case.fault == "eio-read"));
        assert!(cases.iter().any(|case| case.fault == "enospc-write"));
    }

    #[test]
    fn parser_accepts_documented_lane_names() {
        for lane in FaultLane::ALL {
            assert_eq!(lane.as_str().parse::<FaultLane>(), Ok(lane));
        }
        assert_eq!(
            "lock_timeout".parse::<FaultLane>(),
            Ok(FaultLane::LockTimeout)
        );
    }

    #[test]
    fn workload_sql_mentions_every_access_method() {
        for am in FaultAm::ALL {
            let sql = workload_setup_sql(am, 16);
            assert!(sql.contains(workload_table(am)));
            assert!(sql.contains(workload_index(am)));
            assert!(sql.contains(am.as_str()));
            assert!(workload_scan_sql(am).contains(workload_table(am)));
            assert!(workload_insert_sql(am).contains(workload_table(am)));
            assert!(workload_vacuum_sql(am).contains(workload_table(am)));
        }
    }

    #[test]
    fn provider_environment_pins_provider_and_mode() {
        let env = provider_environment(
            ProviderMode::EioRead,
            "base/",
            3,
            None,
            Some("/tmp/ecaz-fault-provider.marker"),
        );
        assert!(env.iter().any(|(key, value)| {
            key == "LD_PRELOAD" && (value.ends_with(".so") || value.contains("not built"))
        }));
        assert!(env
            .iter()
            .any(|(key, value)| key == "ECAZ_FAULT_PROVIDER_MODE" && value == "eio-read"));
        assert!(env
            .iter()
            .any(|(key, value)| key == "ECAZ_FAULT_PROVIDER_AFTER" && value == "3"));
    }
}
