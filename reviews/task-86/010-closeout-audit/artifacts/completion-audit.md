# Task 86 Completion Audit

Generated: `2026-06-07T19:35:00Z`

Branch: `task-86-turbovec-turboquant-improvements`

Head audited: `d6462c594210e60e15fd9bb6b46f1f82508ee82f` plus packet 001
follow-up report edits in this packet.

Base audited: `origin/main` at
`71e16fcdced96714e7db1dd98f396cd68941180e`.

## Requirement Coverage

| Task 86 requirement | Evidence | Status |
| --- | --- | --- |
| Explain TurboVec "query in same space" claim | `reviews/task-86/001-turbovec-tq-analysis/artifacts/turbovec-tq-analysis.md` says TurboVec rotates and inverse-calibrates the query, then builds LUTs; it does not pack queries into database code bytes. Packet 008 repeats the same conclusion after re-audit. | Covered |
| Identify new encoding/query/scorer choices | Packet 001 identifies TQ+ shift/scale, dense rotation, per-vector renorm, `u8` LUTs, 32-vector blocks, fused top-k, and multi-query scoring. | Covered |
| Compare vector size | Packet 001 now includes 768/1536/3072 rows at 2-bit and 4-bit for both TurboVec and our TQ payload semantics. Packet 008 records the accepted SPIRE code slice has unchanged storage. | Covered |
| Compare SIMD kernels | Packet 001 compares TurboVec 32-vector block scan, `u8` LUTs, filter skips, fused top-k, and 4-query lanes against our one-payload-at-a-time shared TQ scorer and AVX2/NEON/QJL paths. | Covered |
| Identify index type | Packet 001 identifies TurboVec as a flat positional compressed-vector scan index plus optional IdMap wrapper, not HNSW, DiskANN, IVF, or SPIRE. | Covered |
| Explain transfer to our AMs | Packet 001 and packet 008 state kernel ideas can transfer, but flat-index end-to-end claims do not transfer automatically to HNSW, DiskANN, IVF, or SPIRE. Packet 008 measures the SPIRE-specific transfer. | Covered |
| Use our TurboQuant baseline only | All Task 86 packets use TurboQuant-only comparisons except incidental source context. Packet 008 compares pre-LUT SPIRE TurboQuant vs post-LUT SPIRE TurboQuant. | Covered |
| Use `ecaz bench suite` for benchmark matrices | Packet 008 uses `suite-lutoff.json` and `suite-luton.json`, both audited with `ecaz bench suite audit` and run with `ecaz bench suite run`. | Covered |
| Record p50/p95/p99 latency | Packet 008 now summarizes SQL and pipeline p50/p95/p99 in `artifacts/benchmark-delta.md`, with raw JSONL rows in `artifacts/lutoff/results-report.jsonl` and `artifacts/luton/results-report.jsonl`. | Covered |
| Benchmark real 10/50/100 spread | Packet 008 uses real10k, real50k, and real100k DBPedia corpora with low/medium/high probe settings for each corpus. | Covered |
| Test SPIRE with TurboQuant | Packet 008 profile is `ec_spire`, storage format `turboquant`, bits `4`, seed `42`. | Covered |

## Candidate Outcomes

Accepted production candidate:

- SPIRE no-QJL 4-bit TurboQuant dimension-LUT scoring.
- Source slice: `src/am/ec_spire/quantizer/mod.rs` and
  `src/am/ec_spire/quantizer/tests.rs`.
- Validation: packet 009 focused PG18-feature Rust test passes.
- Benchmark: packet 008 real-corpus before/after suite shows unchanged recall
  and storage with faster SQL mean and pipeline p50 at all nine sweep points.

Shelved or follow-up candidates:

- TQ+ calibration-only: packet 002 and packet 003 are probe evidence only. They
  do not justify production without a real-corpus recall/latency/storage suite.
- TurboVec renormalization scalar: packet 003 did not show normalized-IP quality
  improvement. Blocker: quality, and possible extra 4 bytes/vector if persisted.
- Byte-pair LUT: packet 004 showed slower scoring than the dimension LUT on the
  tested lane. Blocker: latency and query-LUT footprint.
- 32-vector blocked slabs: source-grounded option, not implemented. Blocker:
  AM transfer complexity; flat contiguous slabs do not map directly onto graph
  traversal or page-bounded scans.
- Dense rotation: source-grounded option, not implemented. Blocker: query-prep
  cost and need for a separate SRHT/FWHT quality probe.
