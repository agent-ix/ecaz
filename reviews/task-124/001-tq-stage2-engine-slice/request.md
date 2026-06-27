# Review Request: Task 124 TQ Stage-2 Engine Slice

## Summary

This is the first TurboQuant-focused Task 124 engine checkpoint. It is not a closeout.

Implemented a disabled-by-default IVF path for:

- RaBitQ/normal candidate frontier up to existing `rerank_width`;
- index-side TurboQuant sidecar scoring over that frontier;
- exact/source f32 final rerank over a new bounded `stage2_final_rerank_width`.

The initial runtime fixture uses `rerank_width = 8` and `stage2_final_rerank_width = 3`; the product path will start from width 25 per the Task 122 sidecar result.

## Code Changes

- Added `ec_ivf.stage2_final_rerank_width` GUC and `stage2_final_rerank_width` reloption, default `0` / disabled.
- Validated nonzero `stage2_final_rerank_width` is only accepted for `storage_format='coarse_rerank'`, `rerank_placement='index'`, `rerank_format='turboquant'`.
- Added scan logic that runs the existing index-side TQ sidecar scorer first, sorts the stage-2 prefix, then exact-reranks only the final bounded prefix through source/f32.
- Preserved existing behavior when `stage2_final_rerank_width = 0`.
- Added a PG18 runtime test proving TQ payload scoring still occurs and exact source reads are bounded to the final width.

## TurboQuant Audit

Packet-local audit: `artifacts/tq-score-surface-audit.md`.

Key point: the Task 124 hot path is not the scalar exact-dequant TQ path. It uses the index-side sidecar borrowed payload-ref batch route:

`score_sidecar_payload_refs_batch_with_centroid_ips` -> `score_turboquant_batch_from_payload_refs` -> `candidate_batch`.

The focused runtime test asserts the current TQ path scores payload bytes and avoids the survivor payload slab copy.

## Validation

- `cargo test -p ecaz am::ec_ivf::options`
  - `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 2197 filtered out`
- `cargo test -p ecaz am::ec_ivf::scan`
  - `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out`
- `cargo pgrx test pg18 test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads`
  - `test tests::pg_test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2222 filtered out`

Logs and manifest are under `artifacts/`.

## Known Gaps / Next Work

- This does not add Phase 3 attribution counters yet. Existing `rerank_rows` is cumulative across TQ stage-2 and final exact f32; the test therefore asserts source bytes and payload bytes instead.
- This does not include the required 10k / 50k / 100k A/B benchmark matrix and must not be promoted as Task 124 complete.
- Next slice should expose stage-2/final row counts and scorer telemetry through the bench surfaces, then run the width-25 A/B suite against RaBitQ + f32.
