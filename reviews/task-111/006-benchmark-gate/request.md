# Task 111 Packet 006: Benchmark Gate

## Scope

This packet is the Phase 5 benchmark gate for Task 111. It compares gated dense posting blocks against the legacy row posting layout for local real 100k IVF cells:

- TurboQuant row vs dense.
- RaBitQ `quant_bits=1` row vs dense.
- Warm latency, recall, build time, index storage, EXPLAIN scan counters, and batch scorer counters.

The run used `ecaz bench suite` only. The local checkout had the real 100k corpus available; no local 1M manifest was present under `data/`, so the packet does not claim 1M coverage.

## Evidence

- Artifact manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task111-dense-posting-suite.json`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Parsed report: `artifacts/suite/results-report.jsonl`
- Raw structured results: `artifacts/suite/results.jsonl`
- Install/setup logs: `artifacts/install-ecaz-pg18-release.log`, `artifacts/create-task111-clean-bench-db.log`, `artifacts/create-extension-task111-clean-bench.log`
- Per-step logs: `artifacts/suite/*.log` and EXPLAIN SQL files under `artifacts/suite/`

The suite completed 20/20 steps with 0 failed, 0 skipped, 0 missing artifacts, and 0 stale artifacts. The committed packet excludes the generated recall truth cache per repository packet policy.

## Results Summary

Recall was unchanged in every compared cell:

| Cell | nprobe | row recall@10 | dense recall@10 | row NDCG@10 | dense NDCG@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| TurboQuant | 16 | 0.8980 | 0.8980 | 0.9915 | 0.9915 |
| TurboQuant | 32 | 0.9370 | 0.9370 | 0.9966 | 0.9966 |
| RaBitQ | 16 | 0.7490 | 0.7490 | 0.9826 | 0.9826 |
| RaBitQ | 32 | 0.7630 | 0.7630 | 0.9875 | 0.9875 |

Warm latency:

| Cell | nprobe | row p50/p95/p99 | dense p50/p95/p99 | Outcome |
| --- | ---: | --- | --- | --- |
| TurboQuant | 16 | 16.5 / 20.3 / 26.3 ms | 19.9 / 23.7 / 25.4 ms | p50/p95 regression |
| TurboQuant | 32 | 31.7 / 42.3 / 52.0 ms | 39.2 / 45.7 / 48.2 ms | p50/p95 regression |
| RaBitQ | 16 | 7.68 / 10.1 / 11.4 ms | 6.65 / 8.17 / 9.55 ms | improvement |
| RaBitQ | 32 | 14.4 / 16.5 / 19.6 ms | 12.3 / 13.9 / 14.1 ms | improvement |

Build time and storage:

| Cell | row build | dense build | row index | dense index |
| --- | ---: | ---: | ---: | ---: |
| TurboQuant | 7.05 s | 7.38 s | 87.6 MiB | 78.9 MiB |
| RaBitQ | 6.81 s | 6.77 s | 29.7 MiB | 22.5 MiB |

EXPLAIN counters at `nprobe=32` show the expected scan-shape change:

| Cell | row pages | dense pages | row scratch bytes | dense scratch bytes |
| --- | ---: | ---: | ---: | ---: |
| TurboQuant | 4700 | 4233 | 32387328 payload + 253026 heap TID | 0 |
| RaBitQ | 1577 | 1189 | 8602884 payload + 253026 heap TID | 0 |

The latency logs explain the TurboQuant regression: dense TurboQuant scored approximately the same candidate counts but created many more small SIMD scorer flushes. At `nprobe=32`, row TurboQuant used 20379 SIMD flushes with most widths `>=32`; dense TurboQuant used 521755 SIMD flushes, mostly width `8..15`.

## Recommendation

Iterate, do not promote dense posting blocks as the default yet.

Dense blocks satisfy the correctness and storage parts of the gate: recall is unchanged, row-format compatibility remains covered by prior packets, dense blocks reduce index size, and EXPLAIN counters show row-posting scratch copies are eliminated. The full promotion criterion is not met because TurboQuant dense regressed p50 and p95 at both measured nprobe cells. RaBitQ improved, but Task 111 scoped both active compact IVF formats and the default should not move while TurboQuant regresses.

Recommended next slice: keep `dense_posting_blocks` gated/off by default and work on dense block packing or scan-side coalescing so TurboQuant dense can feed the batch scorer in larger widths before reconsidering promotion. Also keep the Task 42 note active before any durable page-format promotion: this experimental `0x25` dense layout changed during the task, so packet-002-era dev indexes must be rebuilt.
