# Task 103 Packet 003 Artifact Manifest

- head SHA: `f271717aa` (test-lock fix; backend code identical to
  `248472ea2` — the commit under this packet changes test code only)
- task bucket: `reviews/task-103/`
- packet path: `reviews/task-103/003-rabitq32-avx2-validation/`
- timestamp: 2026-06-10
- lane: local PG18 / Intel AVX2
- fixture: `task103_diskann_rabitq_2k` — synthetic 2k × 1536d corpus +
  64 queries created by this suite (`corpus generate` seeds 10301/10302,
  `corpus load --profile ec_diskann --storage-format rabitq`,
  DiskANN rabitq sidecar is bits=1 by construction,
  `DISKANN_RABITQ_BITS = 1`). Own prefix, one index per table.
  The pre-existing local IVF rabitq fixtures could not serve this
  packet: they use the default `rabitq_bits=4`, and the rabitq32 block
  kernel only serves the bits=1 lane.
- backend provenance: same release install as packet 002
  (`248472ea2`, SHA
  `c1875a84304d7124b32a7aaff98c4961cc59045b68a328fb5bbb29df3e69639c`);
  no backend code changed and no pg_tests ran since that install
  (every cargo-test filter below matches zero `#[pg_test]` names).
  `build-profile-probe.log` → `release`.

## Suites

`task103-diskann-rabitq-suite.json` (7 steps, `ecaz bench suite`):
generate corpus/queries, load, recall kernel-on / kernel-off
(`ec_diskann.candidate_batch_scoring=off`), latency kernel-off then
kernel-on at `list_size=64,128` with
`--task87-candidate-batch-counters`. `suite-run.log`,
`suite-audit.log` (passed: 7), `suite-status.log` (7 succeeded),
`results.jsonl`, `results-report.jsonl`, `suite-manifest.json`.

## Key result lines

### AC4: rabitq32 AVX2 counter attribution

| Cell | candidates | kernel ns/c | isa | scalar_candidates |
| --- | ---: | ---: | --- | ---: |
| list=64 batch-on | 261,126 | 81.1 | avx2 | 0 |
| list=128 batch-on | 348,227 | 80.4 | avx2 | 0 |

Width histograms (list=128): <8 = 7,526; 8–15 = 8,964; 16–31 = 9,133;
≥32 = 209 — the DiskANN neighbor-batch shape exercises the AVX2
partial path across the whole sub-block range, plus full 32-wide
blocks.

### Recall (kernel-on vs kernel-off)

| Cell | recall@k | ndcg@k | p50 / worst |
| --- | ---: | ---: | --- |
| batch-on (avx2) | 0.5984 | 0.9541 | 0.6000 / 0.3000 |
| batch-off (per-candidate) | 0.5984 | 0.9541 | 0.6000 / 0.3000 |

Byte-equal. (The rabitq32 family contract is ADR-076 tolerance, not
bit-equality, for SIMD-vs-scalar — byte-equal bench recall exceeds the
required gate.)

### End-to-end (200 iterations, concurrency 1)

| Cell | list=64 p50 / p95 | list=128 p50 / p95 |
| --- | --- | --- |
| batch-on (avx2) | 3.84 / 4.20 ms | 4.07 / 4.43 ms |
| batch-off | 3.83 / 4.08 ms | 4.09 / 4.45 ms |

Within noise both directions — scoring is a small share of DiskANN
query time at this scale; the cell exists to satisfy "no end-to-end
regression beyond noise" (AC5), not to claim a win.

## Test and lint artifacts

- `cargo-test-rabitq32.log`: 6 passed on this AVX2+FMA host —
  `host_expected_simd_isa()` resolves to Avx2, so
  `dispatched_block32_matches_anchor_within_tolerance`,
  `simd_block32_is_bit_equal_with_production_batch`, and
  `partial_dispatch_matches_anchor_and_production_batch` exercise and
  assert the AVX2 backend (the Task 93 "when-available" suite, now run
  on Intel).
- `cargo-test-rabitq-bits1-dispatch.log`: 5 passed — AM-level dispatch
  routes through the block kernel and matches it bit-exactly
  (`am::ec_ivf::quantizer` dispatch tests + `candidate_batch`
  counter tests). Required the test-lock fix in `f271717aa`
  (pre-existing isolation gap, see commit message).
- `cargo-clippy.log`: `--all-targets -D warnings` clean.
