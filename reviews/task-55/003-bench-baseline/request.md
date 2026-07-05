# Task 55 Packet 003 — DiskANN M5 Baseline + Bench Gate

Status: **proposed**

Establishes the DiskANN M5 baseline (the new
`benchmarks/task-55-m5-diskann-baseline/`) and runs the full 8-step
`ecaz bench suite` against it. Per `plan/tasks/55-*.md` §Performance
Gate, this is the bench gate that §Exit Criterion #3 requires:

> DiskANN bench packet at `benchmarks/task-55-m5-diskann-baseline/`
> runs cleanly with 8/8 steps succeeded.

## Suite shape

| Step | Profile | Sweep | Trials |
| --- | --- | --- | --- |
| load 10k DiskANN | `ec_diskann` | — | corpus 10k × queries 200 |
| recall 10k DiskANN | `ec_diskann` | list_size {64, 128, 200, 400, 800} | k=10 |
| latency 10k DiskANN | `ec_diskann` | list_size {64, 128, 200, 400, 800} | 1000 |
| storage 10k DiskANN | — | — | per-row B |
| load 100k DiskANN | `ec_diskann` | — | corpus 100k × queries 1000 |
| recall 100k DiskANN | `ec_diskann` | list_size {64, 128, 200, 400, 800} | k=10 |
| latency 100k DiskANN | `ec_diskann` | list_size {64, 128, 200, 400, 800} | 1000 |
| storage 100k DiskANN | — | — | per-row B |

Same shape as Task 50's local DiskANN baseline at
`benchmarks/task-50-local-baseline/`, host-adapted (M5 fixtures
at 10k + 100k corpus sizes).

## Build-path coverage

The `load-{10k,100k}-diskann` steps exercise the migrated DiskANN
build chain:

- `ambuild::write_data_pages` (Task 55 packet 002 — safe `fn(RelationHandle, &DataPageChain)` consuming `LockedBufferGuard::read_main_locked_handle` + `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}`).
- `ambuild::initialize_metadata_page_handle` (safe) + `ambuild::overwrite_metadata_page_handle` (safe).
- `ambuild::write_metadata_to_buffer` consuming the P3 wrappers.

The `recall-*` and `latency-*` steps exercise:

- `scan_state::materialize_chain_from_index_handle` (safe variant) for `beginscan`-time chain materialization.
- `DiskannInsertRelation::read_main` / `read_main_locked` safe-fn surface (consumed by aminsert path; not bench-exercised directly but compile-gated by the suite-run binary).

## Artifacts

Logs under `benchmarks/task-55-m5-diskann-baseline/artifacts/`:

- `corpus-load-ec_real_{10k,100k}-diskann.log`
- `recall-ec_real_{10k,100k}-diskann.log`
- `latency-ec_real_{10k,100k}-diskann.log`
- `storage-ec_real_{10k,100k}-diskann.log`
- `results.jsonl`, `suite-manifest.json`, `suite-run.log`

Packet-local manifest at `reviews/task-55/003-bench-baseline/artifacts/manifest.md`
points at the benchmarks/ packet as the source of truth.

## Acceptance

- 8/8 steps complete with exit 0.
- Recall@10 within historical DiskANN envelope (typically 0.95+ at
  list_size=400 at 10k; 0.90+ at list_size=200 at 100k).
- Storage per-row reported for both corpora.
- Build wall-clock recorded.

This establishes the reference. There is no prior M5 DiskANN
baseline to compare against; later Task 55 / 33 / 56 / 57 work
compares against this.

## Cross-references

- `benchmarks/task-55-m5-diskann-baseline/manifest.md`
- `reviews/task-55/002-consumer-migration/request.md`
- `benchmarks/task-50-m5-hnsw-baseline/` (shape template)
- `benchmarks/task-50-local-baseline/suite.json` §DiskANN steps (parameter template)
