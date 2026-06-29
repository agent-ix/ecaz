# Task 124 Packet 009: TQ Stage-2 Width Sweep

This is a TurboQuant-focused diagnostic benchmark packet. It is not Task 124 closeout.

## Summary

I tested whether reducing the full-TQ4 stage-2 frontier width can improve 100k latency while preserving the existing final exact f32 safety boundary.

The suite keeps the same in-engine pipeline under Task 124:

- coarse RaBitQ frontier
- index-side full 4-bit TurboQuant stage-2 rerank
- final exact f32 rerank at `stage2_final_rerank_width=15`

The result is useful but not promotable. Width 75 preserves the prior 100k full-TQ recall target, and the TQ scorer remains fully NEON/SIMD with zero scalar candidates, but the latency improvement is modest and storage remains roughly 4.7x the f32/source index baseline from packet 006.

## Code Commit

No code change in this packet.

Code under test:

- `676f924e4b377c833949285ed98f450e3cf9e464`

## Benchmark Evidence

Suite config:

- `reviews/task-124/009-tq-stage2-width-sweep/artifacts/task124-tq-stage2-width-100k-suite.json`

Suite command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/009-tq-stage2-width-sweep/artifacts/task124-tq-stage2-width-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/009-tq-stage2-width-sweep/artifacts/suite-run.log
```

Report check:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/009-tq-stage2-width-sweep/artifacts/stage2-width-100k/suite-manifest.json
```

Report summary: `completed 12`, `failed 0`, `skipped 0`.

Artifacts:

- `reviews/task-124/009-tq-stage2-width-sweep/artifacts/manifest.md`
- `reviews/task-124/009-tq-stage2-width-sweep/artifacts/suite-run.log`
- `reviews/task-124/009-tq-stage2-width-sweep/artifacts/stage2-width-100k/suite-manifest.json`
- `reviews/task-124/009-tq-stage2-width-sweep/artifacts/stage2-width-100k/results.jsonl`
- per-step load, recall, latency, and storage logs under `stage2-width-100k/`

Truth cache files are intentionally untracked.

## Results

Recall at k=10:

| Stage-2 width | nprobe=32 | nprobe=64 |
| ---: | ---: | ---: |
| 25 | 0.9530 | 0.9790 |
| 50 | 0.9710 | 0.9980 |
| 75 | 0.9730 | 1.0000 |

Latency:

| Stage-2 width | nprobe | p50 | p95 | p99 |
| ---: | ---: | ---: | ---: | ---: |
| 25 | 32 | 4.58 ms | 5.10 ms | 5.43 ms |
| 25 | 64 | 8.59 ms | 8.86 ms | 9.20 ms |
| 50 | 32 | 4.78 ms | 5.32 ms | 5.60 ms |
| 50 | 64 | 8.97 ms | 10.5 ms | 10.9 ms |
| 75 | 32 | 4.86 ms | 5.46 ms | 5.63 ms |
| 75 | 64 | 9.07 ms | 9.41 ms | 9.72 ms |

Storage:

| Stage-2 width | ec_ivf index size | per row |
| ---: | ---: | ---: |
| 25 | 116.3 MiB | 1219.1 B |
| 50 | 100.8 MiB | 1057.2 B |
| 75 | 105.9 MiB | 1110.0 B |

TQ scorer counters:

| Stage-2 width | nprobe | quant | isa | scalar_candidates | TQ candidates |
| ---: | ---: | --- | --- | ---: | ---: |
| 25 | 32 | turboquant | neon | 0 | 2500 |
| 25 | 64 | turboquant | neon | 0 | 2500 |
| 50 | 32 | turboquant | neon | 0 | 5000 |
| 50 | 64 | turboquant | neon | 0 | 5000 |
| 75 | 32 | turboquant | neon | 0 | 7500 |
| 75 | 64 | turboquant | neon | 0 | 7500 |

## Interpretation

This rules out a remaining scalar-dispatch explanation for full TQ4 stage-2 latency on this lane: the scorer is already SIMD for widths 25/50/75, and prior packets showed width 100 is also SIMD.

Width 75 is the only tested frontier reduction that preserves the packet 006 100k recall target of `0.9730 / 1.0000`. It improves nprobe32 p99 versus packet 006 TQ final15 (`5.63 ms` here versus `6.11 ms`) and is essentially neutral at nprobe64 p99 (`9.72 ms` here versus `9.71 ms`). That is not enough to make TQ competitive because storage remains `105.9 MiB` at 100k versus the packet 006 f32/source baseline of `22.5 MiB`.

The surprising storage result is that lower runtime frontier width does not monotonically reduce index bytes: width 25 is worse than width 50, and width 75 is still larger than width 50. This means `rerank_width` is not a clean latency-only runtime knob for TQ stage-2. It affects the persisted sidecar grouping/layout enough that the next useful implementation should target payload layout/materialization directly instead of only shrinking the stage-2 frontier.

## Decision

Do not promote width reduction as the Task 124 solution.

Task 124 remains open. The next TQ-focused implementation work should preserve the full TQ4 scoring surface and attack storage/locality overhead: decouple runtime stage-2 frontier from persisted TQ sidecar grouping, reduce per-group header/layout overhead, or fuse TQ score/top-k/materialization so fewer large sidecar payloads have to be touched.
