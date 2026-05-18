# Task 29c Vamana Prune Active-Mask Profile

## Request

Review the Task 29c single-process Vamana prune optimization and release-mode
real-10k measurement.

Measured head: `e765b83e0e04efe1734a10fc0bfa6ecb6564e81f`

This packet keeps raw local PG18 logs under `artifacts/`.

## Summary

`robust_prune` now keeps the sorted candidate array stable and tracks live
candidates with an active mask instead of repeatedly doing `remove(0)` and
`retain` compaction. The selected candidate order and alpha-dominance predicate
are unchanged.

On the same local PG18, release-installed extension, isolated real-10k 1536-d
fixture, and `ec_diskann` reloptions `graph_degree=32`,
`build_list_size=100`, `alpha=1.2`:

| checkpoint | total_ms | build_persist_ms | core_graph_ms | pass0_ms | pass1_ms |
|---|---:|---:|---:|---:|---:|
| before, packet `11102` | 79,238 | 77,485 | 75,903 | 21,539 | 54,363 |
| active-mask prune | 70,678 | 69,000 | 67,571 | 20,737 | 46,832 |

Delta:

- total index-only build improved by `8.560s` (`10.8%`)
- Vamana core graph time improved by `8.332s` (`11.0%`)
- pass 1 improved by `7.531s` (`13.9%`)
- index size stayed `4,939,776` bytes (`4824 kB`)
- isolated L=200 recall@10 stayed `0.9970` with NDCG `0.9999`

The first recall attempt in this packet measured recall@10 `0.9595` with mean
query time `2.73 ms`, but that was not an `ec_diskann` quality regression. The
same table still had HNSW reference indexes from packet `11102`, so the planner
could pick the wrong access method. After dropping those HNSW reference indexes,
the isolated `ec_diskann` recall check returned to `0.9970`.

## Recommendation

Keep the active-mask prune change. It is a narrow semantics-preserving
optimization with a measured release-mode build win and no recall regression on
the isolated real-10k check.

Task 29 remains ready for landing review, now with a better local build
baseline: release-mode real-10k DiskANN index-only build is `70.678s` rather
than the previous `79.238s`.

## Validation

Passed before the code commit:

- `cargo test --lib am::ec_diskann::vamana -- --nocapture`
- `cargo test --lib am::ec_diskann::build -- --nocapture`
- `cargo check --all-targets --no-default-features --features pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

## Artifacts

See `artifacts/manifest.md`.
