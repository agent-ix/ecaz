# Artifact Manifest: SPIRE Remote Candidates Split

Packet: `30991-spire-remote-candidates-split`
Head SHA at run time: `7310eff9` plus working-tree checkpoint changes
Timestamp: 2026-05-13 11:44-11:48 America/Los_Angeles

This packet is a layout/refactor checkpoint. Lane, fixture, storage format,
and rerank mode are not applicable except where a command explicitly names a
test filter.

| Artifact | Command | Result |
|---|---|---|
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | exit 0; `Finished dev profile`; one pre-existing unused-import warning in `src/am/mod.rs` |
| `cargo-fmt-check.log` | `cargo fmt --check` | exit 0; rustfmt emitted stable-toolchain warnings for unstable import-group config |
| `git-diff-check.log` | `git diff --check -- src/am/ec_spire/mod.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/remote_candidates plan/tasks/task30-phase12a-spire-readiness-followups.md plan/tasks/task30-phase12b-spire-cleanup.md` | exit 0 |
| `remote-candidates-line-count.log` | `find src/am/ec_spire/root/remote_candidates -type f -name '*.rs' -print0 \| xargs -0 wc -l \| sort -n` | largest split file is `production_transport.rs` at 1,633 lines; total split surface 12,986 lines |
| `cargo-test-production-executor-state.log` | `cargo test --no-default-features --features pg18 production_executor_state` | exit 0; 34 passed, 0 failed, 1678 filtered out; includes `pg_test_ec_spire_production_executor_state_summary_is_dry` |

