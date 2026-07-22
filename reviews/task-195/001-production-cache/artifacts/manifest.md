# Artifact manifest — Task 195 packet 001

- Head SHA: `65e746fc1a85f8efa8032a81aa2cc95b55c23503`
- Task bucket / packet: `reviews/task-195/001-production-cache/`
- Primary target: PG18
- Lane: production owner-schema cache implementation and lifecycle safety
- Fixture / storage / rerank / wire behavior: no benchmark fixture; storage,
  rerank, result payload, placement, traversal, and materialization window are
  unchanged
- Isolation: correctness and compile evidence only; packet 002 owns the
  one-index-per-table release A/B benchmark matrix
- Timestamp: 2026-07-22 America/Los_Angeles

## Files and commands

| Artifact | SHA-256 | Command / result |
|---|---|---|
| `pg18-multi-epoch-production.log` | `6d6dd8ce91ff2282fa978aa48fd9fbf7e5ad3c1ab74231ade4e9c4e511e17a51` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx test pg18 test_distann_multi_epoch_publish --no-default-features --features pg18`; 1 passed, 0 failed, 2,519 filtered |
| `clippy-pg18.log` | `8750e8264f8129b66c2388f1481a816048323a9db22b0c1eff6e29a7f2e7295f` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`; pass |
| `clippy-pg18-attribution.log` | `8b95832ee6487b7d358bc73c1d2da6dab57d4d7682b3806e1f9ed2b109ceed13` | `cargo clippy --all-targets --no-default-features --features pg18,distann-head-attribution-benchmark -- -D warnings`; pass |
| `cli-removed-selector-test.log` | `826901c90c586e082a5221bfe7344f2522d70f073f3d5c4d8d71a9dfd3da3bc6` | `cargo test -p ecaz-cli owner_validation -- --nocapture`; removed JSON selector rejected, 1 passed |
| `cli-production-path-test.log` | `57b23edd5810fc208869e44adcf5a436ea1e40097f6fb2d136714cf734100652` | `cargo test -p ecaz-cli production_schema_cache -- --nocapture`; no selector setting emitted, 1 passed |
| `cli-neighbor-controls-test.log` | `93c3d7bf4dfbfe46a0c548f2c58a4c2c1d32ede62f5eccc76358b80999f32e34` | `cargo test -p ecaz-cli owner_plan_and_fixed_work -- --nocapture`; shifted independent controls, 2 passed |

The first sandboxed lifecycle invocation compiled successfully but could not
write the pgrx install directory. The durable artifact is the permission-correct
rerun above, whose command exited 0. That test install is a debug binary and is
not measurement evidence.
