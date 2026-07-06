# Task 145 Packet 009: Large-Leaf Geometry

## Request

Please review the Task 145 AC3 large-leaf geometry measurement.

Packet evidence:

- `reviews/task-145/009-large-leaf-geometry/artifacts/manifest.md`

This packet was packaged after reading the packet 008 feedback. Packet 008 is
treated as an inert/null bound-prune A/B; this packet does not rely on 008 as a
measured bound-prune negative.

## Scope

The suite compares the 100k fine frontier control against the explicit
fewer-larger-leaves candidate from Task 145:

- control: `100k n1024/b0`
- large leaf: `100k n128/b0`

Both cells are release `spire-local-multinode` runs with block summaries built,
block pruning enabled (`global128`), `rerank_width=50`, `max_candidate_rows=100`,
and fixed/ratio4/ratio8 variants across nprobe 8,16,32,64,96.

## Result

Decision: drop `100k n128/b0` for Task 145.

The large-leaf cell does exercise block pruning heavily, but it is not
recall-competitive and does not produce a latency win large enough to matter.

Best rows:

| cell | variant | nprobe | recall@10 | p50 | p95 |
| --- | --- | ---: | ---: | ---: | ---: |
| n1024 control | ratio8 | 96 | 0.9340 | 143.658 ms | 150.947 ms |
| n128 large-leaf | ratio8 | 16 | 0.8480 | 136.718 ms | 141.952 ms |
| n128 large-leaf | fixed | 96 | 0.7840 | 142.198 ms | 151.332 ms |

At nprobe96, n128 loses 15 recall points versus n1024 while p50 stays in the
same transport-dominated band.

Block-pruning engagement at nprobe96:

| cell | blocks available | blocks skipped | leaf candidates |
| --- | ---: | ---: | ---: |
| n1024 control | 126,184 | 49,387 | 1,196,856 |
| n128 large-leaf | 964,963 | 888,163 | 1,228,150 |

Storage improves modestly:

- n1024 SPIRE index: 111.0 MiB
- n128 SPIRE index: 98.5 MiB

That storage reduction is not worth the recall loss.

## Notes

The top-level `suite-results.jsonl` is empty for nested local-multinode steps;
each nested `bench-suite/results.jsonl` has 360 rows and is the authoritative
result source. Generated correctness TSVs, server logs, load logs, and remote
materialization logs are intentionally left uncommitted.
