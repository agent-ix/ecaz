# Review Request: C1 Task16 TurboQuant `rerank_source_column`

Current head at execution: `95bf079`

## Context

Packet `431` productized persisted `rerank_source_column`, but only for
`storage_format = 'pq_fastscan'`. Reviewer feedback on `431` called out the
remaining gap: task 16's serious-lane source-raw win was measured on the
TurboQuant V3 path, while the productized reloption was still fenced away from
TurboQuant indexes.

This slice closes that product gap without changing TurboQuant's default
rerank policy.

## What Landed

### Reloption / build validation

- `src/am/options.rs`
  - removed the reloption parser gate that rejected
    `rerank_source_column` unless `storage_format = 'pq_fastscan'`
  - widened the reloption help text from `pq_fastscan heap_f32 rerank` to the
    generic grouped heap-rerank path
- `src/am/build.rs`
  - generalized `validate_pq_fastscan_rerank_source_column*` into
    `validate_grouped_rerank_source_column*`
  - build and empty-build validation now applies whenever
    `rerank_source_column` is present, regardless of storage format
  - type policy stays `real[] or bytea`

### Runtime behavior

- `src/am/scan.rs`
  - no default-policy change: TurboQuant still resolves to quantized rerank
    unless heap rerank is explicitly selected
  - once `heap_f32` is selected, TurboQuant now uses the same persisted
    `rerank_source_column > build_source_column` source-selection path that was
    already shared by the grouped heap-rerank machinery
  - generalized heap-rerank error text from `PqFastScan`-specific wording to
    the shared grouped path

### Coverage

- `src/lib.rs`
  - extended the TurboQuant binary runtime fixture so it can persist a
    `source_raw bytea` rerank column alongside `source real[]`
  - added a distinct `source_raw` payload so the test can prove TurboQuant heap
    rerank is using `rerank_source_column`, not silently falling back to
    `build_source_column`
  - added `test_turboquant_persisted_rerank_source_default_stays_quantized`
  - added `test_turboquant_persisted_bytea_rerank_emits_scores`
  - replaced the old non-TurboQuant reloption rejection test with
    `test_turboquant_rerank_source_rejects_wrong_type`
  - updated the invalid-rerank-mode expectation to the new shared grouped error
    string

## Readout

### 1. TurboQuant can now persist a dedicated raw rerank source

Users can set:

```sql
WITH (
  build_source_column = 'source',
  rerank_source_column = 'source_raw',
  storage_format = 'turboquant'
)
```

and get build-time validation plus runtime access to that raw source when
`heap_f32` rerank is selected.

### 2. Default TurboQuant behavior stays unchanged

This slice does **not** flip TurboQuant to heap rerank by default. The new pg
coverage locks in that a source-backed TurboQuant index with persisted
`rerank_source_column` still runs the quantized default lane unless rerank mode
is explicitly overridden.

### 3. Explicit heap rerank now honors `rerank_source_column` precedence

The new TurboQuant bytea test uses different values in `source` vs
`source_raw`, then verifies the emitted exact scores match `source_raw`. That
proves the runtime path is actually consuming the persisted rerank-source
column instead of merely accepting it at DDL time.

## Validation

Green checkpoint on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Remaining Task-16 Gap

This slice productizes the persisted source selector for TurboQuant, but it is
still not a measurement packet. The serious-lane question remains: how much of
the remaining TurboQuant gap is recovered once the supported TurboQuant path is
measured with persisted `source_raw` instead of env-only overrides?
