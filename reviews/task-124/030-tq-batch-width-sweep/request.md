# Review Request: Task 124 / 030 TQ Batch Width Sweep

## Summary

This slice addresses required Task 124 lever 3: **batch/flush width**.

Change:

- Added an ignored TQ-internal batch-width profiler for the no-QJL 4-bit IVF scorer surface.
- Changed the no-QJL LUT batch scorer wrapper to cascade over `CandidatePayload` directly instead of allocating and filling a per-flush `Vec<&[u8]>`.
- Kept validation and bit-exact scoring behavior unchanged.

This is a TurboQuant scorer-path optimization. It reports `ns/candidate` on the TQ batch scorer, not f32 comparison, storage, nprobe, or end-to-end latency.

## TQ-Internal Result

Primary comparable hot-run deltas:

| Width | Before ns/candidate | After ns/candidate | Delta |
| --- | ---: | ---: | ---: |
| 32 | 254.1 | 236.4 | -7.0% |
| 64 | 265.3 | 245.8 | -7.4% |
| 75 | 282.4 | 271.1 | -4.0% |
| 96 | 266.6 | 254.9 | -4.4% |
| 100 | 275.4 | 264.7 | -3.9% |
| 128 | 264.5 | 254.7 | -3.7% |
| 256 | 266.4 | 257.8 | -3.2% |

The sweep also confirms the expected width shape: exact 32-candidate block multiples are cheapest; widths such as 75 and 100 pay a partial-tail cost.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test --release --lib --features bench am::common::candidate_batch::tests::turboquant_lut_batch_matches_scalar_tail -- --nocapture`
- `cargo test --release --lib --features bench am::common::candidate_batch::tests::turboquant_lut_batch_records_surface_counters -- --nocapture`
- `cargo test --release --lib --features bench am::ec_ivf::quantizer::tests::turboquant_no_qjl_4bit_batch_scores_match_scalar_scores -- --nocapture`
- `ECAZ_TQ_BATCH_WIDTH_PROFILE_CANDIDATES=256000 ECAZ_TQ_BATCH_WIDTH_PROFILE_LOG=reviews/task-124/030-tq-batch-width-sweep/artifacts/tq-batch-width-direct-payload-rerun.log cargo test --release --lib --features bench task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture`

## Scope

Kept:

- Direct-payload no-QJL TQ batch scorer cascade.
- Batch-width profiler for future Task 124 flush-width checks.

Not changed:

- Stage-2 rerank width policy. Width policy affects recall and must be changed only with a separate recall/latency benchmark slice.

Task 124 remains open. Completed required levers so far on this reopened scorer-focused pass:

- Scoring kernel profile/attempt: packet 028.
- Per-query LUT/query-prep: packet 029.
- Batch/flush width: this packet.

