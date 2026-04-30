# Artifact Manifest

Head SHA: `11393c34`

Packet: `review/11101-task29c-diskann-build-phase-profile`

Lane: Task 29c DiskANN build performance tuning.

Fixture: local PG18, real-10k 1536-d corpus copied into isolated prefix
`task29c_phase_profile`, 200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with persisted binary
sidecar payload.

Reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`.

Table model: isolated one-index-per-table prefix `task29c_phase_profile`.

Cache state: warm local run on the existing PG18 scratch instance.

Timestamp: 2026-04-30T13:07:14-07:00

## Artifacts

### `enable-build-log-client-messages.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/enable-build-log-client-messages.log --sql "ALTER DATABASE task29_diskann_baseline SET client_min_messages TO log;"`

Key result: `ALTER DATABASE`.

This was an attempted way to capture `pgrx::log!` output. The loader did not
mirror that output, so the timing line was changed to a single `NOTICE`.

### `drop-task29c-phase-profile-prefix.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/drop-task29c-phase-profile-prefix.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; DROP TABLE IF EXISTS task29c_phase_profile_corpus; DROP TABLE IF EXISTS task29c_phase_profile_queries;"`

Key result: old isolated prefix was absent.

### `load-task29c-phase-profile-real10k.log`

Command:

`target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11101-task29c-diskann-build-phase-profile/artifacts/load-task29c-phase-profile-real10k.log corpus load --prefix task29c_phase_profile --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch`

Key result rows:

- copied corpus table in `4.63s`
- encoded corpus table in `4.47s`
- built index in `501.64s`
- completed prefix in `512.16s`

This run used the `LOG` timing line, which was not visible in the loader log.

### `drop-task29c-phase-profile-rerun-prefix.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/drop-task29c-phase-profile-rerun-prefix.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; DROP TABLE IF EXISTS task29c_phase_profile_corpus; DROP TABLE IF EXISTS task29c_phase_profile_queries;"`

Key result: isolated prefix dropped for the rerun.

### `load-task29c-phase-profile-notice-real10k.log`

Command:

`target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11101-task29c-diskann-build-phase-profile/artifacts/load-task29c-phase-profile-notice-real10k.log corpus load --prefix task29c_phase_profile --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch`

Key result rows:

- copied corpus table in `4.32s`
- encoded corpus table in `4.28s`
- built index in `495.48s`
- completed prefix in `505.38s`

The loader path still did not surface notices, so index-only profiling used
`ecaz-cli dev sql` below.

### `create-index-task29c-phase-profile-notice.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/create-index-task29c-phase-profile-notice.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"`

Key timing line:

- `heap_scan_ms=4374`
- `source_ref_ms=15`
- `training_ms=4429`
- `sidecar_setup_ms=32`
- `payload_derivation_ms=10187`
- `build_persist_ms=471757`
- `overflow_ms=0`
- `codebook_ms=2`
- `write_pages_ms=47`
- `metadata_ms=0`
- `flush_total_ms=486473`
- `total_ms=490850`

### `create-index-task29c-heap-build-search-fixed.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/create-index-task29c-heap-build-search-fixed.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"`

Key timing line:

- `heap_scan_ms=10312`
- `source_ref_ms=14`
- `training_ms=5214`
- `sidecar_setup_ms=37`
- `payload_derivation_ms=11652`
- `build_persist_ms=524053`
- `overflow_ms=0`
- `codebook_ms=2`
- `write_pages_ms=43`
- `metadata_ms=0`
- `flush_total_ms=541019`
- `total_ms=551334`

This was the corrected heap-frontier build-search experiment. It regressed and
was reverted in `11393c34`.

### `reset-build-log-client-messages.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11101-task29c-diskann-build-phase-profile/artifacts/reset-build-log-client-messages.log --sql "ALTER DATABASE task29_diskann_baseline RESET client_min_messages;"`

Key result: `ALTER DATABASE`.
