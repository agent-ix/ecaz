# Review Request: C1 Task16 Vacuum Entry Repair and Scan Fallback

Current head at execution: `be4c0a4`

## Context

Packet `437` closed the live lever-4 / lever-5 score-mode matrix, but the
requested concurrent `INSERT + scan + VACUUM` rerun uncovered a correctness
problem on current head:

- `scripts/vacuum_concurrency_scratch.sh --socket-dir /home/peter/.pgrx --duration 60`
- failed with both scan workers hitting:
  - `unexpected tqhnsw scan result count: 0`

That was not a scorer issue. The failure reproduced on the generic scratch
vacuum harness after a neutral quantized restart.

## Root Cause

Two runtime gaps combined:

1. `initialize_scan_entry_candidate(...)` treated a dead or empty
   `metadata.entry_point` as terminal and returned no seed candidate, so scans
   could transiently emit zero rows.
2. `vacuum.rs` finalized fully dead elements but never repaired
   `metadata.entry_point` afterward, so once vacuum deleted the current entry
   point the metadata could keep advertising a dead seed until a later insert
   happened to repair it.

The concurrent harness hit exactly that window.

## What Landed

### Scan fallback

- `src/am/shared.rs`
  - added `LiveEntryCandidate`
  - added `highest_level_live_entry_candidate(...)` to find a surviving live
    top-level element directly from on-disk tuples
- `src/am/scan.rs`
  - `initialize_scan_entry_candidate(...)` now falls back to that surviving
    live candidate when `metadata.entry_point` is invalid, deleted, or empty
    instead of returning no seed candidate

### Vacuum metadata repair

- `src/am/vacuum.rs`
  - after `repair_graph_connections(...)` and
    `finalize_fully_dead_elements(...)`, vacuum now repairs
    `metadata.entry_point` if the finalized set included the current metadata
    entry or the metadata was already invalid
  - when live elements remain, metadata is rewritten to a surviving highest-level
    live element and `metadata.max_level` is kept aligned with that repaired
    entry point
  - when none remain, metadata falls back to `INVALID` / `0`

### Coverage

- `src/lib.rs`
  - added `test_tqhnsw_vacuum_repairs_deleted_entry_point_metadata`
  - added `test_tqhnsw_scan_falls_back_from_stale_entry_metadata`

## Validation

Green checkpoint on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused regression checks:

- `cargo pgrx test pg17 test_tqhnsw_vacuum_repairs_deleted_entry_point_metadata`
- `cargo pgrx test pg17 test_tqhnsw_scan_falls_back_from_stale_entry_metadata`

Scratch concurrency rerun on current head:

```bash
./scripts/restart_adr030_scratch.sh --pgrx-home /home/peter/.pgrx --rerank-mode quantized
scripts/vacuum_concurrency_scratch.sh --socket-dir /home/peter/.pgrx --duration 60
```

Observed result:

- `vacuum concurrency harness passed`
- `insert_worker_iterations=335`
- `vacuum_worker_iterations=196`
- `scan_a_worker_iterations=585`
- `scan_b_worker_iterations=583`
- `final_live_rows=2948`
- `final_live_elements=2948`
- `final_reachable_live_elements=2012`
- `reference_reachable_live_elements=1732`
- `reachable_vs_reference_percent=116`
- `final_scan_result_count=40`

## Readout

### 1. The transient zero-result scan bug is fixed

The failing `437` harness no longer drops to `0` rows during concurrent vacuum.
That was the correctness regression this slice targeted.

### 2. The fix is generic AM hygiene, not a TurboQuant-only scorer change

The issue was stale metadata entry-point handling in scan/vacuum lifecycle code,
so the repair lives in generic scan/vacuum paths instead of the TurboQuant
exact-score experiments.

### 3. Task-16 can keep moving without the prior vacuum-safety blocker

This clears the previously red concurrency rerun from the `428` / `432`
feedback thread and restores the expected scratch proof path for concurrent
insert/scan/vacuum activity on current head.
