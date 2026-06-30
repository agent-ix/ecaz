# Task 124 Packet 026: f32 vs TQ nprobe60 discriminator

## Summary

This packet answers the reviewer discriminator from packet 023:

> Does TQ stage-2 + exact rerank let `ec_ivf` use `nprobe=60` at equal recall
> where the f32/source baseline cannot?

Result: **No.** In this same-fixture 10k / 50k / 100k run, f32/source at
`nprobe=60` preserves recall at all scales. TQ at `nprobe=60` is faster in this
run at 10k, 50k, and 100k, but 50k TQ recall is lower than f32
(`0.9980` vs `1.0000`) and TQ storage remains about 4.5x the f32/source index
at 100k.

This means the earlier `nprobe60` observation must not be claimed as a
TQ-attributable speed win. It is a frontier operating point that f32 can also
use.

## Validation

- `cargo build --release -p ecaz`: passed
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`: passed
- `ecaz bench suite audit`: passed, 24 steps
- `ecaz bench suite run`: completed, 24 succeeded / 0 failed
- `ecaz bench suite status`: completed, 24 succeeded / 0 failed
- `ecaz bench suite report`: generated

Artifact source of truth:

- `artifacts/manifest.md`
- `artifacts/task124-f32-vs-tq-nprobe60-10-50-100-suite.json`
- `artifacts/suite-audit-r2.log`
- `artifacts/suite-run-r2.log`
- `artifacts/suite-manifest-r2.json`
- `artifacts/results-r2.jsonl`
- `artifacts/suite-status-r2.log`
- `artifacts/suite-report-r2.log`
- `artifacts/report-results-r2.jsonl`
- packet-local recall, latency, and storage logs under
  `artifacts/f32-vs-tq-nprobe60-r2/`

## Result

Both variants use `nprobe=60` with coarse RaBitQ 1-bit. The f32/source baseline
uses `rerank_placement=source`, `rerank_format=f32`, `rerank_width=100`,
`stage2_final_rerank_width=0`. TQ uses `rerank_placement=index`,
`rerank_format=turboquant`, `rerank_width=75`, `rerank_group_width=50`,
`stage2_final_rerank_width=15`.

| Scale | Variant | Recall@10 | NDCG@10 | p50 | p95 | p99 | ec_ivf index |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 1.0000 | 1.0000 | 1.22 ms | 1.38 ms | 1.43 ms | 2.9 MiB |
| 10k | TQ final15 | 1.0000 | 1.0000 | 1.13 ms | 1.28 ms | 1.37 ms | 10.9 MiB |
| 50k | f32/source | 1.0000 | 1.0000 | 4.48 ms | 5.32 ms | 5.83 ms | 11.6 MiB |
| 50k | TQ final15 | 0.9980 | 1.0000 | 4.23 ms | 4.47 ms | 4.54 ms | 50.9 MiB |
| 100k | f32/source | 1.0000 | 1.0000 | 9.46 ms | 9.76 ms | 9.92 ms | 22.5 MiB |
| 100k | TQ final15 | 1.0000 | 1.0000 | 8.77 ms | 9.01 ms | 9.22 ms | 100.8 MiB |

TQ scorer counters remain fully SIMD:

| Scale | TQ candidates | TQ scalar candidates | TQ elapsed | TQ ISA |
| --- | ---: | ---: | ---: | --- |
| 10k | 7,500 | 0 | 1.811008 ms | neon |
| 50k | 7,500 | 0 | 1.851708 ms | neon |
| 100k | 7,500 | 0 | 1.907458 ms | neon |

The shared coarse frontier work is the same order for f32 and TQ at the same
scale and `nprobe=60`:

| Scale | Variant | Coarse candidates |
| --- | --- | ---: |
| 10k | f32/source | 936,366 |
| 10k | TQ final15 | 936,366 |
| 50k | f32/source | 4,525,933 |
| 50k | TQ final15 | 4,525,933 |
| 100k | f32/source | 9,556,278 |
| 100k | TQ final15 | 9,556,278 |

## Decision

This discriminator is negative for the nprobe-based TQ claim. f32/source also
holds recall at `nprobe=60`, so the reviewer-requested condition for a
TQ-attributable frontier win is not met.

The TQ stage-2 path is still a real implementation and remains fully NEON/SIMD,
but Task 124 should not close as "TQ speed improved" from the `nprobe60` knob.
At this point the durable finding is that the remaining TQ-component speed
levers measured so far are exhausted or negative, while the TQ sidecar still
costs materially more storage than f32/source.
