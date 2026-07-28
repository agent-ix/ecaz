//! PG-level fault-injection matrix for ECAZ operator smoke lanes.
//!
//! This crate is intentionally free of PostgreSQL client dependencies. It
//! defines the fault model, required coverage, and post-condition probes used by
//! the `ecaz dev fault` CLI and Makefile smoke targets.

use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProviderMode {
    EioRead,
    EnospcWrite,
    SlowDisk,
    SocketReset,
    SocketSlow,
}

impl ProviderMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderMode::EioRead => "eio-read",
            ProviderMode::EnospcWrite => "enospc-write",
            ProviderMode::SlowDisk => "slow-disk",
            ProviderMode::SocketReset => "socket-reset",
            ProviderMode::SocketSlow => "socket-slow",
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
    arm_file: Option<&str>,
    peer_match: Option<&str>,
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
    if let Some(arm_file) = arm_file {
        env.push((
            "ECAZ_FAULT_PROVIDER_ARM_FILE".to_string(),
            arm_file.to_string(),
        ));
    }
    if let Some(peer_match) = peer_match {
        env.push((
            "ECAZ_FAULT_PROVIDER_PEER".to_string(),
            peer_match.to_string(),
        ));
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
    DistAnn,
}

impl FaultAm {
    pub const ALL: [FaultAm; 5] = [
        FaultAm::Hnsw,
        FaultAm::Ivf,
        FaultAm::DiskAnn,
        FaultAm::Spire,
        FaultAm::DistAnn,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FaultAm::Hnsw => "ec_hnsw",
            FaultAm::Ivf => "ec_ivf",
            FaultAm::DiskAnn => "ec_diskann",
            FaultAm::Spire => "ec_spire",
            FaultAm::DistAnn => "ec_distann",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DistannCodec {
    RaBitQ,
    TurboQuant,
    GroupedPq,
}

impl DistannCodec {
    pub const ALL: [DistannCodec; 3] = [
        DistannCodec::RaBitQ,
        DistannCodec::TurboQuant,
        DistannCodec::GroupedPq,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            DistannCodec::RaBitQ => "rabitq",
            DistannCodec::TurboQuant => "turboquant",
            DistannCodec::GroupedPq => "grouped_pq",
        }
    }
}

impl fmt::Display for DistannCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DistannCodec {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "rabitq" | "ra-bit-q" => Ok(DistannCodec::RaBitQ),
            "turboquant" | "turbo-quant" => Ok(DistannCodec::TurboQuant),
            "grouped_pq" | "grouped-pq" => Ok(DistannCodec::GroupedPq),
            other => Err(format!("unknown ec_distann codec {other:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FaultFixture {
    Hnsw,
    Ivf,
    DiskAnn,
    Spire,
    DistAnn(DistannCodec),
}

impl FaultFixture {
    pub const ALL: [FaultFixture; 7] = [
        FaultFixture::Hnsw,
        FaultFixture::Ivf,
        FaultFixture::DiskAnn,
        FaultFixture::Spire,
        FaultFixture::DistAnn(DistannCodec::RaBitQ),
        FaultFixture::DistAnn(DistannCodec::TurboQuant),
        FaultFixture::DistAnn(DistannCodec::GroupedPq),
    ];

    pub const fn access_method(self) -> FaultAm {
        match self {
            FaultFixture::Hnsw => FaultAm::Hnsw,
            FaultFixture::Ivf => FaultAm::Ivf,
            FaultFixture::DiskAnn => FaultAm::DiskAnn,
            FaultFixture::Spire => FaultAm::Spire,
            FaultFixture::DistAnn(_) => FaultAm::DistAnn,
        }
    }

    pub const fn codec(self) -> Option<DistannCodec> {
        match self {
            FaultFixture::DistAnn(codec) => Some(codec),
            FaultFixture::Hnsw
            | FaultFixture::Ivf
            | FaultFixture::DiskAnn
            | FaultFixture::Spire => None,
        }
    }

    pub fn for_access_method(access_method: FaultAm) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|fixture| fixture.access_method() == access_method)
            .collect()
    }

