# Task 167 packet 050 artifact manifest

- Head under review:
  `22c1e01c3f7dfb188f3f38c2022b5208252825e4` — restore the
  batch-consistent append-when-room default, relabel the shipped/control arms,
  and structure the new backlink-strategy A/B row.
- Formatter-only companion:
  `e2b16e6d5aee62510fdd411d61b4b45b6faa912f` — repository rustfmt
  normalization explicitly approved by the operator and committed separately.
- Owning packet: `reviews/task-167/050-batch-consistent-backlink-default/`.
- Timestamp: `2026-08-22`.
- Scope: extension default and benchmark attribution only. No benchmark result
  is claimed in this code-review packet.
- Before comparator: packet 047 at runtime head
  `8bf0ac8a451f9cd73813dd0ab59ed305fab026bd`, 50k heldout physical
  `0.848722`, fresh `0.857333`, deficit `0.008611`, allowed `0.007000`, miss
  `0.001611`.

## Static algorithm reconciliation

- Batch reference: `src/am/ec_diskann/vamana.rs` appends while the backlink
  target is under `max_degree`; only a full target re-prunes the union.
- Shared incremental planner: `plan_insert_backlink` in
  `src/am/ec_distann/insert.rs` returns the existing edges plus the new edge
  when capacity remains. Its `backlink_appends_when_free` regression asserts
  that exact edge-preserving result.
- Divergent physical behavior: the previous production GUC default enabled
  `debug_disable_append_when_room`, routing under-capacity targets into the
  robust-prune union in `physical_dml.rs`.
- Correction: the default is false again; robust-prune-all remains an explicit
  diagnostic control. The suite measures exact shipped quality before that
  control is allowed to mutate the disposable fixture.

## Validation

- Command:
  `cargo test -p ecaz --no-default-features --features pg18 distann_default_backlink_strategy_matches_batch_append_when_room`.
- Result: passed, 1/1. Artifact: `cargo-test-default.log`, committed
  LF-normalized SHA-256
  `8be3431aad5df34ebcd1a8384ae80aaa11d8d2be273ebfa071f20b50812509a6`.
- Command:
  `cargo test -p ecaz --no-default-features --features pg18 backlink_appends_when_free`.
- Result: passed, 1/1. Artifact: `cargo-test-planner.log`, committed
  LF-normalized SHA-256
  `5a89e8e630e5876e5ae96c797b1ceb0d717dcf86f2d5d1d31635e22342b75d20`.
- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`.
- Result: passed, 1/1. Artifact: `cargo-test-suite-parser.log`, committed
  LF-normalized SHA-256
  `6e75f6d0c89f928bf385cd018cdc3549ea4790cc5dcf25bc6f13c733407222df`.
- Command:
  `cargo test -p ecaz-cli --no-default-features task167_quality_gate`.
- Result: passed, 2/2. Artifact: `cargo-test-quality-gate.log`, committed
  LF-normalized SHA-256
  `e93cf608538fbeaff34abd99f7596660c074b1d6a56090505a2cb372f618cdf8`.
- `git diff --check` passed before both code checkpoints.
