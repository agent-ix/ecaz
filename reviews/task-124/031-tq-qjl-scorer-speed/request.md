# Review Request: Task 124 / 031 TQ QJL Scorer Speed

## Summary

This slice addresses required Task 124 lever 6: **QJL scoring speed**.

Change:

- Added an ignored QJL scorer-width profiler for the QJL-active TurboQuant 4-bit surface.
- Changed the QJL batch scorer wrapper to cascade over validated `CandidatePayload` entries directly instead of allocating and filling a temporary `Vec<(&[u8], f32)>` per flush.
- Kept QJL block32/octet/scalar scorer semantics unchanged.

This is a TurboQuant QJL scorer-path optimization. It reports `ns/candidate` on the QJL scorer wrapper, not f32 comparison, storage, nprobe, or end-to-end latency.

## TQ-Internal Result

Primary comparable hot-run deltas:

| Width | Before ns/candidate | After ns/candidate | Delta |
| --- | ---: | ---: | ---: |
| 8 | 182.2 | 178.9 | -1.8% |
| 16 | 181.7 | 178.8 | -1.6% |
| 25 | 200.7 | 197.0 | -1.8% |
| 32 | 179.2 | 176.2 | -1.7% |
| 64 | 178.5 | 176.0 | -1.4% |
| 96 | 178.1 | 176.4 | -1.0% |
| 100 | 195.8 | 194.0 | -0.9% |
| 128 | 177.7 | 175.9 | -1.0% |

The sweep also confirms the QJL scorer is not scalar-only on this host: block/octet widths dispatch through the NEON QJL path.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test --release --lib --features bench am::ec_ivf::quantizer::tests::common_quant_codec_turboquant_batch_is_bit_exact_with_scalar -- --nocapture`
- `cargo test --release --lib --features bench quant::qjl32::tests::qjl32_batch_with_blocks_and_tail_matches_pre_slice_scorer_bits -- --nocapture`
- `cargo test --release --lib --features bench quant::qjl32::tests::qjl32_block32_matches_pre_slice_scorer_bits -- --nocapture`
- `ECAZ_TQ_QJL_PROFILE_CANDIDATES=192000 ECAZ_TQ_QJL_PROFILE_LOG=reviews/task-124/031-tq-qjl-scorer-speed/artifacts/tq-qjl-direct-payload-rerun.log cargo test --release --lib --features bench task124_profile_tq_qjl_flush_widths -- --ignored --nocapture`

## Scope

Kept:

- Direct-payload QJL batch scorer cascade.
- QJL scorer-width profiler for future Task 124 QJL checks.

Task 124 remains open. Completed required levers so far on this reopened scorer-focused pass:

- Scoring kernel profile/attempt: packet 028.
- Per-query LUT/query-prep: packet 029.
- Batch/flush width: packet 030.
- QJL scoring speed: this packet.

Remaining required levers include dimension/subspace reduction, TQ2 with a real SIMD kernel, and payload prefetch/pipelining.

