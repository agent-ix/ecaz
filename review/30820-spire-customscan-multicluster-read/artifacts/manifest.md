# Artifact Manifest: 30820 SPIRE CustomScan Multicluster Read

## `multicluster-customscan-read.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: PG18 local multicluster
  CustomScan read fixture; `ecvector_spire_ip_ops`; separate coordinator and
  remote clusters; `rabitq`; tuple-payload projection
- command used:
  `scripts/run_spire_multicluster_customscan_read_pg18.sh --artifact-dir review/30820-spire-customscan-multicluster-read/artifacts --run-id cscan30820c`
- timestamp: 2026-05-11T08:29:00-07:00
- isolated/shared surface: isolated coordinator and remote PostgreSQL data
  directories under `target/spire-cscan-read-pg18-cscan30820c`; separate socket
  endpoints; remote descriptor resolves through `conninfo_secret_name`
- key result lines:
  `plan=Limit -> Custom Scan (EcSpireDistributedScan) on ec_spire_customscan_coord_sql`;
  `read_row=10,remote alpha`;
  `payload_probe=ready,2,{"id": 10, "title": "remote alpha"}`;
  `SPIRE multicluster CustomScan read passed`

## `remote-postgres.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: remote PG18 cluster log for
  the CustomScan multicluster read fixture
- command used: produced by the multicluster CustomScan read script above
- timestamp: 2026-05-11T08:29:00-07:00
- isolated/shared surface: remote data directory
  `target/spire-cscan-read-pg18-cscan30820c/remote`
- key result lines: PostgreSQL startup, checkpoint, fast shutdown, and clean
  shutdown entries with no fixture error

## `coord-postgres.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: coordinator PG18 cluster log
  for the CustomScan multicluster read fixture
- command used: produced by the multicluster CustomScan read script above
- timestamp: 2026-05-11T08:29:00-07:00
- isolated/shared surface: coordinator data directory
  `target/spire-cscan-read-pg18-cscan30820c/coord`
- key result lines: PostgreSQL startup, checkpoint, fast shutdown, and clean
  shutdown entries with no fixture error

## `cargo-test-custom-scan-status-lib.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: focused Rust and PG18 status
  coverage for `ec_spire_custom_scan_status()`
- command used:
  `script -q -e -c "cargo test custom_scan_status --lib" review/30820-spire-customscan-multicluster-read/artifacts/cargo-test-custom-scan-status-lib.log`
- timestamp: 2026-05-11T08:30:02-07:00
- isolated/shared surface: cargo test with pg_test-backed SQL status fixture
- key result lines:
  `test am::ec_spire::custom_scan::tests::custom_scan_status_reports_executor_stream_tuple_payload_slots ... ok`;
  `test tests::pg_test_ec_spire_custom_scan_status_registered_fail_closed ... ok`;
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1617 filtered out`

## `bash-n-customscan-read.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: shell syntax check for the
  new multicluster CustomScan read fixture
- command used:
  `script -q -e -c "bash -n scripts/run_spire_multicluster_customscan_read_pg18.sh" review/30820-spire-customscan-multicluster-read/artifacts/bash-n-customscan-read.log`
- timestamp: 2026-05-11T08:30:00-07:00
- isolated/shared surface: script syntax only
- key result lines: command exited successfully

## `cargo-fmt-check.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: Rust formatting check
- command used:
  `script -q -e -c "cargo fmt --check" review/30820-spire-customscan-multicluster-read/artifacts/cargo-fmt-check.log`
- timestamp: 2026-05-11T08:30:00-07:00
- isolated/shared surface: workspace formatting check
- key result lines: command exited successfully; output contains the
  repository's existing stable-rustfmt warnings about nightly-only import
  options

## `git-diff-cached-check.log`

- head SHA: `03d789e36863982dc2e186214e3d0f21056ccfc4`
- packet/topic: `30820 / spire-customscan-multicluster-read`
- lane / fixture / storage format / rerank mode: cached whitespace check for
  the code commit
- command used:
  `script -q -e -c "git diff --cached --check" review/30820-spire-customscan-multicluster-read/artifacts/git-diff-cached-check.log`
- timestamp: 2026-05-11T08:29:00-07:00
- isolated/shared surface: staged code/tracker changes only, with unrelated
  local WIP left unstaged
- key result lines: command exited successfully with no whitespace errors
