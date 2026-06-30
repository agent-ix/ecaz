# Task 124 / 031 TQ QJL Scorer Speed Manifest

- Head SHA: `3058c38b607257102e4ff2aa286206837a1b8225`
- Task bucket: `reviews/task-124/031-tq-qjl-scorer-speed`
- Lane: TurboQuant QJL-active 4-bit batch scorer microprofile
- Fixture: `ProdQuantizer::new(1024, 4, 42)`, 1024D unit query, NEON backend, shared `score_turboquant_qjl_batch_for` IVF surface
- Storage / rerank mode: not applicable; this is TQ-internal QJL scorer timing, not end-to-end IVF latency
- Measurement command:
  - `ECAZ_TQ_QJL_PROFILE_CANDIDATES=192000 ECAZ_TQ_QJL_PROFILE_LOG=<artifact> cargo test --release --lib --features bench task124_profile_tq_qjl_flush_widths -- --ignored --nocapture`
- Validation commands:
  - `cargo fmt --check`
  - `cargo test --release --lib --features bench am::ec_ivf::quantizer::tests::common_quant_codec_turboquant_batch_is_bit_exact_with_scalar -- --nocapture`
  - `cargo test --release --lib --features bench quant::qjl32::tests::qjl32_batch_with_blocks_and_tail_matches_pre_slice_scorer_bits -- --nocapture`
  - `cargo test --release --lib --features bench quant::qjl32::tests::qjl32_block32_matches_pre_slice_scorer_bits -- --nocapture`
- Timestamp: 2026-06-30
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `tq-qjl-baseline.log`

Initial QJL scorer-width baseline after release rebuild. Kept for provenance; hot rerun is the primary comparison.

### `tq-qjl-baseline-rerun.log`

Primary baseline hot rerun before direct-payload QJL scoring.

Key lines:

```text
task124_tq_qjl_profile backend=neon dim=1024 total_candidates=192000
width=8 iterations=24000 candidates=192000 total=34.976416ms ns_per_candidate=182.2
width=16 iterations=12000 candidates=192000 total=34.892208ms ns_per_candidate=181.7
width=25 iterations=7680 candidates=192000 total=38.533916ms ns_per_candidate=200.7
width=32 iterations=6000 candidates=192000 total=34.410417ms ns_per_candidate=179.2
width=64 iterations=3000 candidates=192000 total=34.278542ms ns_per_candidate=178.5
width=96 iterations=2000 candidates=192000 total=34.198458ms ns_per_candidate=178.1
width=100 iterations=1920 candidates=192000 total=37.597834ms ns_per_candidate=195.8
width=128 iterations=1500 candidates=192000 total=34.116458ms ns_per_candidate=177.7
```

### `tq-qjl-direct-payload.log`

Initial post-change run after direct-payload QJL scoring. Kept for provenance; hot rerun is the primary comparison.

### `tq-qjl-direct-payload-rerun.log`

Primary post-change hot rerun after direct-payload QJL scoring.

Key lines:

```text
task124_tq_qjl_profile backend=neon dim=1024 total_candidates=192000
width=8 iterations=24000 candidates=192000 total=34.340375ms ns_per_candidate=178.9
width=16 iterations=12000 candidates=192000 total=34.32575ms ns_per_candidate=178.8
width=25 iterations=7680 candidates=192000 total=37.828375ms ns_per_candidate=197.0
width=32 iterations=6000 candidates=192000 total=33.822583ms ns_per_candidate=176.2
width=64 iterations=3000 candidates=192000 total=33.796375ms ns_per_candidate=176.0
width=96 iterations=2000 candidates=192000 total=33.862334ms ns_per_candidate=176.4
width=100 iterations=1920 candidates=192000 total=37.245208ms ns_per_candidate=194.0
width=128 iterations=1500 candidates=192000 total=33.775ms ns_per_candidate=175.9
```

## Primary Delta

Using the immediate hot reruns for a like-for-like comparison:

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

The sweep confirms the QJL scorer is already NEON-blocked for block/octet widths; this slice removes wrapper allocation/materialization around that scorer and keeps the speedup.

