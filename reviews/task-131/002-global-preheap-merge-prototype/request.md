# Task 131 Packet 002: Global Pre-Heap Merge Prototype

## Summary

This checkpoint adds an opt-in Phase 1 prototype for coordinator global merge-before-heap pruning on the production pooled remote read path.

Code commit: `34272982a6b997a0413082e56c70c58229408f6f` (`task 131 add opt-in global preheap merge`)

## Changes

- Adds userset GUC `ec_spire.remote_search_global_pre_heap_merge`, default off.
- Adds `ec_spire_remote_search_explicit_local_heap_candidates(...)`, an internal remote heap endpoint that accepts an explicit compact candidate subset. Binary candidate fields are passed as hex `text[]` values across libpq and decoded on the worker.
- When the GUC is enabled and no tuple payload projection is requested, the pooled production candidate+heap path now:
  - merges compact candidate batches globally before heap resolution;
  - records existing global pre-heap input/candidate/duplicate/pruned metrics from the actual merged subset;
  - sends each worker only its globally surviving candidates;
  - skips the heap query entirely for workers with no surviving candidates.
- Tuple-payload projection reads intentionally keep the existing path in this checkpoint.

## Validation

- `artifacts/cargo-check-pg18.log`: `cargo check --no-default-features --features pg18` passed.
- `artifacts/git-diff-check-head.log`: `git diff --check HEAD~1..HEAD` passed with no output.
- `artifacts/cargo-test-explicit-heap-params.log`: `timeout 150s cargo test explicit_heap_candidate_parameters_encode_binary_fields_as_hex --no-default-features --features pg18` exited `124` while compiling before test execution.

## Not Closeout Evidence

This is an implementation checkpoint only. It does not claim a Task 131 win or close any phase. Task 131 still requires local multi-instance `ecaz bench suite` evidence at 10k, 50k, and 100k, including recall/result identity, latency p50/p95/p99, heap rows avoided, payload bytes avoided, and storage where applicable before any promotion or closeout decision.

