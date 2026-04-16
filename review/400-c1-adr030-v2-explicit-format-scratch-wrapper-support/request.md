# Review Request: C1 ADR-030 V2 Explicit Format Scratch Wrapper Support

## Context

Packets 398 and 399 moved the real-corpus recall harness onto the first-class
storage-format story:

- the loader can now build coexisting `<prefix>_<storage_format>_m{N}_idx`
  families
- the scratch recall runner can now target those families
- the Rust external recall smoke surface now proves legacy/default,
  explicit `turboquant`, and explicit `pq_fastscan` families on one shared
  staged corpus/query pair

But two operator-facing wrappers still lagged behind:

- `scripts/prepare_real_corpus_scratch.sh` could not forward an explicit
  `--storage-format` into the scratch loader
- `scripts/bench_sql_latency_verified.sh` still only derived
  `<prefix>_m{N}_idx` unless the operator manually spelled `--index-name`

That meant the explicit-format path was documented and partially automated, but
the scratch convenience wrappers still pushed operators back toward ad hoc
manual index naming.

## Problem

Without this slice:

1. the repo-local scratch "convert + load" wrapper could not directly stage an
   explicit `turboquant` or `pq_fastscan` family
2. the planner-verified latency launcher could not derive the matching
   explicit-format index name even though the loader now creates one
3. the only ergonomic way to bench explicit-format indexes was to manually pass
   `--index-name`

That is not a runtime gap. It is operator-harness friction.

## Planned Slice

One scripts/docs checkpoint:

1. let `prepare_real_corpus_scratch.sh` forward explicit storage format
2. let the verified latency launcher derive explicit-format index names
3. keep `--index-name` as the highest-precedence override
4. add regression coverage for the verified launcher
5. tighten the real-corpus docs so the scratch-wrapper path matches the new
   behavior

No AM behavior change.

## Implementation

Updated:

- `scripts/prepare_real_corpus_scratch.sh`
- `scripts/bench_sql_latency_verified.sh`
- `scripts/tests/test_bench_sql_latency_verified.py`
- `docs/RECALL_REAL_CORPUS.md`

### 1. Scratch prepare wrapper now forwards `--storage-format`

Added to `scripts/prepare_real_corpus_scratch.sh`:

- `--storage-format turboquant|pq_fastscan`

The wrapper now validates the value and forwards it into
`scripts/load_real_corpus_scratch.sh`, which already forwards into the storage-
format-aware Python loader from packet 398.

### 2. Verified latency launcher now derives explicit-format index names

Added to `scripts/bench_sql_latency_verified.sh`:

- `--storage-format turboquant|pq_fastscan`

Resolution precedence is now:

1. explicit `--index-name`
2. derived `<prefix>_<storage_format>_m{N}_idx` when `--storage-format` is set
3. legacy/default `<prefix>_m{N}_idx`

When the launcher derives an explicit-format name, it also forwards the derived
`--index-name` to `scripts/bench_sql_latency.sh` so the delegate script and the
planner verification step both resolve the same target index.

### 3. Added verified-launcher regression coverage

Updated `scripts/tests/test_bench_sql_latency_verified.py`:

- `_run_verified(...)` now supports a `storage_format` parameter
- added `test_verified_launcher_derives_explicit_storage_format_index_name`

That test proves the verified launcher:

- derives `tqhnsw_real_test_pq_fastscan_m8_idx`
- prints the canonical planner-verification banner for that derived index
- successfully runs the measured cell through the fake `psql` harness

### 4. Docs now mention the wrapper path explicitly

Updated `docs/RECALL_REAL_CORPUS.md` to note:

- `scripts/prepare_real_corpus_scratch.sh` can now take `--storage-format`
- the verified latency launcher can now derive explicit-format index names via
  `--storage-format pq_fastscan`

## Measurements

No benchmark or real-corpus rerun in this slice.

## Validation

Passed:

- `python3 scripts/tests/test_bench_sql_latency_verified.py`
- `scripts/tests/run.sh`
- `bash -n scripts/bench_sql_latency_verified.sh`
- `bash -n scripts/prepare_real_corpus_scratch.sh`
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

This slice finishes the main scratch-wrapper ergonomics for explicit-format
real-corpus work:

1. the scratch prepare wrapper can now stage explicit `turboquant` /
   `pq_fastscan` families directly
2. the verified latency launcher can now derive the matching explicit-format
   index name without manual `--index-name`
3. the wrapper docs and regression tests now reflect the same explicit-format
   workflow the loader/harness already supports

What this slice intentionally does **not** do:

- rerun the real-corpus recall or latency lanes
- change the underlying latency benchmark delegate script's public interface
- change any AM/runtime behavior

## Next Slice

The remaining meaningful work is no longer harness plumbing. It is execution:

1. run the real-corpus recall/latency lanes against the explicit
   `turboquant` and `pq_fastscan` families when the environment is available
2. compare those results against the task-15 landing bar
3. then process any outside review feedback that lands on packets 398–400
