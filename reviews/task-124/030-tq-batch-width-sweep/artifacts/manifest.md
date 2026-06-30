# Task 124 / 030 TQ Batch Width Sweep Manifest

- Head SHA: `d648f88b24cf55603246ea859cec82c633e6982a`
- Task bucket: `reviews/task-124/030-tq-batch-width-sweep`
- Lane: TurboQuant no-QJL 4-bit batch scorer / flush-width microprofile
- Fixture: `ProdQuantizer::new(1536, 4, 42)`, 1536D unit query, NEON backend, shared `score_turboquant_no_qjl_4bit_batch_for` IVF surface
- Storage / rerank mode: not applicable; this is TQ-internal scorer timing, not end-to-end IVF latency
- Measurement command:
  - `ECAZ_TQ_BATCH_WIDTH_PROFILE_CANDIDATES=256000 ECAZ_TQ_BATCH_WIDTH_PROFILE_LOG=<artifact> cargo test --release --lib --features bench task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture`
- Validation commands:
  - `cargo fmt --check`
  - `cargo test --release --lib --features bench am::common::candidate_batch::tests::turboquant_lut_batch_matches_scalar_tail -- --nocapture`
  - `cargo test --release --lib --features bench am::common::candidate_batch::tests::turboquant_lut_batch_records_surface_counters -- --nocapture`
  - `cargo test --release --lib --features bench am::ec_ivf::quantizer::tests::turboquant_no_qjl_4bit_batch_scores_match_scalar_scores -- --nocapture`
- Timestamp: 2026-06-30
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `tq-batch-width-sweep.log`

Initial baseline run before direct-payload scoring. Kept for provenance; release-rebuild-adjacent timings were noisier than the immediate rerun.

### `tq-batch-width-sweep-rerun.log`

Primary baseline hot rerun before direct-payload scoring.

Key lines:

```text
task124_tq_batch_width_profile backend=neon dim=1536 total_candidates=256000
width=32 iterations=8000 candidates=256000 total=65.043625ms ns_per_candidate=254.1
width=64 iterations=4000 candidates=256000 total=67.915125ms ns_per_candidate=265.3
width=75 iterations=3413 candidates=255975 total=72.295417ms ns_per_candidate=282.4
width=96 iterations=2666 candidates=255936 total=68.22625ms ns_per_candidate=266.6
width=100 iterations=2560 candidates=256000 total=70.506208ms ns_per_candidate=275.4
width=128 iterations=2000 candidates=256000 total=67.707792ms ns_per_candidate=264.5
width=256 iterations=1000 candidates=256000 total=68.200375ms ns_per_candidate=266.4
```

### `tq-batch-width-direct-payload.log`

Initial post-change run after direct-payload scoring. Kept for provenance; release-rebuild-adjacent timings were noisier than the immediate rerun.

### `tq-batch-width-direct-payload-rerun.log`

Primary post-change hot rerun after direct-payload scoring.

Key lines:

```text
task124_tq_batch_width_profile backend=neon dim=1536 total_candidates=256000
width=32 iterations=8000 candidates=256000 total=60.522625ms ns_per_candidate=236.4
width=64 iterations=4000 candidates=256000 total=62.919042ms ns_per_candidate=245.8
width=75 iterations=3413 candidates=255975 total=69.390042ms ns_per_candidate=271.1
width=96 iterations=2666 candidates=255936 total=65.246541ms ns_per_candidate=254.9
width=100 iterations=2560 candidates=256000 total=67.757292ms ns_per_candidate=264.7
width=128 iterations=2000 candidates=256000 total=65.210292ms ns_per_candidate=254.7
width=256 iterations=1000 candidates=256000 total=65.995791ms ns_per_candidate=257.8
```

## Primary Delta

Using the immediate hot reruns for a like-for-like comparison:

| Width | Before ns/candidate | After ns/candidate | Delta |
| --- | ---: | ---: | ---: |
| 32 | 254.1 | 236.4 | -7.0% |
| 64 | 265.3 | 245.8 | -7.4% |
| 75 | 282.4 | 271.1 | -4.0% |
| 96 | 266.6 | 254.9 | -4.4% |
| 100 | 275.4 | 264.7 | -3.9% |
| 128 | 264.5 | 254.7 | -3.7% |
| 256 | 266.4 | 257.8 | -3.2% |

The sweep also confirms the low-level shape: widths aligned to the 32-candidate kernel block are cheapest, while widths that force partial tails cost more. This packet keeps the direct-payload scorer improvement and leaves any recall-sensitive stage-2 width policy change for a separate benchmarked slice.

