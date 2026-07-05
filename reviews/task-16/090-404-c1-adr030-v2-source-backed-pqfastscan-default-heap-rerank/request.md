# Review Request: C1 ADR-030 V2 Source-Backed PqFastScan Default Heap Rerank

## Context

After packets `401` through `403`, the branch had a much clearer task-15
landing picture:

- `turboquant` already passed the canonical `50k` real-corpus gate
- `pq_fastscan` was on the stronger `binary + window=64` lane by default
- but canonical `pq_fastscan` still narrowly missed one gate row on the full
  `1000`-query harness:
  - `m=8, ef=128`: `0.8826 Recall@10` vs gate `0.89`

That miss was small enough that the next move should be an operating-point
correction, not another storage-format redesign.

I checked the live explicit-format families on the real `~/.pgrx` cluster and
remeasured them on the current branch build:

- explicit `turboquant` gate still passed cleanly
- explicit `pq_fastscan` on the source-backed real corpus still failed only the
  `m=8, ef=128` row on the default quantized-rerank lane

The obvious remaining lever was rerank mode: all of these canonical
`pq_fastscan` real-corpus indexes are built with `build_source_column='source'`,
so the runtime has a raw heap `real[]` column available for exact rerank.

## Problem

The source-backed `pq_fastscan` path was still defaulting to quantized rerank
even though:

1. the index has an explicit raw-source heap column available
2. runtime already has a working `heap_f32` rerank path for that source-backed
   layout
3. a quick live-cluster experiment showed that switching only rerank mode from
   `quantized` to `heap_f32` was enough to move the failing row from:
   - `0.8826 Recall@10` to `0.9078 Recall@10`

So the branch was leaving recall on the table by default on exactly the
source-backed `pq_fastscan` indexes task 15 is trying to land first-class.

## Planned Slice

Make source-backed `pq_fastscan` default to heap rerank while preserving the
existing override surface:

1. if `TQVECTOR_PQ_FASTSCAN_RERANK_MODE` is set, honor it exactly
2. otherwise, default source-backed `pq_fastscan` scans to `heap_f32`
3. keep source-less fallback at `quantized`
4. update the runtime-settings/debug surface to report the new first-class
   default
5. update the scratch restart wrapper so its no-flag path matches the new
   runtime default
6. add regression coverage that the source-backed default path actually emits
   heap-exact rerank scores

This slice intentionally does not:

- remove `quantized` rerank support
- change `turboquant`
- change on-disk `pq_fastscan` layout
- add new reloptions or GUCs

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

Concrete changes:

1. added a source-aware `default_grouped_rerank_mode(...)` in `src/am/scan.rs`
2. changed `configure_grouped_heap_rerank_state(...)` so `pq_fastscan` now
   defaults to:
   - `heap_f32` when `build_source_column` is present
   - `quantized` otherwise
3. kept explicit env override precedence unchanged:
   - `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=quantized|heap_f32`
4. changed the exported default-mode constant to `heap_f32`
5. updated `current_pq_fastscan_runtime_settings()` /
   `tqhnsw_debug_pq_fastscan_runtime_settings()` /
   `tqhnsw_debug_adr030_runtime_settings()` to report the new first-class
   source-backed default:
   - rerank mode = `heap_f32`
   - rerank source label = `build_source_column`
6. generalized the pg helper that collects rerank comparison rows so tests can
   exercise:
   - default source-backed rerank
   - explicit `heap_f32`
   - bytea source override
7. added a new pg regression:
   - `test_pq_fastscan_default_source_rerank_emits_heap_scores`
8. kept quantized-override coverage explicit by pinning the quantized profile
   tests with `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=quantized`
9. updated the canonical rerank-profile SQL-surface test so the default
   source-backed path now expects heap-rerank counters instead of quantized
   counters
10. changed `scripts/restart_adr030_scratch.sh` default rerank mode from
    `quantized` to `heap_f32`

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `bash -n scripts/restart_adr030_scratch.sh`

Required full-test commands were run and still hit the same workstation linker
boundary as the rest of this branch:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Live Runtime Validation

Because the local `cargo test` / `cargo pgrx test` lanes are still linker-blocked
on this workstation, I also validated the changed runtime on the explicit
`~/.pgrx` cluster.

