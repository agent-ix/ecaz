# Task 28 IVF Local Landing Status

## Scope

Consolidate the current A1-A10 state after the A3/A7/A10 follow-up packets,
the bounded HNSW reference attempt, and the local 990k recall deferral.

This packet introduces no new measurements. It is a reviewer map for deciding
whether Task 28's local IVF lane is ready to land, with larger benchmark work
kept outside the local desktop gate.

## Gate Status

| gate | status | evidence |
|---|---|---|
| A1 cost model audit | done | 30076, 30077 |
| A2 streaming vacuum | done | 30079, 30109, 30129 |
| A3 physical compaction/reuse | done for local v1 claim; flat rotating-window churn requires explicit slack | 30080, 30139, 30140, 30141, 30142, 30153 |
| A4 typed exact-score dispatch | done | 30102 |
| A5 quantizer cache-key audit | done | 30102 |
| A6 planner cross-test matrix | done with mixed-predicate caveat | 30077 |
| A7 score-bound pruning | done for PQ-FastScan selected path | 30115, 30116, 30117, 30137, 30138 |
| A8 PQ-FastScan + RaBitQ wiring | done after RaBitQ score hot-path fix and seeded quantizer cache | 30081, 30082, 30152, 30153 |
| A9 100k+ scale | local IVF lane covered; larger/fresher exact comparison deferred | 30126, 30130, 30131, 30133, 30135, 30146, 30149, 30150 |
| A10 quantizer assessment | done for local recommendation with corrected RaBitQ rows | 30097, 30137, 30143, 30144, 30145, 30152 |

## Local Landing Read

The branch is now in a reasonable local landing posture for Task 28's IVF
substrate work:

- correctness and PG18 callback behavior have focused test coverage in the code
  packets
- streaming vacuum and page reuse have 1M/100k local evidence; the flat
  rotating-window churn result depends on explicit `posting_slack_percent`
- PQ-FastScan and RaBitQ are wired through build, scan, insert, and vacuum
- A7 score-bound pruning is landed and measured on the selected PQ-FastScan path
- A10 has an explicit recommendation: keep `auto` unchanged, recommend explicit
  `pq_fastscan, pq_group_size=8` for larger high-dimensional IVF surfaces where
  speed and index size dominate

## Explicit Deferrals

The following should not be treated as local desktop blockers:

- fresh 990k exact-recall fills beyond the already recorded 990k IVF frontier
- long HNSW rebuild/reference attempts on this desktop
- product-class cold/warm/cache/memory claims that need a dedicated benchmark
  environment

Those are larger benchmark-lane items, not unresolved correctness work in the
current IVF implementation.

## Remaining Local Caveats

- A9 is not a product benchmark. It is local scale evidence plus explicit
  deferral of measurements that do not fit this machine.
- A3's flat rotating-window churn behavior requires explicit
  `posting_slack_percent`. Default `posting_slack_percent=0` preserves build
  size and reserves no churn headroom; packet 30142 measured about 24% growth
  on the rotating-window workload without slack and flat behavior with slack
  enabled.
- A10's corrected RaBitQ rows are still bounded, but packet 30152 replaces the
  earlier multi-second rows that measured a per-posting quantizer rebuild bug.
- The recall harness now has better cache mechanics, but first-generation
  990k exact truth is still too expensive to make a local gate.

## Suggested Reviewer Focus

Review should focus on:

- correctness of the IVF code changes since packet 30106
- whether the A3 local v1 claim is narrow and honest
- whether A7's PQ-FastScan-only pruning is correctly gated
- whether A10's recommendation is stated as local evidence, not a product claim

## Artifacts

- `artifacts/manifest.md`
