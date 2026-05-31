# Artifact Manifest

- Head SHA: `a93e464024818b64bd18b0b438d2652dc0964d8f`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/001-scan-profile-notice/`
- Timestamp: `2026-05-31T18:07:37Z`
- Lane / fixture / storage format / rerank mode: code-checkpoint instrumentation only; no benchmark fixture run in this packet.
- Isolated one-index-per-table or shared-table surface: not applicable; no measurement run.

## Validation Artifacts

No raw log files were captured for this code checkpoint. Terminal validation results:

| Command | Result |
| --- | --- |
| `cargo check --all-targets --no-default-features --features pg18` | Pass. Initial run before final formatting took 9m53s; incremental rerun after final edit took 35.74s. Both emitted only existing PostgreSQL header warnings from `pg18_pgstat_shim.c`. |
| `cargo fmt --check` | Pass. Rustfmt emitted existing warnings that `imports_granularity` and `group_imports` are nightly-only. |
| `cargo test --no-default-features --features pg18 scan_profile_notice_guc_defaults_to_off` | Pass. 1 matching unit test passed; remaining tests filtered out. |
| `cargo test --no-default-features --features pg18 sc_011_scan_with_scratch_reuse_matches_fresh` | Pass. 1 matching unit test passed; remaining tests filtered out. |

## Key Lines Cited By Request

- The scan profile NOTICE fields are implemented in `src/am/ec_diskann/routine.rs`.
- The GUC default-off behavior is implemented and unit-tested in `src/am/ec_diskann/options.rs`.
