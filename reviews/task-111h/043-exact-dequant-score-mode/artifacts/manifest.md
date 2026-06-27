# Task 111h / Packet 043 Artifacts Manifest

- Head SHA: `1ed1cd9e55e825d9e2b739db168baf5ea749d526`
- Task bucket: `reviews/task-111h`
- Packet path: `reviews/task-111h/043-exact-dequant-score-mode`
- Lane / fixture / storage format / rerank mode: PG18 unit/integration
  validation for compact rerank exact-dequant score mode; no benchmark lane.
- Timestamp: 2026-06-20
- Isolated one-index-per-table or shared-table surface: not applicable; no
  benchmark or SQL corpus run.

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `cargo-test-exact-dequant.log` | `CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 exact_dequant --lib` | `4 passed; 0 failed` |
| `cargo-test-on-disk-ivf-metadata.log` | `CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test on_disk_fixtures ivf_metadata` | `8 passed; 0 failed` |
| `cargo-test-upgrade-matrix.log` | `CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test upgrade_matrix` | `2 passed; 0 failed` |
| `cargo-test-size-of-assertions.log` | `CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test size_of_assertions` | `13 passed; 0 failed` |
| `cargo-check-pg18.log` | `CARGO_INCREMENTAL=0 cargo check --no-default-features --features pg18` | finished successfully |

## Validation Notes

- Earlier parallel validation attempts hit `ENOSPC` in Cargo build artifacts,
  not source/test failures. The temporary worktree `target/` was removed and
  validation was rerun serially with `CARGO_INCREMENTAL=0`; the passing logs
  above are the source of record for this packet.
- No benchmark results are cited by this packet.
