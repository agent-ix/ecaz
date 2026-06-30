# Task 124 Packet 035: post-scorer product suite

## Summary

This packet reruns the Task 124 10k / 50k / 100k product matrix on current
HEAD after the reopened TurboQuant scorer-path work in packets 028-034.

The reopened scorer phase is now complete:

| Lever | Evidence |
| --- | --- |
| TQ scoring kernel itself | `028-tq-scorer-kernel-profile` |
| Per-query LUT / query-prep | `029-tq-query-prep-lut16` |
| Batch/flush width | `030-tq-batch-width-sweep` |
| QJL scoring speed | `031-tq-qjl-scorer-speed` |
| TQ2 real SIMD kernel | `032-tq2-simd-scorer` |
| Prefetch / pipelining | `033-tq-prefetch-pipeline` |
| Dimension/subspace reduction | `034-tq-dimension-subspace` |

This packet answers the remaining closeout question with current-HEAD product
evidence. It is not an f32 bake-off substitution for scorer work; it is the
required post-change 10k / 50k / 100k validation after the scorer work landed.

## Validation

Passed:

- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
- `ecaz bench suite audit`: 24 steps
- `ecaz bench suite run`: 24 succeeded / 0 failed
- `ecaz bench suite status`: 24 completed / 0 failed / 0 stale
- `ecaz bench suite report`: generated

The first suite run omitted explicit PG host/port and failed before any step
executed. The authoritative run is `suite-run-r2.log` with
`--host /Users/peter/.pgrx --port 28818`.

## Product Matrix Result

Config: `artifacts/task124-post-scorer-product-suite.json`, run with
`--artifact-dir artifacts/post-scorer-suite`.

| Scale | Variant | Recall@10 | NDCG@10 | p50 | p95 | p99 | ec_ivf index |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | f32/source | 1.0000 | 1.0000 | 1.14 ms | 1.26 ms | 1.36 ms | 2.9 MiB |
| 10k | TQ final15 | 1.0000 | 1.0000 | 1.04 ms | 1.19 ms | 1.35 ms | 10.9 MiB |
| 50k | f32/source | 1.0000 | 1.0000 | 4.13 ms | 4.33 ms | 4.41 ms | 11.6 MiB |
| 50k | TQ final15 | 0.9980 | 1.0000 | 4.13 ms | 4.48 ms | 4.60 ms | 50.9 MiB |
| 100k | f32/source | 1.0000 | 1.0000 | 8.22 ms | 8.48 ms | 9.24 ms | 22.5 MiB |
| 100k | TQ final15 | 1.0000 | 1.0000 | 8.30 ms | 9.41 ms | 9.68 ms | 100.8 MiB |

## TQ Scorer Path Result

The product-path TQ scorer remains fully SIMD on the measured local arm64 host:
all TQ scorer counter rows report `isa=neon` and `scalar_candidates=0`.

Compared with packet 026's product-path TQ scorer counters, current HEAD
improves scorer elapsed:

| Scale | Packet 026 TQ scorer elapsed | Packet 035 TQ scorer elapsed | Delta |
| --- | ---: | ---: | ---: |
| 10k | 1.811008 ms | 1.779246 ms | -1.8% |
| 50k | 1.851708 ms | 1.788211 ms | -3.4% |
| 100k | 1.907458 ms | 1.804748 ms | -5.4% |

So the reopened TQ scorer work did improve the TQ scorer component in the
in-engine path. It did not turn the product matrix into a durable promotion
result because the shared frontier dominates and the TQ sidecar still has recall
and storage costs at the relevant scales.

## Decision

Closeout outcome: **Shelve product promotion after completing the TQ scorer
optimization pass.**

Task 124 should not claim that TQ final15 is product-ready. The completed
scorer work produced real TQ-internal speed improvements and confirmed full
SIMD dispatch, but the current product matrix still fails the promote condition:

- 50k TQ recall is lower than f32/source (`0.9980` vs `1.0000`).
- 50k and 100k TQ tail latency is not better than f32/source.
- TQ index storage remains much larger than the f32/source index.

The branch now has the evidence that was missing from the rejected `027`
shelve: all seven reopened TQ scorer/compute-path levers were implemented or
attempted and measured with TQ-internal deltas, and the current product matrix
was rerun after those changes.