- Fused top-k / multi-query scoring: source-grounded option, not implemented.
  Blocker: API and workload fit; likely useful only after a contiguous-candidate
  lane proves enough scorer headroom remains.

## Benchmark Evidence

Packet 008 key result:

- Recall: unchanged at all nine real-corpus sweep points.
- Storage: unchanged at all reported precisions.
- SQL mean latency:
  - real10k: `3.44 -> 3.30 ms`, `8.02 -> 7.69 ms`, `10.2 -> 9.74 ms`
  - real50k: `12.3 -> 12.0 ms`, `33.9 -> 32.9 ms`, `48.0 -> 46.1 ms`
  - real100k: `25.3 -> 24.5 ms`, `74.1 -> 71.7 ms`, `95.3 -> 92.3 ms`
- Pipeline p50:
  - real10k: `3.549 -> 3.406 ms`, `8.089 -> 7.675 ms`, `10.283 -> 9.711 ms`
  - real50k: `12.660 -> 11.938 ms`, `33.779 -> 32.299 ms`, `48.192 -> 46.042 ms`
  - real100k: `25.646 -> 24.670 ms`, `74.584 -> 72.274 ms`, `95.084 -> 92.184 ms`
- SQL tail latency improves at every sweep point:
  - real10k p95: `3.78 -> 3.57 ms`, `8.39 -> 8.09 ms`, `10.4 -> 9.96 ms`
  - real50k p95: `13.7 -> 13.3 ms`, `36.3 -> 35.4 ms`, `48.5 -> 46.6 ms`
  - real100k p95: `27.6 -> 26.7 ms`, `77.0 -> 74.4 ms`, `96.0 -> 93.0 ms`
- Pipeline tail latency improves at every sweep point:
  - real10k p99: `3.907 -> 3.732 ms`, `8.649 -> 8.206 ms`, `10.844 -> 9.997 ms`
  - real50k p99: `14.169 -> 13.367 ms`, `37.368 -> 35.757 ms`, `50.154 -> 47.437 ms`
  - real100k p99: `29.383 -> 28.483 ms`, `79.099 -> 76.258 ms`, `100.015 -> 93.718 ms`

Packet 008 is the lifting evidence for packet 005 seq 02 and packet 007 seq 01:
it uses real data, a baseline source install, current source install, and
packet-local `ecaz bench suite` artifacts.

## Review Feedback Status

- Packet 001: approved. Follow-up gaps are addressed by the updated report:
  methodology, corrected `prod.rs` citations, byte table, renormalization
  derivation, and "not learnable from analysis alone" bounds.
- Packet 002: conditionally approved as a probe only. Not promoted.
- Packet 003: approved as a clean isolation probe. Not promoted.
- Packet 004: approved as a negative-result probe. Not promoted.
- Packet 005: request-changes due to missing real-corpus benchmark evidence.
  Packet 008 supplies the requested real 10/50/100 baseline-vs-current suite.
- Packet 006: approved with tightening request. The options report now states
  which evidence is measured and which ideas remain unproven.
- Packet 007: blocked because synthetic-only evidence could not validate the
  production optimization. Packet 008 supplies the real-corpus suite.
- Packet 009: approved and closes the LUT parity concern.

Packet 008 has not yet received outside reviewer feedback, so the coder-side
status is "ready for review" rather than "reviewer accepted."

## Scope / Safety Checks

Commands run:

```text
git fetch origin main
git diff --name-only origin/main...HEAD -- src
git diff origin/main...HEAD -- src | rg -n "unsafe"
```

Source files changed under `src/`:

```text
src/am/ec_spire/quantizer/mod.rs
src/am/ec_spire/quantizer/tests.rs
src/lib.rs
src/quant/prod.rs
```

The `unsafe` search over the source diff returned no matches.

Scope assessment:

- Production behavior change is SPIRE TurboQuant no-QJL 4-bit LUT scoring.
- TQ+ calibration, renormalization, and byte-LUT code is behind
  `#[cfg(any(test, feature = "bench"))]` and exists to support Task 86 probes.
- No durable on-disk format change is included.
- No SQL-visible storage contract change is included.

## Residual Risk

- Packet 008 does not expose scorer-only or query-prep-only timings because the
  current suite surface does not report those fields for the SPIRE AM path.
  SQL and pipeline latency are measured; scorer/query-prep isolation should be
  a follow-up suite-runner extension before broader AM rollout.
- SPIRE evidence does not prove transfer to HNSW, DiskANN, or IVF. The accepted
  code change is intentionally scoped to SPIRE.
- TQ+ remains an investigation candidate, not a production improvement.
