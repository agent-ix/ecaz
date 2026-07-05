# Review Request: Task 51 IVF EXPLAIN Counter Cleanup

## Summary

This checkpoint adds the local observability counters Task 51 needs before
running the next AWS RaBitQ/IVF gate.

Code commit under review:

- `fa005394ad5f58341ff8e0f37dec578dfcc9c9b7` — `Add IVF explain counters for AWS smoke`

Changed files:

- `src/am/common/explain.rs`
  - extends `IvfExplainCounters` with:
    - `Candidates Emitted`
    - `Heap Blocks Fetched`
    - `Approximate Scan Elapsed Us`
    - `Exact Rerank Elapsed Us`
  - updates the IVF EXPLAIN property order and unit expectations.

- `src/am/ec_ivf/scan.rs`
  - records emitted candidates in `amgettuple`, so `LIMIT N` reflects rows
    actually returned to the executor.
  - records approximate scan elapsed time around posting-list candidate
    materialization.
  - records exact rerank elapsed time only when heap-f32 rerank actually runs.
  - records distinct heap blocks fetched for heap-f32 rerank after the rerank
    frontier is sorted by heap TID.

## Local Validation

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the source of
truth for command lines and key result lines.

- `cargo check --lib --no-default-features --features pg18` passed.
- Scoped `rustfmt --check` for the two changed files passed.
- Scoped `git diff --check` for the two changed files passed.
- Local isolated PG18 preload smoke passed:
  - `shared_preload_libraries=ecaz`
  - `ec_ivf` index path used with `storage_format='rabitq'`
  - `rerank='heap_f32'`, `rerank_width=3`
  - EXPLAIN emitted the new counters:
    - `Candidates Emitted: 3`
    - `Heap Blocks Fetched: 1`
    - `Approximate Scan Elapsed Us: 68`
    - `Exact Rerank Elapsed Us: 42`

## Notes

AWS remains the final gate only. This packet is local smoke coverage for the
counter cleanup before scaling any benchmark or optimization run to Amazon.
