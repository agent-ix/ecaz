# Task 122 review request: SPIRE prune release suite

## Scope

This packet records release-backed 10k / 50k / 100k A/B evidence for the SPIRE
pre-materialization prune added in `aa799704b`.

No new code change is under review in this packet. It validates the previous
GUC-gated prune change against a release backend.

## Evidence

Primary artifacts:

- `artifacts/task122-spire-prune-release-suite.json`
- `artifacts/guc-check-release.log`
- `artifacts/suite-audit-release.log`
- `artifacts/suite-dry-run-release.log`
- `artifacts/suite-run-release.log`
- `artifacts/suite/suite-manifest.json`
- `artifacts/suite/results.jsonl`
- per-step logs and funnel JSONL under `artifacts/suite/`

The suite manifest records:

- backend `build_profile=release`
- all 36 steps `succeeded`

## Result

At fixed `nprobe=24`, `rerank_width=25`, and `ec_spire.max_candidate_rows=25`:

- Prune-on and prune-off have identical recall/NDCG at 10k, 50k, and 100k.
- Candidate materialization drops from:
  - 251,555 to 8,495 at 10k
  - 525,067 to 11,796 at 50k
  - 766,494 to 10,517 at 100k
- Heap rerank output remains 2,500 rows at every scale.
- Latency improves slightly versus prune-off, but does not beat RaBitQ across the matrix.

Key release latency rows:

| Scale | Lane | recall@k | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k | TQ prune on | 1.0000 | 2.15 ms | 2.44 ms | 2.84 ms |
| 10k | RaBitQ | 1.0000 | 2.14 ms | 2.35 ms | 2.84 ms |
| 50k | TQ prune on | 0.9450 | 4.40 ms | 4.74 ms | 5.84 ms |
| 50k | RaBitQ | 0.9450 | 4.33 ms | 4.80 ms | 5.80 ms |
| 100k | TQ prune on | 0.8940 | 6.30 ms | 6.92 ms | 8.02 ms |
| 100k | RaBitQ | 0.8940 | 6.39 ms | 6.93 ms | 8.05 ms |

## Ask

Please review whether the evidence supports keeping the prune as a Phase 2
materialization improvement, and whether the next Task 122 slice should move to
matched-recall candidate-budget / rerank-width sweeps rather than further
micro-optimizing this fixed `nprobe=24`, `rerank_width=25` point.
