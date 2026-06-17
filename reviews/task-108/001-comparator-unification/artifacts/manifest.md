# Artifacts manifest — Task 108 packet 001 (comparator bench unification)

Head SHA: `6f3427d96` (code), artifacts captured at branch tip
`task-108-109-comparator-unification`.
Task bucket / packet: `reviews/task-108/001-comparator-unification/`.
Host: local Intel desktop (WSL2). Storage format / rerank: N/A — these are
build/lint/expansion checks, not a corpus benchmark (no DB run).

This is a code-review packet, not a measurement packet; there is no corpus
load. The AWS Graviton 4 vchord measurement is the separate packet 002 (Task
108.4).

## Artifacts

### `comparator-tests.log`
- Command: `cargo test -p ecaz-cli comparator`
- Result: `test result: ok. 15 passed; 0 failed`.
- Key cited lines:
  - `vchord_index_sql_pins_rabitq_residual_options ... ok`
    (asserts `USING vchordrq (embedding vector_ip_ops)`,
    `residual_quantization = true`, `[build.internal]`, `lists = [N]`, `$vco$`)
  - `knn_sql_uses_ip_operator_and_bind_cast ... ok` (`<#>` +
    `$1::real[]::vector(dim)`)
  - `default_lists_is_ceil_sqrt_rows ... ok` (50000→224, 100000→317,
    1000000→1000, override path)
  - `expands_comparator_with_vchord_engine_and_lists ... ok`
  - `parses_comparator_table_and_summary_rows ... ok`

### `comparator-dry-run.log`
- Command (per config): `cargo run -q -p ecaz-cli -- bench suite run
  --config crates/ecaz-cli/suites/<cfg>.json --dry-run`
- Configs: `profile-hnsw-100k`, `profile-ivf-100k`,
  `profile-cross-engine-real10k`.
- Key cited line (vchord-shaped expansion is identical in form):
  `comparator-pgvector-hnsw-real100k -> ... bench comparator --engine
  pgvector-hnsw --prefix profile_real100k_hnsw_m16 --k 10 --sweep
  "64,128,200,400" --m 16 --ef-construction 128 --queries-limit 100
  --log-output ... --rebuild`
  — every migrated compare step now expands to a `comparator` step; no
  `--profile` appears (no ecaz side).

### `clippy-summary.log`
- Command: `cargo clippy -p ecaz-cli --all-targets`
- The 17/16 warnings are pre-existing crate-wide lints; a filtered run
  (`grep comparator.rs`) shows **no** warnings in the new module. The
  `large_size_difference` on `SuiteStep` pre-dates this change (largest variant
  is `SpirePipelineStep`, not the new `ComparatorStep`).
- `cargo build -p ecaz-cli`: clean.

## Re-run

```sh
cargo test -p ecaz-cli comparator
cargo clippy -p ecaz-cli --all-targets
for cfg in profile-hnsw-100k profile-ivf-100k profile-cross-engine-real10k; do
  cargo run -q -p ecaz-cli -- bench suite run \
    --config crates/ecaz-cli/suites/$cfg.json --dry-run
done
```
