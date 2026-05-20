# Review Request: HNSW Parallel Build DSM Accessors

## Summary

This packet covers the Task 50 top-15 file `src/am/ec_hnsw/build_parallel.rs`.

Code commits:

- `3c9a1223` Reduce HNSW parallel build DSM unsafe access
- `8f552553` Fix HNSW DSM facade clippy mutation access

The change adds a small `EcHnswConcurrentDsmGraphParts` facade for DSM graph header/node/lock/insert-state access, then lifts DSM graph image attach/init/readback/insert helpers from `unsafe fn` to safe functions where the helper can keep the raw pointer projection local. Test setup now goes through `initialize_test_concurrent_dsm_graph`.

An intermediate facade method returned mutable references from `&self`; `cargo clippy` caught this as `clippy::mut_from_ref`, and `8f552553` fixes it by requiring mutable access for init-only projections and using raw pointer projections for lock/atomic-cell addresses.

## Unsafe Count

| File | Start | Now | Target | Status |
|---|---:|---:|---:|---|
| `src/am/ec_hnsw/build_parallel.rs` | 203 | 139 | <=142 | met (-31.5%) |

Artifacts:

- `artifacts/block-count-planning-baseline.log`
- `artifacts/block-count-after.log`

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/build_parallel.rs`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with existing warnings.
- `git diff --check`: passed.
- `cargo test build_parallel --lib --no-default-features --features pg18`: compiled, then failed at runtime with `undefined symbol: CacheRegisterRelcacheCallback`.
- `cargo fmt --all --check`: failed on existing repo-wide formatting backlog outside this touched file.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: failed on existing repo-wide clippy backlog; regenerated log has no `build_parallel.rs` clippy finding after `8f552553`.

## Bench / Perf

No benchmark lane is claimed for this packet. The touched code is the HNSW parallel build DSM graph path, so HNSW build-latency/regression evidence should be included in the Task 50 tranche closeout or the next HNSW benchmark packet before Task 50 is closed.

## Review Notes

Please review whether the new DSM graph facade keeps the raw pointer contracts in the right layer, especially:

- `EcHnswConcurrentDsmGraphParts::{header,node,node_lock,node_insert_state_cell}`
- `initialize_concurrent_dsm_graph_image`
- `insert_concurrent_dsm_graph_node`
- `concurrent_dsm_graph_to_build_nodes`
- `parallel_graph_build_worker_main`

Also note that new reviewer feedback for packets 012-015 was recorded in `e583f720`; those findings request a follow-up soundness fix before further HNSW/SPIRE facade work.

