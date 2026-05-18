# Review Request: C1 ADR-030 V2 External Recall Smoke Storage Formats

## Context

Packet 398 made the real-corpus harness storage-format-aware:

- `scripts/load_real_corpus.py --storage-format ...` now builds coexisting
  `<prefix>_<storage_format>_m{N}_idx` families on one shared staged corpus
- `scripts/run_real_corpus_recall_scratch.sh` can now target those families
  through a derived fixture/index prefix
- `docs/RECALL_REAL_CORPUS.md` now documents the shared-table /
  format-specific-index contract

But the Rust-side external recall smoke surface in `src/lib.rs` still only
proved the legacy single-family layout:

- one shared corpus table
- one shared query table
- one legacy index family `<prefix>_m{N}_idx`

That left the in-tree proof surface behind the harness contract we just
documented.

## Problem

Without this slice, the branch could claim that the external recall harness
supports coexisting `TurboQuant` and `PqFastScan` index families, but the
only Rust smoke test still exercised the older one-family shape.

That is a proof mismatch:

1. the loader/runner now derive storage-format-specific index prefixes
2. the SQL helper already accepts a free-form fixture/index prefix
3. the smoke surface was not proving that those pieces actually compose

## Planned Slice

One ignored pg-test checkpoint:

1. split the external recall smoke fixture into shared-table and index-family
   helpers
2. add tiny helper functions for derived external recall index prefixes/names
3. build three families on one staged corpus:
   - legacy/default
   - explicit `turboquant`
   - explicit `pq_fastscan`
4. run the same summary + gate smoke assertions against each family

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Split the smoke fixture into shared tables plus index families

Replaced the one-shot helper shape with:

- `create_external_recall_smoke_tables(...)`
- `create_external_recall_smoke_indexes(...)`

This keeps the corpus/query tables shared while letting the smoke test attach
multiple index families to those same tables.

### 2. Added small derived-prefix helpers

Added:

- `external_recall_index_prefix(...)`
- `external_recall_index_name(...)`
- `external_recall_storage_format_clause(...)`

These mirror the new loader behavior:

- legacy/default family: `<prefix>_m{N}_idx`
- explicit family: `<prefix>_<storage_format>_m{N}_idx`

### 3. Added one shared probe assertion helper

Added:

- `assert_external_recall_smoke_probe(...)`

This runs the existing external summary and gate surfaces against one chosen
index family and asserts:

- expected `(m, ef_search, corpus_rows, query_count)`
- sane metric ranges
- summary determinism across reruns
- one gate row per configured A4 checkpoint

### 4. Replaced the old smoke test with multi-family proof

Added/replaced:

- `test_tqhnsw_recall_external_smoke_500_formats`

The ignored pg test now:

1. seeds one shared synthetic external recall corpus/query pair
2. builds three index families on top of that same data:
   - legacy/default
   - explicit `turboquant`
   - explicit `pq_fastscan`
3. runs the same external summary + gate assertions against each family

That is the in-tree proof that the new storage-format-aware harness contract is
real, not just documented.

## Measurements

No benchmark or real-corpus rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This slice upgrades the Rust proof surface to match the new harness contract:

1. the external recall smoke path now proves coexisting format-specific index
   families on one shared staged corpus/query pair
2. the same SQL helper APIs are exercised for legacy/default, explicit
   `turboquant`, and explicit `pq_fastscan`
3. packet 398's loader/runner changes now have a matching in-tree smoke proof

What this slice intentionally does **not** do:

- run the ignored smoke test on this workstation
- change the public external recall SQL API
- change any AM/runtime behavior

## Next Slice

The remaining harness follow-up is mostly operator ergonomics:

1. review whether wrapper scripts like `prepare_real_corpus_scratch.sh` should
   forward explicit `--storage-format`
2. tighten the latency/recall launcher docs around explicit-format index names
3. then move from proof surfaces to an actual real-corpus rerun when the
   environment is ready