### Before the code change: default quantized rerank on current branch build

Runtime settings:

- `grouped_scan_window = 64`
- `grouped_scan_score_mode = binary`
- `grouped_scan_rerank_mode = quantized`
- `grouped_exact_traversal_enabled = false`

Canonical `50k` explicit `pq_fastscan` gate:

| m | ef_search | Recall@10 | gate | passes |
|---|-----------|-----------|------|--------|
| 8 | 40  | 0.8084 | —    | true |
| 8 | 128 | 0.8826 | 0.89 | false |
| 8 | 200 | 0.8906 | —    | true |
| 16 | 200 | 0.9280 | —   | true |

Artifact:

- `tmp/real_corpus_runs/20260416T235926Z_gate_tqhnsw_real_50k_pq_fastscan_tqhnsw_real_50k_queries.tsv`

### Manual experiment: same lane, heap rerank enabled

Restarted the explicit cluster with:

- `window=64`
- `grouped_score_mode=binary`
- `rerank_mode=heap_f32`
- `rerank_source_column=source`

Focused `m=8, ef=128` summary on the full `1000`-query lane:

- `Recall@10 = 0.9078`
- `exact-quantized Recall@10 = 0.9144`

Artifact:

- `tmp/real_corpus_runs/20260417T001014Z_summary_tqhnsw_real_50k_pq_fastscan_m8_idx_m8_ef128_tqhnsw_real_50k_queries.tsv`

Full canonical `50k` explicit `pq_fastscan` gate on that heap-rerank lane:

| m | ef_search | Recall@10 | gate | passes |
|---|-----------|-----------|------|--------|
| 8 | 40  | 0.8231 | —    | true |
| 8 | 128 | 0.9078 | 0.89 | true |
| 8 | 200 | 0.9174 | —    | true |
| 16 | 200 | 0.9671 | —   | true |

Artifact:

- `tmp/real_corpus_runs/20260417T001149Z_gate_tqhnsw_real_50k_pq_fastscan_tqhnsw_real_50k_queries.tsv`

### After the code change: default path aligned

Installed the current branch into `~/.pgrx`, updated the local restart wrapper
default, and restarted with the plain default operator path:

- `./scripts/restart_adr030_scratch.sh --window 64 --grouped-score-mode binary`

The compatibility runtime-settings helper on that live cluster now reports:

- `grouped_scan_window = 64`
- `grouped_scan_score_mode = binary`
- `grouped_scan_rerank_mode = heap_f32`
- `grouped_scan_rerank_source_column = build_source_column`

Canonical `50k` explicit `pq_fastscan` gate on that default path:

| m | ef_search | Recall@10 | gate | passes |
|---|-----------|-----------|------|--------|
| 8 | 40  | 0.8231 | —    | true |
| 8 | 128 | 0.9078 | 0.89 | true |
| 8 | 200 | 0.9174 | —    | true |
| 16 | 200 | 0.9671 | —   | true |

Artifact:

- `tmp/real_corpus_runs/20260417T002339Z_gate_tqhnsw_real_50k_pq_fastscan_tqhnsw_real_50k_queries.tsv`

## Outcome

This slice turns the last meaningful `pq_fastscan` task-15 gap from a runtime
experiment into the default source-backed path:

1. source-backed `pq_fastscan` no longer leaves quantized rerank on by default
2. explicit `quantized` override still exists and is still covered
3. the local scratch restart wrapper now matches the intended first-class lane
4. the canonical explicit `pq_fastscan` family passes the full `50k` gate on
   that aligned default path

That does **not** mean the whole branch is done, but it materially changes the
remaining work:

- this is no longer a “make `pq_fastscan` clear the real-corpus harness”
  problem
- it is now primarily a “capture the final landing proof cleanly and finish
  the remaining branch hygiene” problem

## Next Slice

Write the landing-proof packet that ties together:

1. explicit `turboquant` `50k` gate pass
2. explicit `pq_fastscan` `50k` gate pass on the new default lane
3. the already-landed insert/vacuum round-trip proof from packet `393`

That packet should say, concretely, whether task 15 is now satisfied except for
branch-to-main merge mechanics.