    pub fn slug(self) -> &'static str {
        match self {
            FaultFixture::Hnsw => "hnsw",
            FaultFixture::Ivf => "ivf",
            FaultFixture::DiskAnn => "diskann",
            FaultFixture::Spire => "spire",
            FaultFixture::DistAnn(DistannCodec::RaBitQ) => "distann_rabitq",
            FaultFixture::DistAnn(DistannCodec::TurboQuant) => "distann_turboquant",
            FaultFixture::DistAnn(DistannCodec::GroupedPq) => "distann_grouped_pq",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FaultFixture::Hnsw => "ec_hnsw",
            FaultFixture::Ivf => "ec_ivf",
            FaultFixture::DiskAnn => "ec_diskann",
            FaultFixture::Spire => "ec_spire",
            FaultFixture::DistAnn(DistannCodec::RaBitQ) => "ec_distann/rabitq",
            FaultFixture::DistAnn(DistannCodec::TurboQuant) => "ec_distann/turboquant",
            FaultFixture::DistAnn(DistannCodec::GroupedPq) => "ec_distann/grouped_pq",
        }
    }

    pub fn dimensions(self) -> usize {
        match self {
            FaultFixture::DistAnn(DistannCodec::TurboQuant) => 1536,
            FaultFixture::DistAnn(DistannCodec::RaBitQ | DistannCodec::GroupedPq) => 64,
            FaultFixture::Hnsw
            | FaultFixture::Ivf
            | FaultFixture::DiskAnn
            | FaultFixture::Spire => 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaultCase {
    pub id: String,
    pub lane: FaultLane,
    pub access_method: FaultAm,
    pub codec: Option<DistannCodec>,
    pub fault: &'static str,
    pub trigger: &'static str,
    pub expected: &'static str,
    pub postconditions: &'static [&'static str],
}

pub fn required_smoke_cases(lane: FaultLane) -> Vec<FaultCase> {
    FaultFixture::ALL
        .into_iter()
        .flat_map(|fixture| lane_cases(lane, fixture.access_method(), fixture.codec()))
        .collect()
}

pub fn all_smoke_cases() -> Vec<FaultCase> {
    FaultLane::ALL
        .into_iter()
        .flat_map(required_smoke_cases)
        .collect()
}

fn lane_cases(
    lane: FaultLane,
    access_method: FaultAm,
    codec: Option<DistannCodec>,
) -> Vec<FaultCase> {
    match lane {
        FaultLane::Io => vec![
            case(
                lane,
                access_method,
                codec,
                "eio-read",
                "inject EIO on relation read path",
                "clean ERROR; backend remains connected",
            ),
            case(
                lane,
                access_method,
                codec,
                "enospc-write",
                "inject ENOSPC on page extension or WAL write",
                "clean ERROR; no partial AM-visible tuple",
            ),
        ],
        FaultLane::Memory => vec![
            case(
                lane,
                access_method,
                codec,
                "palloc-nth-failure",
                "fail the Nth allocation while the AM callback is active",
                "clean ERROR; Rust guards release PG resources",
            ),
            case(
                lane,
                access_method,
                codec,
                "backend-sigkill-oom-proxy",
                "SIGKILL the backend while build/scan/insert work is active",
                "postmaster recovers; no leaked fault state remains",
            ),
            case(
                lane,
                access_method,
                codec,
                "backend-rlimit-oom",
                "cap backend address space with prlimit while AM build work is active",
                "backend reports an OOM-class failure or disconnects; new sessions remain usable",
            ),
        ],
        FaultLane::Cancel => vec![
            case(
                lane,
                access_method,
                codec,
                "pg-cancel-backend",
                "cancel the backend while build/scan/insert/vacuum is in progress",
                "query cancels promptly; no leaked pins or locks",
            ),
            case(
                lane,
                access_method,
                codec,
                "pg-terminate-backend",
                "terminate the backend while build/scan/insert/vacuum is in progress",
                "backend exits cleanly; no leaked pins or locks",
            ),
        ],
        FaultLane::Timeout => vec![
            case(
                lane,
                access_method,
                codec,
                "statement-timeout",
                "SET statement_timeout low enough to interrupt active AM work",
                "timeout ERROR; all retained AM state is dropped",
            ),
            case(
                lane,
                access_method,
                codec,
                "idle-in-transaction-timeout",
                "SET idle_in_transaction_session_timeout after touching an AM fixture in a transaction",
                "idle session is terminated; transaction state is rolled back",
            ),
        ],
        FaultLane::LockTimeout => vec![case(
            lane,
            access_method,
            codec,
            "lock-timeout",
            "SET lock_timeout while contended DDL waits on AM relations",
            "lock timeout ERROR; no relation lock survives the session",
        )],
        FaultLane::Resource => vec![
            case(
                lane,
                access_method,
                codec,
                "tiny-work-mem",
                "run build/scan with tiny work_mem and maintenance_work_mem",
                "clean ERROR or successful bounded execution; no negative counters",
            ),
            case(
                lane,
                access_method,
                codec,
                "temp-file-limit",
                "force temp spill under a tiny temp_file_limit",
                "clean ERROR; backend remains usable and temp state is released",
            ),
            case(
                lane,
                access_method,
                codec,
                "wal-rotation-accounting",
                "perform AM-backed writes and force a WAL segment switch",
                "WAL LSN advances and pg_stat_wal counters remain readable and non-decreasing",
            ),
        ],
        FaultLane::SlowDisk => vec![case(
            lane,
            access_method,
            codec,
            "latency-injection",
            "delay relation and remote-object reads",
            "operation remains cancellable and timeout-governed",
        )],
    }
}

fn case(
    lane: FaultLane,
    access_method: FaultAm,
    codec: Option<DistannCodec>,
    fault: &'static str,
    trigger: &'static str,
    expected: &'static str,
) -> FaultCase {
    FaultCase {
        id: match codec {
            Some(codec) => format!(
                "{}-{}-{}-{fault}",
                access_method.as_str(),
                codec.as_str(),
                lane.as_str()
            ),
            None => format!("{}-{}-{fault}", access_method.as_str(), lane.as_str()),
        },
        lane,
        access_method,
        codec,
        fault,
        trigger,
        expected,
        postconditions: &[
            "no surviving ecaz-fault sessions",
            "no surviving relation or advisory locks",
            "no prepared transactions in the test database",
            "no pinned pg_buffercache entries for ecaz fault fixtures when pg_buffercache is available",
            "pg_stat_io counters remain readable and non-decreasing when pg_stat_io is available",
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

pub fn optional_leak_probe_sql() -> &'static [&'static str] {
    &[
        "pg_buffercache fixture pin count",
        "pg_stat_io non-decreasing total operation count",
        "pg_stat_wal non-decreasing record and byte counters",
        "pg_stat_database temp_bytes before/after resource temp-spill accounting",
    ]
}

pub fn workload_table(fixture: FaultFixture) -> String {
    format!("ecaz_fault_{}", fixture.slug())
}

pub fn workload_index(fixture: FaultFixture) -> String {
    format!("{}_idx", workload_table(fixture))
}

pub fn workload_setup_sql(fixture: FaultFixture, rows: i64) -> String {
    format!(
        "{};
         {};",
        workload_table_sql(fixture, rows),
        workload_create_index_sql(fixture, rows)
    )
}

pub fn workload_table_sql(fixture: FaultFixture, rows: i64) -> String {
    let table = workload_table(fixture);
    let dimensions = fixture.dimensions();
    let source_vector = if fixture.access_method() == FaultAm::DistAnn {
        format!(
            "ARRAY(
                 SELECT (
                     sin((gs * 0.013 * (d + 1))::double precision) +
                     cos((gs * 0.0031 * (d + 1))::double precision)
                 )::real
                 FROM generate_series(0, {dimensions} - 1) AS d
             )"
        )
    } else {
        "ARRAY[
             cos((gs * 0.013)::double precision)::real,
             sin((gs * 0.013)::double precision)::real,
             0.0::real,
             0.0::real
         ]::real[]"
            .to_owned()
    };
    format!(
        "DROP TABLE IF EXISTS {table} CASCADE;
         CREATE TABLE {table} (
             id bigserial PRIMARY KEY,
             embedding ecvector NOT NULL
         );
         INSERT INTO {table} (embedding)
         SELECT encode_to_ecvector(
             {source_vector},
             4,
             42
         )
         FROM generate_series(1, {rows}) AS gs"
    )
}

pub fn workload_create_index_sql(fixture: FaultFixture, rows: i64) -> String {
    workload_create_named_index_sql(fixture, &workload_index(fixture), rows)
}

pub fn workload_create_named_index_sql(fixture: FaultFixture, index: &str, rows: i64) -> String {
    let table = workload_table(fixture);
    match fixture.access_method() {
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
        FaultAm::DistAnn => format!(
            "CREATE INDEX {index} ON {table} USING ec_distann (embedding ecvector_distann_ip_ops) \
             WITH (graph_degree = 8, build_list_size = 20, head_index_cap = 64, neighbor_code_format = '{}')",
            fixture
                .codec()
                .expect("ec_distann fault fixture requires a codec")
                .as_str()
        ),
    }
}

pub fn workload_resource_setup_sql(
    fixture: FaultFixture,
    rows: i64,
    pressure_limit: i64,
) -> String {
    format!(
        "{};
         {};",
        workload_table_sql(fixture, rows),
        workload_create_resource_index_sql(fixture, pressure_limit, rows)
    )
}

pub fn workload_create_resource_index_sql(
    fixture: FaultFixture,
    pressure_limit: i64,
    rows: i64,
) -> String {
    let table = workload_table(fixture);
    let index = workload_index(fixture);
    let pressure_limit = pressure_limit.clamp(1, 1_000);
    let nlists = rows.clamp(4, 16);
    match fixture.access_method() {
        FaultAm::Hnsw => format!(
            "CREATE INDEX {index} ON {table} USING ec_hnsw (embedding ecvector_ip_ops) \
             WITH (m = 8, ef_construction = 32, ef_search = {pressure_limit})"
        ),
        FaultAm::Ivf => format!(
            "CREATE INDEX {index} ON {table} USING ec_ivf (embedding ecvector_ip_ops) \
             WITH (nlists = {nlists}, nprobe = {nlists}, training_sample_rows = {rows}, storage_format = 'turboquant', rerank = 'heap_f32', rerank_width = {pressure_limit})"
        ),
        FaultAm::DiskAnn => format!(
            "CREATE INDEX {index} ON {table} USING ec_diskann (embedding ecvector_diskann_ip_ops) \
             WITH (graph_degree = 8, build_list_size = 32, list_size = {pressure_limit}, rerank_budget = {pressure_limit}, top_k = {pressure_limit})"
        ),
        FaultAm::Spire => format!(
            "CREATE INDEX {index} ON {table} USING ec_spire (embedding ecvector_spire_ip_ops) \
             WITH (nlists = {nlists}, nprobe = {nlists}, storage_format = 'rabitq', rerank_width = {pressure_limit}, max_candidate_rows = {pressure_limit})"
        ),
        FaultAm::DistAnn => format!(
            "CREATE INDEX {index} ON {table} USING ec_distann (embedding ecvector_distann_ip_ops) \
             WITH (graph_degree = 8, build_list_size = 32, head_index_cap = {pressure_limit}, neighbor_code_format = '{}')",
            fixture
                .codec()
                .expect("ec_distann fault fixture requires a codec")
                .as_str()
        ),
    }
}

pub fn workload_accumulator_pressure_settings_sql(
    fixture: FaultFixture,
    pressure_limit: i64,
) -> String {
    let pressure_limit = pressure_limit.clamp(1, 1_000);
    match fixture.access_method() {
        FaultAm::Hnsw => format!("SET ec_hnsw.ef_search = {pressure_limit};"),
        FaultAm::Ivf => format!(
            "SET ec_ivf.nprobe = 16;
             SET ec_ivf.rerank_width = {pressure_limit};"
        ),
        FaultAm::DiskAnn => format!("SET ec_diskann.list_size = {pressure_limit};"),
        FaultAm::Spire => format!(
            "SET ec_spire.nprobe = 16;
             SET ec_spire.rerank_width = {pressure_limit};
             SET ec_spire.max_candidate_rows = {pressure_limit};"
        ),
        FaultAm::DistAnn => format!(
            "SET ec_distann.beam_width = 64;
             SET ec_distann.hop_rounds = 100;
             SET ec_distann.top_k = {pressure_limit};"
        ),
    }
}

pub fn workload_accumulator_pressure_sql(fixture: FaultFixture, pressure_limit: i64) -> String {
    let table = workload_table(fixture);
    let pressure_limit = pressure_limit.clamp(1, 1_000);
    let query = workload_query_vector_sql(fixture, "1");
    format!(
        "SELECT count(*)::bigint
         FROM (
             SELECT id FROM {table}
             ORDER BY embedding <#> {query}
             LIMIT {pressure_limit}
         ) AS nearest"
    )
}

pub fn workload_scan_sql(fixture: FaultFixture) -> String {
    let table = workload_table(fixture);
    let query = workload_query_vector_sql(fixture, "1");
    format!(
        "SET enable_seqscan = off;
         SET enable_bitmapscan = off;
         SET enable_sort = off;
         SELECT id FROM {table}
         ORDER BY embedding <#> {query}
         LIMIT 5"
    )
}

pub fn workload_repeated_scan_sql(fixture: FaultFixture, iterations: i64) -> String {
    let table = workload_table(fixture);
    let query = workload_query_vector_sql(fixture, "probe.i");
    format!(
        "SET enable_seqscan = off;
         SELECT count(*)
         FROM generate_series(1, {iterations}) AS probe(i)
         CROSS JOIN LATERAL (
             SELECT id FROM {table}
             ORDER BY embedding <#> {query}
             LIMIT 5
         ) AS nearest"
    )
}

pub fn workload_insert_sql(fixture: FaultFixture) -> String {
    let table = workload_table(fixture);
    let query = workload_query_vector_sql(fixture, "991");
    format!(
        "INSERT INTO {table} (embedding)
         VALUES (encode_to_ecvector({query}, 4, 42))"
    )
}

pub fn workload_bulk_insert_sql(fixture: FaultFixture, rows: i64) -> String {
    let table = workload_table(fixture);
    let rows = rows.max(1);
    let dimensions = fixture.dimensions();
    let source_vector = if fixture.access_method() == FaultAm::DistAnn {
        format!(
            "ARRAY(
                 SELECT (
                     sin((gs * 0.017 * (d + 1))::double precision) +
                     cos((gs * 0.0041 * (d + 1))::double precision)
                 )::real
                 FROM generate_series(0, {dimensions} - 1) AS d
             )"
        )
    } else {
        "ARRAY[
             cos((gs * 0.017)::double precision)::real,
             sin((gs * 0.017)::double precision)::real,
             0.0::real,
             0.0::real
         ]::real[]"
            .to_owned()
    };
    format!(
        "INSERT INTO {table} (embedding)
         SELECT encode_to_ecvector(
             {source_vector},
             4,
             42
         )
         FROM generate_series(1, {rows}) AS gs"
    )
}

