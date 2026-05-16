# Artifact Manifest: SPIRE Phase 13d Read Efficiency and Observability

Head SHA: `d2e1334cf927d7e40a0365a71f453ca336039bdf`
Packet/topic: `766-c1-spire-phase13d-read-efficiency-observability`
Lane: Phase 13d production-read measurement and low-risk read-path efficiency
Fixture: static PG18 build/check; local two-cluster PG18 CustomScan read smoke
Storage format: `rabitq`
Rerank mode: `rerank_width = 0`
Surface: `EcSpireDistributedScan` production candidate/heap read path and
`ec_spire_remote_search_production_read_profile`
Timestamp: 2026-05-15T17:52:12Z
Isolated one-index-per-table or shared-table surfaces: isolated
one-index-per-table coordinator and remote tables

## Commands

1. `cargo check --no-default-features --features pg18`
   - Result: passed.
   - Key lines: `Finished dev profile`; warning only for the pre-existing
     unused-import cluster in `src/am/mod.rs`.

2. `cargo fmt`
   - Result: passed.
   - Key lines: stable rustfmt reports the repository's existing warnings for
     nightly-only `imports_granularity` and `group_imports` settings.

3. `git diff --check`
   - Result: passed.

4. `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/766-c1-spire-phase13d-read-efficiency-observability/artifacts`
   - Result: passed.
   - Artifacts:
     - `multicluster-customscan-read.log`
     - `remote-postgres.log`
     - `coord-postgres.log`
   - Key lines:
     - `Custom Scan (EcSpireDistributedScan)`
     - `remote_fanout: 1`
     - `tuple_transport_status: ready`
     - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
     - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
     - `profile_summary=ready|remote_ready|1|1|1|1|1|1|1`
     - `SPIRE multicluster CustomScan read passed`

5. `cargo test production_read_profile_row_preserves_metric_rollup --lib --no-default-features --features pg18`
   - Result: build completed, test binary execution failed before assertions.
   - Artifact: `cargo-test-production-read-profile.log`
   - Key lines: `undefined symbol: pg_re_throw`; `process didn't exit
     successfully`.

6. `cargo test remote_heap_candidate_result_merge_reports_duplicates_before_top_k --lib --no-default-features --features pg18`
   - Result: build completed, test binary execution failed before assertions.
   - Artifact: `cargo-test-heap-merge-stats.log`
   - Key lines: `undefined symbol: pg_re_throw`; `process didn't exit
     successfully`.
