# Task 111e: Coarse Rerank Preset

## Summary

This packet adds an explicit gated IVF mode:

```text
storage_format = 'coarse_rerank'
```

The preset resolves to the Task 111e baseline contract:

- dense page-local RaBitQ-1 coarse postings,
- heap/table-side f32 rerank,
- existing `rerank_width` for candidate frontier width,
- existing RaBitQ storage/scoring machinery underneath.

It is intentionally a narrow preset, not a new page format. The Phase 1 packet
showed that the 50k dense RaBitQ-1 candidate frontier reaches near-plateau
recall by candidate_k 50-100, so this slice creates the durable reloption
surface for the heap-f32 baseline path before broader representation/placement
work.

## Code Under Review

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/quantizer.rs`
- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/cost.rs`
- `src/am/ec_ivf/scan.rs`

## Behavior

`storage_format = 'coarse_rerank'`:

- parses as a first-class storage format and persists through metadata;
- resolves the quantizer to RaBitQ;
- forces `quant_bits = 1`;
- enables dense posting blocks;
- forces one-page dense groups with typed layout;
- resolves `rerank = auto` to `rerank = heap_f32`;
- allows explicit `rerank = heap_f32`;
- rejects `rerank = off` and `rerank = source_column`;
- uses RaBitQ scoring names/multipliers and scan batch gating.

## Validation

Artifacts are under `reviews/task-111e/002-coarse-rerank-preset/artifacts/`.

```text
cargo test -q coarse_rerank --lib --no-default-features --features pg18
4 passed; 0 failed; 2131 filtered out

cargo test -q metadata_roundtrip --lib --no-default-features --features pg18
8 passed; 0 failed; 2127 filtered out

cargo test -q scratch_soa_batch_decode_gate --lib --no-default-features --features pg18
1 passed; 0 failed; 2134 filtered out
```

## Review Ask

Please review whether this preset is the right durable Task 111e heap-f32
baseline contract before adding 100k measurement coverage and compact quantized
rerank variants.
