# Task 123 Review Request: Local Multi-Instance Communications Measurement

## Scope

This packet responds to the reopened Task 121/123 testing gap with a 200-query local PG18 multi-instance measurement focused on the core SPIRE routing/materialization/remote heap algorithm. It does not claim true cross-network coverage.

This packet also incorporates the pre-positioned reviewer feedback now present at `feedback/2026-06-28-01-reviewer.md`: for `boundary_replica_count > 0`, the current pre-materialization prune is structurally inert because the candidate configs use `VecIdDedupeEnabled`. Therefore the prune on/off arms here are a no-op confirmation and should not be read as a valid prune efficacy A/B for the b2/b4 candidates.

The run covers:

- 100k staged real corpus, `rabitq`
- `n128` / `boundary_replica_count=4` / `nprobe=96`
- `n1024` / `boundary_replica_count=2` / `nprobe=64`
- 200 production-read queries per variant
- `source` projection (`id,source`) and narrow `id` projection
- `ec_spire.pre_materialization_prune=on` vs `off`
- default and `--max-routed-candidate-rows 25000` forms

## Artifacts

Primary provenance:

- `artifacts/manifest.md`
- `artifacts/task123-mi-communications-prune-ab-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-run.log`

Structured result sources:

- `artifacts/n128-b4-200q/bench-suite/results.jsonl`
- `artifacts/n1024-b2-200q/bench-suite/results.jsonl`

Human-readable logs:

- `artifacts/n128-b4-200q/bench-suite/production-read-k10-*.log`
- `artifacts/n1024-b2-200q/bench-suite/production-read-k10-*.log`
- `artifacts/n128-b4-200q/local-multinode.log`
- `artifacts/n1024-b2-200q/local-multinode.log`

Generated corpus/query TSVs were deleted before commit, per packet rules. The packet has no remaining `*.tsv` files.

## Verdict

The local multi-instance core path is now measurable with structured artifacts, clean projection coverage, and no remote heap dispatch failures. The earlier 32-query latency optimism should remain retracted.

The review-relevant result is the projection/communications signal: `id,source` sends about 73.9 MB of heap payload over the three remotes for 200 queries, while `id` sends 48 KB, but latency is close within each surface. That points away from transport payload bytes as the dominant latency driver for this local core path.

This is not a prune closeout: the b2/b4 prune arm is confirmed inert under the current dedupe mode. A meaningful prune A/B still needs either a recall-safe implementation under `VecIdDedupeEnabled` or an explicitly scoped b0 mechanism datapoint.

Both fixtures passed:

```text
SPIRE local multinode fixture passed
HARNESS PASSED
```

## Key Results

n128 / b4 / nprobe 96 / 200 queries:

| variant | p50 | p95 | recall@k |
| --- | ---: | ---: | ---: |
| source-prune-on-default | 5524.614 ms | 5933.579 ms | 1.0000 |
| source-prune-off-default | 5508.976 ms | 5902.777 ms | 1.0000 |
| id-prune-on-default | 5472.119 ms | 5940.613 ms | 1.0000 |
| id-prune-off-default | 5690.467 ms | 7266.924 ms | 1.0000 |

n1024 / b2 / nprobe 64 / 200 queries:

| variant | p50 | p95 | recall@k |
| --- | ---: | ---: | ---: |
| source-prune-on-default | 851.406 ms | 1002.419 ms | 1.0000 |
| source-prune-off-default | 836.644 ms | 963.494 ms | 1.0000 |
| id-prune-on-default | 803.108 ms | 973.497 ms | 1.0000 |
| id-prune-off-default | 796.862 ms | 914.903 ms | 1.0000 |

All variants on both surfaces reported:

- `remote_heap_ready_dispatch_sum=600`
- `remote_heap_failed_dispatch_sum=0`
- `remote_heap_candidate_sum=6000`
- `payload_rows_sum=6000`
- `returned_sum=2000`

Measured heap payload from per-node timeline rows:

| projection | bytes per remote | total bytes over 3 remotes / 200 queries |
| --- | ---: | ---: |
| `id,source` | 24632000 | 73896000 |
| `id` | 16000 | 48000 |

## Reviewer Notes

- The projection failure from packet 012 is fixed for the local multi-instance core path by running with `ec_spire.max_remote_payload_bytes_per_row=16384`; both `id,source` and `id` projections complete.
- The packet preserves structured `results.jsonl` evidence under each fixture's nested bench suite. The top-level suite `results.jsonl` is empty because the local multinode harness writes the production-read result artifacts inside the fixture directories.
- Disk was not in the prior ENOSPC condition during this run. After cleanup, `/dev/sdf` mounted at `/tmp` had `48G` available and the packet artifact directory was `1.4M`.
- This packet supports a conservative algorithm verdict: `id` projection materially reduces payload bytes, but payload bytes are not the dominant latency driver in this local multi-instance measurement.
- `ec_spire.pre_materialization_prune` is not evaluated as an engaged b2/b4 candidate lever here; the on/off rows are retained as evidence that the existing guard is inert for these boundary-replica configs.
