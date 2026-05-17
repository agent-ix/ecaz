# Task 28 IVF Cost Model Posting-Scale Follow-Up

## Scope

This packet covers the follow-up cost-model repair after packet 30053 showed
normal planning selecting a sequential scan for the 10k `nlists=128` IVF
surface, while packet 30054 showed the forced index path was much faster.

Code checkpoint: `077aae15e01113fc41aced09dbb6624bec84ccb1`

## Change

`src/am/ec_ivf/cost.rs` now models the default IVF path as quantized index
scoring over mostly sequential index pages:

- centroid scoring scale lowered from `0.75` to `0.03` dimensions;
- posting scoring scale lowered from `0.03` to `0.01` dimensions;
- centroid and posting page costs use `seq_page_cost * 0.25` instead of
  `random_page_cost`.

This keeps cost increasing with `nprobe`, but stops charging IVF posting scans
as if each candidate were a full-dimensional f32 score plus random heap I/O.

## Result

On the existing local PG18 10k DBPedia-derived n128 surface
`task28_ivf_postopt10k_n128w25`, normal benchmark execution without
`--force-index` now uses the IVF latency band:

| run | nprobe | result |
|---|---:|---|
| recall smoke, 20 queries | 8 | `recall@10=0.7000`, `ndcg@10=0.9723`, mean query time `40.98 ms` |
| latency smoke, 20 iterations, c1 | 8 | mean `34.8 ms`, p50 `33.5 ms`, p95 `39.3 ms`, p99 `62.3 ms` |
| prepared EXPLAIN | 8 | `Index Scan using task28_ivf_postopt10k_n128w25_idx`, cost `43.00..594.25`, execution `64.663 ms` |

This does **not** change the n128 frontier conclusion from packet 30054:
n128 remains useful evidence, not the recommended high-recall operating point
for this fixture. The immediate fix is narrower: normal planning no longer
turns the n128 smoke into a 4s sequential-scan path.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/metadata.sql`
- `artifacts/metadata.log`
- `artifacts/explain_n128_nprobe8_prepared.sql`
- `artifacts/explain_n128_nprobe8_prepared.log`
- `artifacts/recall_10k_n128w25_nprobe8_normal.log`
- `artifacts/latency_10k_n128w25_nprobe8_normal.log`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::cost --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18,pg_test --no-default-features`
- `cargo pgrx test pg18 test_ec_ivf_cost_snapshot_reports_modeled_costs`
- `git diff --check`

## Next Slice

Return to the competitive-substrate backlog after this planner repair:
live-insert fixed per-row work remains the largest correctness/performance
gap, while posting-list scoring/layout work remains the likely scan-latency
lever. DiskANN stays out of Task 28.
