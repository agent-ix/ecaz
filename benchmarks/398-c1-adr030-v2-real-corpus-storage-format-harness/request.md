# Review Request: C1 ADR-030 V2 Real-Corpus Storage-Format Harness

## Context

Task 15 and ADR-032 now treat `TurboQuant` and `PqFastScan` as first-class
coexisting formats selected per index through `storage_format`.

The branch already has:

- reloption-backed build/runtime support for both formats
- pg-test proof for build/insert/vacuum round-trips on both formats
- canonical `pq_fastscan` runtime/debug naming on most of the branch

But the real-corpus harness still assumed one legacy index family:

- `scripts/load_real_corpus.py` only built `<prefix>_m{N}_idx`
- the loader only checked reloptions `(m, ef_construction, build_source_column)`
- `scripts/run_real_corpus_recall_scratch.sh gate` always passed the base
  `<prefix>` into `tqhnsw_graph_scan_recall_external_gate_report(...)`
- the docs only described the single-family `<prefix>_m{N}_idx` path

That made the task-15 landing proof awkward: one staged corpus could not keep
explicit `turboquant` and `pq_fastscan` index families side by side without
manual naming.

## Problem

Without this slice, the branch could claim both formats are first-class, but
the real-corpus harness still behaved like a single-format surface.

That leaves two practical problems:

1. operators cannot stage `turboquant` and `pq_fastscan` indexes together on
   the same loaded `<prefix>_corpus` / `<prefix>_queries` tables without
   ad hoc manual index naming
2. the documented scratch runner path cannot target a format-specific external
   recall gate family without spelling the SQL call manually

This is not an AM-runtime bug. It is a landing-proof and operational-harness
gap.

## Planned Slice

One scripts/docs checkpoint:

1. teach the loader about explicit `--storage-format`
2. derive coexisting format-specific index families without changing table names
3. let the scratch recall runner target those format-specific families
4. document the coexistence contract
5. add focused Python regression coverage for the loader helpers

No AM behavior change.

## Implementation

Updated:

- `scripts/load_real_corpus.py`
- `scripts/run_real_corpus_recall_scratch.sh`
- `docs/RECALL_REAL_CORPUS.md`
- `scripts/tests/test_load_real_corpus_storage_format.py`

### 1. Loader now supports explicit storage-format index families

Added to `scripts/load_real_corpus.py`:

- `--storage-format {turboquant,pq_fastscan}`
- `_index_prefix(...)`
- `_index_name(...)`
- `_expected_index_reloptions(...)`
- `_build_index_sql(...)`

Behavior:

- default/legacy run:
  - table names remain `<prefix>_corpus` / `<prefix>_queries`
  - index names remain `<prefix>_m{N}_idx`
- explicit format run:
  - table names still remain `<prefix>_corpus` / `<prefix>_queries`
  - index names become `<prefix>_<storage_format>_m{N}_idx`
  - reloption checks now also include `storage_format=<...>`

That means one staged corpus can now hold:

- `tqhnsw_real_50k_turboquant_m8_idx`
- `tqhnsw_real_50k_pq_fastscan_m8_idx`

without duplicating tables or relying on hand-written `CREATE INDEX` SQL.

### 2. Scratch recall runner can now target the derived family

Updated `scripts/run_real_corpus_recall_scratch.sh`:

- new optional `--storage-format turboquant|pq_fastscan`
- gate mode now derives the fixture/index prefix as `<prefix>_<storage_format>`
  when requested and passes that derived prefix to
  `tests.tqhnsw_graph_scan_recall_external_gate_report(...)`
- summary mode can now derive the index name from `--prefix`, `--m`, and
  optional `--storage-format` instead of always requiring `--index`

This keeps the shared table names stable while letting the runner select the
coexisting index family cleanly.

### 3. Docs now describe legacy and explicit coexistence modes

Updated `docs/RECALL_REAL_CORPUS.md` to describe:

- the legacy/default `<prefix>_m{N}_idx` family
- the explicit-format coexistence families:
  - `<prefix>_turboquant_m{N}_idx`
  - `<prefix>_pq_fastscan_m{N}_idx`
- example loader invocations for both formats
- the fact that the external gate helper's third argument is the
  fixture/index prefix, not the table prefix

### 4. Added focused loader regression tests

Added `scripts/tests/test_load_real_corpus_storage_format.py` covering:

- legacy/default index naming remains unchanged
- explicit `turboquant` / `pq_fastscan` derive coexisting prefixes
- reloption expectations include `storage_format` only when requested
- generated `CREATE INDEX` SQL includes `storage_format` only when requested
- invalid storage-format values are rejected

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `scripts/tests/run.sh`
- `python3 -m py_compile scripts/load_real_corpus.py`
- `bash -n scripts/run_real_corpus_recall_scratch.sh`
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

This slice closes a real task-15 harness gap without changing AM logic:

1. one staged corpus can now keep explicit `turboquant` and `pq_fastscan`
   index families side by side
2. the loader and scratch runner now encode the same coexistence rule instead
   of requiring manual SQL naming
3. the docs now explain the difference between shared table names and
   format-specific index prefixes

What this slice intentionally does **not** do:

- run the 50k real-corpus gate itself
- change the Rust external recall helper API
- add new AM/runtime behavior

## Next Slice

The next high-value follow-up is to strengthen the Rust-side external recall
smoke surface so it exercises coexisting format-specific index families too,
not just the legacy single-family path.
