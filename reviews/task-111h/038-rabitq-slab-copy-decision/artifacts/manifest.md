# Artifact Manifest

Task bucket: `reviews/task-111h/`
Packet: `reviews/task-111h/038-rabitq-slab-copy-decision/`
Head SHA: `0629a89409dd75a10e488b1767cb8d5e6b90004c`
Timestamp: `2026-06-20T15:03:56-07:00`
Branch: `bench-ivf-111g-115-attribution`

## Scope

This is a read-only Task 111h closeout packet for the copy/slab checklist row:

```text
Implement or explicitly benchmark away owned per-survivor payload copies and
double-copy batch-scoring slabs in the compact index path.
```

No code changed and no new runtime test or benchmark command was run for this
packet. The packet cites existing packet-local benchmarks and fixtures.

## Artifacts

| Artifact | Description |
| --- | --- |
| `rabitq-slab-copy-decision.md` | Source and benchmark audit explaining why f16/TurboQuant are implemented no-slab paths and why RaBitQ4/8 retain the contiguous arithmetic-estimator slab for Task 111h. |

## Commands

Read-only inspection commands used while preparing this packet:

```text
git rev-parse HEAD
sed -n '2580,2795p' src/am/ec_ivf/scan.rs
sed -n '2808,3195p' src/am/ec_ivf/scan.rs
sed -n '440,640p' src/am/ec_ivf/rerank.rs
sed -n '460,565p' src/am/ec_ivf/quantizer.rs
sed -n '1080,1210p' src/quant/rabitq.rs
sed -n '4320,4485p' src/quant/rabitq.rs
sed -n '350,405p' src/am/common/candidate_batch/mod.rs
sed -n '616,742p' src/am/common/candidate_batch/mod.rs
sed -n '1510,1645p' src/tests/ec_ivf.rs
sed -n '1,140p' reviews/task-106/001-m5-multibit-rabitq-bench/artifacts/manifest.md
sed -n '1,140p' reviews/task-106/002-intel-avx2-bench/artifacts/manifest.md
sed -n '1,120p' reviews/task-111h/032-turboquant-borrowed-rerank/artifacts/manifest.md
sed -n '1,130p' reviews/task-111h/036-rabitq8-score-clip-ab/artifacts/manifest.md
```

## Cited Packet Evidence

| Packet | Evidence used |
| --- | --- |
| `reviews/task-106/001-m5-multibit-rabitq-bench/` | M5 microbench and index-level routing evidence: bits=4 block dispatch slower than arithmetic estimator; bits=4 stays off block-kernel counters. |
| `reviews/task-106/002-intel-avx2-bench/` | Intel AVX2 microbench and suite evidence: bits=4 block dispatch slower than arithmetic estimator; bits=4/8 stay on estimator route. |
| `reviews/task-111h/030-counter-fixture-closeout-audit/` | Identifies the remaining RaBitQ slab copy and the PG18 counter fixture exposing it. |
| `reviews/task-111h/032-turboquant-borrowed-rerank/` | Implements and validates the TurboQuant borrowed no-slab path. |
| `reviews/task-111h/036-rabitq8-score-clip-ab/` | Measures the current contiguous RaBitQ8 estimator path after clip tuning. |

## Key Result Lines

- f16 and TurboQuant compact index rerank have zero batch slab copied bytes in
  the current PG18 counter fixture.
- RaBitQ4 records slab copied bytes equal to scored payload bytes, so the
  retained copy remains visible in EXPLAIN/debug counters.
- M5 bits=4 microbench: block dispatch median `12.853 us`; scalar/arithmetic
  estimate median `4.6153 us`.
- Intel bits=4 microbench: block dispatch median `72.900 us`;
  scalar/arithmetic estimate median `12.810 us`.
- RaBitQ8 estimator clip=4 reached recall@10 `0.9305` at nprobe32 and `0.9915`
  at nprobe200 in the current 100k Task 111h A/B.

## Non-Claims

This packet does not close the full Task 111h benchmark matrix, cold/remote
evidence, or final decision table. It only closes the copy/slab row by
documenting the implemented no-slab formats and the measured reason not to route
RaBitQ4/8 through the existing borrowed block-kernel path.