fn workload_query_vector_sql(fixture: FaultFixture, seed_expr: &str) -> String {
    let dimensions = fixture.dimensions();
    if fixture.access_method() == FaultAm::DistAnn {
        format!(
            "ARRAY(
                 SELECT (
                     sin(({seed_expr} * 0.000001 * (d + 1))::double precision) +
                     cos(({seed_expr} * 0.0000031 * (d + 1))::double precision)
                 )::real
                 FROM generate_series(0, {dimensions} - 1) AS d
             )"
        )
    } else {
        format!(
            "ARRAY[
                 cos(({seed_expr} * 0.000001)::double precision)::real,
                 sin(({seed_expr} * 0.000001)::double precision)::real,
                 0.0::real,
                 0.0::real
             ]::real[]"
        )
    }
}

pub fn workload_vacuum_sql(fixture: FaultFixture) -> String {
    format!("VACUUM (ANALYZE) {}", workload_table(fixture))
}

pub fn workload_vacuum_full_sql(fixture: FaultFixture) -> String {
    format!("VACUUM (FULL) {}", workload_table(fixture))
}

pub fn workload_reindex_sql(fixture: FaultFixture) -> String {
    format!("REINDEX INDEX CONCURRENTLY {}", workload_index(fixture))
}

pub fn workload_temp_spill_sql(rows: i64) -> String {
    let rows = rows.max(100_000);
    format!(
        "SELECT count(*)
         FROM (
             SELECT repeat(md5(gs::text), 8) AS payload
             FROM generate_series(1, {rows}) AS gs
             ORDER BY payload
         ) AS spilled"
    )
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use std::process::Command;

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
    fn all_lanes_cover_every_distann_codec_with_distinct_fixture_ids() {
        for lane in FaultLane::ALL {
            let cases = required_smoke_cases(lane);
            for codec in DistannCodec::ALL {
                let codec_cases = cases
                    .iter()
                    .filter(|case| {
                        case.access_method == FaultAm::DistAnn && case.codec == Some(codec)
                    })
                    .collect::<Vec<_>>();
                assert!(
                    !codec_cases.is_empty(),
                    "lane {lane} missing ec_distann codec {codec}"
                );
                assert!(codec_cases
                    .iter()
                    .all(|case| case.id.contains(codec.as_str())));
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
    fn parser_accepts_documented_distann_codec_names() {
        for codec in DistannCodec::ALL {
            assert_eq!(codec.as_str().parse::<DistannCodec>(), Ok(codec));
        }
        assert_eq!(
            "grouped-pq".parse::<DistannCodec>(),
            Ok(DistannCodec::GroupedPq)
        );
    }

    #[test]
    fn workload_sql_mentions_every_access_method() {
        for fixture in FaultFixture::ALL {
            let table = workload_table(fixture);
            let index = workload_index(fixture);
            let sql = workload_setup_sql(fixture, 16);
            assert!(sql.contains(&table));
            assert!(sql.contains(&index));
            assert!(sql.contains(fixture.access_method().as_str()));
            assert!(workload_scan_sql(fixture).contains(&table));
            assert!(workload_repeated_scan_sql(fixture, 10).contains(&table));
            assert!(workload_resource_setup_sql(fixture, 1024, 512).contains(&table));
            assert!(workload_resource_setup_sql(fixture, 1024, 512).contains(&index));
            assert!(workload_accumulator_pressure_sql(fixture, 512).contains(&table));
            assert!(workload_insert_sql(fixture).contains(&table));
            assert!(workload_bulk_insert_sql(fixture, 10).contains(&table));
            assert!(workload_vacuum_sql(fixture).contains(&table));
            assert!(workload_reindex_sql(fixture).contains(&index));
            if let Some(codec) = fixture.codec() {
                assert!(sql.contains(codec.as_str()));
                assert!(sql.contains(&fixture.dimensions().to_string()));
            }
        }
        assert!(workload_temp_spill_sql(10).contains("generate_series(1, 100000)"));
    }

    #[test]
    fn distann_turboquant_fixture_uses_the_supported_no_qjl_dimension() {
        let fixture = FaultFixture::DistAnn(DistannCodec::TurboQuant);
        assert_eq!(fixture.dimensions(), 1536);
        assert!(workload_setup_sql(fixture, 16).contains("generate_series(0, 1536 - 1)"));
        assert!(workload_scan_sql(fixture).contains("generate_series(0, 1536 - 1)"));
    }

    #[test]
    fn provider_environment_pins_provider_and_mode() {
        let env = provider_environment(
            ProviderMode::EioRead,
            "base/",
            3,
            None,
            Some("/tmp/ecaz-fault-provider.marker"),
            None,
            None,
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

    #[test]
    fn socket_provider_environment_pins_exact_peer() {
        let env = provider_environment(
            ProviderMode::SocketReset,
            "",
            2,
            None,
            Some("/tmp/ecaz-fault-provider.marker"),
            Some("/tmp/ecaz-fault-provider.arm"),
            Some("tcp:127.0.0.1:39711"),
        );
        assert!(env
            .iter()
            .any(|(key, value)| { key == "ECAZ_FAULT_PROVIDER_MODE" && value == "socket-reset" }));
        assert!(env.iter().any(|(key, value)| {
            key == "ECAZ_FAULT_PROVIDER_PEER" && value == "tcp:127.0.0.1:39711"
        }));
        assert!(env.iter().any(|(key, value)| {
            key == "ECAZ_FAULT_PROVIDER_ARM_FILE" && value == "/tmp/ecaz-fault-provider.arm"
        }));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ldpreload_provider_returns_eio_for_matched_read() {
        let provider = provider_library_path().expect("linux provider should be built");
        let marker = format!(
            "/tmp/ecaz_fault_provider_read_marker_{}",
            std::process::id()
        );
        let output = Command::new("/bin/cat")
            .arg("/etc/hosts")
            .env("LD_PRELOAD", provider)
            .env("ECAZ_FAULT_PROVIDER_ENABLE", "1")
            .env("ECAZ_FAULT_PROVIDER_MODE", "eio-read")
            .env("ECAZ_FAULT_PROVIDER_MATCH", "/etc/hosts")
            .env("ECAZ_FAULT_PROVIDER_AFTER", "1")
            .env("ECAZ_FAULT_PROVIDER_MARKER", &marker)
            .output()
            .expect("run provider-backed cat");
        assert!(
            !output.status.success(),
            "matched read should fail with EIO"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Input/output error"),
            "unexpected stderr: {stderr}"
        );
        let marker_content = std::fs::read_to_string(&marker).expect("read provider marker");
        let _ = std::fs::remove_file(&marker);
        assert!(
            marker_content.contains("fault=1")
                && marker_content.contains("mode=eio-read")
                && marker_content.contains("target=/etc/hosts"),
            "unexpected marker: {marker_content}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ldpreload_provider_arm_file_gates_injection() {
        let provider = provider_library_path().expect("linux provider should be built");
        let arm_file = format!("/tmp/ecaz_fault_provider_arm_{}", std::process::id());
        let marker = format!("/tmp/ecaz_fault_provider_arm_marker_{}", std::process::id());
        let _ = std::fs::remove_file(&arm_file);
        let disarmed = Command::new("/bin/cat")
            .arg("/etc/hosts")
            .env("LD_PRELOAD", provider)
            .env("ECAZ_FAULT_PROVIDER_ENABLE", "1")
            .env("ECAZ_FAULT_PROVIDER_MODE", "eio-read")
            .env("ECAZ_FAULT_PROVIDER_MATCH", "/etc/hosts")
            .env("ECAZ_FAULT_PROVIDER_AFTER", "1")
            .env("ECAZ_FAULT_PROVIDER_MARKER", &marker)
            .env("ECAZ_FAULT_PROVIDER_ARM_FILE", &arm_file)
            .output()
            .expect("run disarmed provider-backed cat");
        assert!(
            disarmed.status.success(),
            "missing arm file must leave matched reads untouched"
        );

        std::fs::write(&arm_file, "").expect("create provider arm file");
        let armed = Command::new("/bin/cat")
            .arg("/etc/hosts")
            .env("LD_PRELOAD", provider)
            .env("ECAZ_FAULT_PROVIDER_ENABLE", "1")
            .env("ECAZ_FAULT_PROVIDER_MODE", "eio-read")
            .env("ECAZ_FAULT_PROVIDER_MATCH", "/etc/hosts")
            .env("ECAZ_FAULT_PROVIDER_AFTER", "1")
            .env("ECAZ_FAULT_PROVIDER_MARKER", &marker)
            .env("ECAZ_FAULT_PROVIDER_ARM_FILE", &arm_file)
            .output()
            .expect("run armed provider-backed cat");
        let marker_content = std::fs::read_to_string(&marker).expect("read provider marker");
        let _ = std::fs::remove_file(&arm_file);
        let _ = std::fs::remove_file(&marker);
        assert!(!armed.status.success(), "arm file must enable injection");
        assert!(
            marker_content.contains("fault=1") && marker_content.contains("mode=eio-read"),
            "unexpected marker: {marker_content}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ldpreload_slow_disk_marks_only_the_matched_path() {
        let provider = provider_library_path().expect("linux provider should be built");
        let marker = format!(
            "/tmp/ecaz_fault_provider_slow_marker_{}",
            std::process::id()
        );
        let run = |path_match: &str| {
            let _ = std::fs::remove_file(&marker);
            let started = std::time::Instant::now();
            let output = Command::new("/bin/cat")
                .arg("/etc/hosts")
                .env("LD_PRELOAD", provider)
                .env("ECAZ_FAULT_PROVIDER_ENABLE", "1")
                .env("ECAZ_FAULT_PROVIDER_MODE", "slow-disk")
                .env("ECAZ_FAULT_PROVIDER_MATCH", path_match)
                .env("ECAZ_FAULT_PROVIDER_AFTER", "1")
                .env("ECAZ_FAULT_PROVIDER_LATENCY_MS", "500")
                .env("ECAZ_FAULT_PROVIDER_MARKER", &marker)
                .output()
                .expect("run provider-backed cat");
            let elapsed = started.elapsed();
            assert!(output.status.success(), "slow disk must not fail the read");
            (
                std::fs::read_to_string(&marker).expect("read provider marker"),
                elapsed,
            )
        };

        let (unmatched, unmatched_elapsed) = run("/definitely/not/the/hosts/path");
        assert!(
            !unmatched.contains("fault=1"),
            "unmatched path was delayed: {unmatched}"
        );
        assert!(
            unmatched_elapsed < std::time::Duration::from_millis(500),
            "unmatched path took {unmatched_elapsed:?}, expected less than injected latency"
        );
        let (matched, matched_elapsed) = run("/etc/hosts");
        let _ = std::fs::remove_file(&marker);
        assert!(
            matched.contains("fault=1")
                && matched.contains("mode=slow-disk")
                && matched.contains("target=/etc/hosts"),
            "unexpected marker: {matched}"
        );
        assert!(
            matched_elapsed >= std::time::Duration::from_millis(500),
            "matched path took {matched_elapsed:?}, expected at least the injected latency"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ldpreload_provider_returns_enospc_for_matched_create() {
        let provider = provider_library_path().expect("linux provider should be built");
        let path = format!("/tmp/ecaz_fault_provider_write_test_{}", std::process::id());
        let marker = format!(
            "/tmp/ecaz_fault_provider_write_marker_{}",
            std::process::id()
        );
        let output = Command::new("dd")
            .arg("if=/dev/zero")
            .arg(format!("of={path}"))
            .arg("bs=1")
            .arg("count=1")
            .env("LD_PRELOAD", provider)
            .env("ECAZ_FAULT_PROVIDER_ENABLE", "1")
            .env("ECAZ_FAULT_PROVIDER_MODE", "enospc-write")
            .env("ECAZ_FAULT_PROVIDER_MATCH", &path)
            .env("ECAZ_FAULT_PROVIDER_AFTER", "1")
            .env("ECAZ_FAULT_PROVIDER_MARKER", &marker)
            .output()
            .expect("run provider-backed dd");
        assert!(
            !output.status.success(),
            "matched create should fail with ENOSPC"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No space left on device"),
            "unexpected stderr: {stderr}"
        );
        let marker_content = std::fs::read_to_string(&marker).expect("read provider marker");
        let _ = std::fs::remove_file(&marker);
        assert!(
            marker_content.contains("fault=1")
                && marker_content.contains("mode=enospc-write")
                && marker_content.contains(&format!("target={path}")),
            "unexpected marker: {marker_content}"
        );
    }
}
