# Task 92 Phase 1 Counter and Suite Contract

Head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`

This addendum makes the Task 92 Phase 2 and Phase 5 implementation contracts
explicit. It is based on the current Task 87 counter surface in
`src/am/common/candidate_batch.rs`, `src/lib.rs`, and
`crates/ecaz-cli/src/commands/bench/mod.rs`.

## Current Surface

The current SQL functions are:

- `ec_task87_candidate_batch_scoring_reset()`
- `ec_task87_candidate_batch_scoring_snapshot()`

The snapshot rows currently expose:

- `surface`
- `flushes`
- `candidates`
- `elapsed_nanos`
- `elapsed_ms`
- `lut32_flushes`
- `lut32_candidates`

The CLI parser currently prints `[task87-counters]` lines and assumes the same
AM-only rows.

## Required New Identity

Task 92 should introduce a kernel counter identity:

```rust
pub(crate) struct BlockKernelCounterKey {
    pub(crate) surface: CandidateBatchScoringSurface,
    pub(crate) quant_kind: QuantCodecKind,
    pub(crate) isa: Isa,
}
```

Add `Diskann` to `CandidateBatchScoringSurface` before DiskANN migration begins.

`Isa` is the ADR-076 enum:

```rust
pub(crate) enum Isa {
    Scalar,
    Neon,
    Sve,
    Avx2,
}
```

## Required Snapshot Fields

The new SQL snapshot should expose one row per observed
`(surface, quant_kind, isa)` tuple:

- `surface text`
- `quant_kind text`
- `isa text`
- `flushes bigint`
- `candidates bigint`
- `elapsed_nanos bigint`
- `elapsed_ms double precision`
- `kernel_flushes bigint`
- `kernel_candidates bigint`
- `kernel_elapsed_nanos bigint`
- `kernel_elapsed_ms double precision`
- `scalar_flushes bigint`
- `scalar_candidates bigint`
- `scalar_elapsed_nanos bigint`
- `scalar_elapsed_ms double precision`

`flushes/candidates/elapsed_*` are total successful scoring work for the row.
The `kernel_*` and `scalar_*` fields split that total by the path actually used.

## Compatibility Plan

Do not remove Task 87 SQL functions in the first Task 92 counter slice.

Phase 2 should add new functions while keeping the old names as compatibility
wrappers:

- new: `ec_block_kernel_scoring_reset()`
- new: `ec_block_kernel_scoring_snapshot()`
- compatibility: `ec_task87_candidate_batch_scoring_reset()` calls the new reset
- compatibility: `ec_task87_candidate_batch_scoring_snapshot()` returns the old
  AM-only shape aggregated from new counters where possible

The CLI should prefer the new function and fall back to the Task 87 function
when running against an older extension binary. This keeps existing benchmark
parsers usable during the transition.

## Counter Semantics

Counters increment only after successful scoring.

Path classification:

- `kernel_*`: candidates scored by a block kernel because width and runtime ISA
  allowed it.
- `scalar_*`: candidates scored by scalar reference because kernel routing was
  disabled, width was below 32, a tail remained after whole blocks, or the host
  lacked a supported ISA.

For a mixed batch with whole blocks plus scalar tail, increment the whole-block
work under the selected ISA row and increment the scalar tail under
`isa=Scalar`. Do not record scalar tails under the selected ISA row's
`scalar_*` fields. This is the binding convention for Tasks 93-98 and keeps the
Task 99 cross-quant matrix comparable across hosts and dispatch outcomes.

## Off-Path Scalar Measurement

When `ec_*.candidate_batch_scoring=off`, the same workload should accumulate
`scalar_*` counters at the per-candidate scorer. That is the required off-path
comparison for Tasks 93-98 speedup claims.

The scalar counter must wrap the existing scalar call without changing:

- candidate order;
- score polarity;
- pruning/bound decision order;
- Task 87 Phase 6 reproducibility inputs.

## CLI Output Contract

The benchmark CLI should print one line per snapshot row:

```text
[block-kernel-counters] command=<command> label=<label> surface=<surface> quant=<quant_kind> isa=<isa> flushes=<n> candidates=<n> elapsed_nanos=<n> elapsed_ms=<ms> kernel_flushes=<n> kernel_candidates=<n> kernel_elapsed_nanos=<n> kernel_elapsed_ms=<ms> scalar_flushes=<n> scalar_candidates=<n> scalar_elapsed_nanos=<n> scalar_elapsed_ms=<ms>
```

Keep `[task87-counters]` output while compatibility SQL exists, or emit both
formats for one implementation slice. Closeout can remove the old line only
after suite parsers and packet manifests have moved to the new line.

## Bench Suite Quant Axis

Suite config should add `quant` as an optional step axis value, parallel to the
existing tag-driven storage-format labels.

Dry-run expansion must report:

- `valid`: AM supports the quant and kernel state is implemented or scalar
  fallback exists.
- `missing_kernel`: AM/quant exists but the requested ISA kernel is reserved
  for Tasks 93-98 and not implemented yet.
- `structurally_absent`: AM does not support that quant/storage combination.
- `invalid_config`: the suite requested an impossible combination, such as
  grouped-PQ without model-building setup.

The runner should write those statuses into `suite-manifest.json` and
`results.jsonl` so packet manifests do not have to infer missing cells from
absent logs.

## Phase 2 Acceptance Evidence

The counter-surface implementation packet should include:

- focused Rust tests for counter reset/snapshot aggregation;
- SQL-level smoke output for the new snapshot function on PG18 if callbacks are
  touched;
- CLI parser test proving new `[block-kernel-counters]` lines parse;
- compatibility test or log proving old `[task87-counters]` output still works;
- an artifact explaining whether scalar tails are attributed to `isa=Scalar` or
  to the selected ISA row's `scalar_*` fields.
