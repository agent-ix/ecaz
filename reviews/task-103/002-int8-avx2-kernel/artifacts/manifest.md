# Task 103 Packet 002 Artifact Manifest

- head SHA: `248472ea2` (Add int8_approx32 AVX2 kernel — Task 103 AC1)
- task bucket: `reviews/task-103/`
- packet path: `reviews/task-103/002-int8-avx2-kernel/`
- timestamp: 2026-06-10
- lane: local PG18 / Intel AVX2
- fixture: `task87_phase6_real10k_hnsw` (real 10k, 1536d,
  `storage_format=turboquant`, exact-mode GUC sweep) — existing
  one-index-per-table fixture; nothing created
- backend provenance: fresh release install at this head —
  `install-ecaz-pg18.log` (`ecaz dev install ecaz-pg-test --pg 18`,
  artifact assertion passed, SHA
  `c1875a84304d7124b32a7aaff98c4961cc59045b68a328fb5bbb29df3e69639c`),
  postmaster restarted 21:34, `build-profile-probe.log` → `release`.
  No pg_tests ran between install and the suite (the focused
  cargo-test logs below were captured after the suite completed and
  match zero `#[pg_test]` names).

## Suite

`task103-int8-avx2-suite.json` (4 steps, run via `ecaz bench suite`):
recall batch-on / batch-off (byte-equality check), latency batch-off
then batch-on at `ef_search=80,160` with
`--task87-candidate-batch-counters`. `suite-run.log`,
`suite-audit.log` (passed: 4), `suite-status.log` (4 succeeded),
`results.jsonl`, `results-report.jsonl`, `suite-manifest.json`.

## Key result lines

### AC1 gate: per-candidate rate (counters, `quant=turboquant_int8`)

| Cell | candidates | kernel ns total | ns/candidate | isa | scalar_candidates |
| --- | ---: | ---: | ---: | --- | ---: |
| ef=80 batch-on | 268,446 | 23,795,530 | **88.6** | avx2 | 0 |
| ef=160 batch-on | 417,687 | 37,005,872 | **88.6** | avx2 | 0 |

Scalar anchor (packet 001, same fixture/cells, scalar path untouched by
this commit): 918.7–923.0 ns/c → **10.4× the anchor** (gate: ≥2×
target, 1.5× floor). The rate is flat across ef levels and across the
narrow-width distribution (ef=160 histogram: width<8 = 176,820 flushes
of 184,755; 8–15 = 6,111; 16–31 = 1,713; ≥32 = 111) — the kernel is
per-candidate dim-parallel, so flush width does not affect its rate.

### Recall byte-equality (AC5)

| Cell | recall@k | ndcg@k | p10/p50/p90/worst |
| --- | ---: | ---: | --- |
| batch-on (avx2) | 0.6230 | 0.9319 | 0.0000 / 0.7500 / 1.0000 / 0.0000 |
| batch-off | 0.6230 | 0.9319 | 0.0000 / 0.7500 / 1.0000 / 0.0000 |

Byte-equal, as required by the family's integer-exact contract.

### End-to-end (300 iterations, concurrency 1)

| Cell | ef=80 p50 / p95 | ef=160 p50 / p95 |
| --- | --- | --- |
| batch-on (avx2 kernel) | **3.63 / 5.37 ms** | **5.37 / 7.71 ms** |
| batch-off (per-candidate scalar) | 4.14 / 6.21 ms | 6.34 / 8.78 ms |

Batch-on now beats batch-off by 12–15% p50 (packet 001 had batch-on
*losing* 4.52 vs 4.19 ms with the scalar block path). Versus packet
001's batch-on cells: 4.52 → 3.63 ms (−19.7%) and 6.62 → 5.37 ms
(−18.9%). int8_approx is now the fastest measured HNSW exact mode on
this host (full_lut packet 001: 4.52 / 6.76 ms).

## Test and lint artifacts

- `cargo-test-int8-approx32.log`: 4 passed (parity incl. the ±128
  extreme-values corner and synthetic dim-tails 7/64/100/191; strict
  `to_bits()` equality on this AVX2 host).
- `cargo-test-candidate-batch.log`: 19 passed.
- `cargo-clippy.log`: `--all-targets -D warnings` clean.
