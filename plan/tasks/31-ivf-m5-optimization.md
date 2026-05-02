# Task 31: IVF M5 Optimization

Status: proposed
Owner: coder1 / runtime-index track
Priority: 1

## Goal

Use the new Apple Silicon M5 laptop as the fast local optimization host for the
landed IVF path. The first objective is not a product benchmark; it is a
repeatable local loop that identifies the biggest IVF bottleneck, lands one
optimization at a time, and records packet-local evidence.

IVF stays first because Task 28 landed the broadest surface area and still has
the clearest open local optimization questions: scan-volume control,
PQ-FastScan/RaBitQ scoring cost, posting-list I/O shape, and churn behavior.

## Baseline Rules

- Record the machine, macOS build, PostgreSQL 18 settings, compiler profile,
  extension SHA, corpus manifest, row count, dimensionality, query count, and
  cache state in every measurement packet.
- Keep local M5 numbers explicitly scoped as local development evidence. Do not
  present them as product-class cloud claims.
- Prefer one-index-per-table fixtures for cross-AM comparisons unless a packet
  explicitly measures shared-table planner behavior.
- Use release-installed extension builds for timing.

## Phase 1: M5 IVF Baseline

- Re-run the strongest landed Task 28 IVF surfaces on the M5:
  - PQ-FastScan group size 8 at 10k, 25k, 100k, and any locally feasible 990k
    surface.
  - RaBitQ on the same surfaces where build time is tolerable.
  - TurboQuant only as a reference row unless profiling says it is still
    relevant.
- Capture build time, index size, recall@10, p50/p95/p99 latency, memory
  high-water mark, and selected planner/explain counters.
- Reconfirm the current recommended explicit IVF profile:
  `storage_format = pq_fastscan`, `pq_group_size = 8`, heap rerank enabled, and
  an explicit `rerank_width`.

## Phase 2: Pick The First Bottleneck

Use Phase 1 plus profiling to classify the first target as one of:

- posting-list I/O and scan volume;
- PQ-FastScan/RaBitQ scoring throughput;
- exact heap rerank;
- candidate allocation/dedup/top-k overhead;
- centroid routing and `nprobe` selection;
- live-insert/vacuum churn.

Only one classification should feed the first implementation checkpoint.

## Phase 3: Implementation Slices

Recommended order, subject to Phase 2 evidence:

1. **Posting-list scan plan.** Finish the scan-level merged block plan from the
   Task 28 handoff and measure whether block-order streaming beats list-order
   streaming on M5 storage/cache.
2. **Score-as-you-read.** Avoid retaining broad candidate materialization where
   the result can be scored, deduped, and bounded incrementally.
3. **Apple Silicon scoring pass.** Profile the active M5 SIMD backend before
   writing hardware-specific code. If the current dispatch only reaches NEON,
   measure NEON ceilings first and file any SVE/SVE2/SME work under Task 21
   rather than hiding it inside IVF.
4. **Rerank budget policy.** Sweep rerank width with recall floors and convert
   any stable recommendation into reloption or documentation changes.
5. **Churn profile.** Re-measure `posting_slack_percent` on sustained insert /
   vacuum workloads and decide whether defaults or guidance should change.

## Validation

- For code changes, use the narrowest PG18 test or benchmark that covers the
  touched behavior.
- Every performance claim needs packet-local raw logs and a manifest.
- If tests are skipped for docs-only or measurement-planning checkpoints, say
  so in the review request.

## Stop Conditions

- Stop a slice when it fails to move the selected metric by at least 5% on a
  repeatable M5 fixture, unless it removes a correctness risk.
- Do not keep tuning `nlists`, `nprobe`, or `rerank_width` without a fixed
  baseline and recall floor.
- Defer larger AWS/RDS-class claims until the local M5 loop identifies a stable
  profile worth reproducing.
