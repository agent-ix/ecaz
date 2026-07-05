# Task 143 Packet 006 Artifact Manifest

- Head SHA: `d926dc287a0c1495186dc07dd6a2f205daa07287`
- Branch: `task-143-spire-leaf-ranking-route-overfetch`
- Task bucket: `reviews/task-143/006-leaf-ranking-decision`
- Slice: Task 143 Phase 2 promote / iterate / negative decision, based on release A/B packets 003-005.
- Lane / fixture / storage / rerank mode: local PG18 release backend evidence from `ec_spire`, `bits=4`, `boundary_replica_count=0`, `storage_format=rabitq`, default rerank width.
- Isolated/shared surface: source packets use isolated local table/index prefixes.
- Backend profile: source packets record `ecaz_build_profile() = release` and node profile `coordinator:28818:release`.

## Source Evidence

| Source packet | Head SHA at run | Fixture | Suite config | Structured results |
| --- | --- | --- | --- | --- |
| `reviews/task-143/003-release-10k-ab` | `368907103c68ef9e91118678d7c6755df6bc8500` | 10k, nlists=128, b0 | `artifacts/suite-task143-10k-ab.json` | `artifacts/suite-results.jsonl` |
| `reviews/task-143/004-release-50k-n1024-ab` | `a363c847b066e913775fdd825be6a9c90d9d9861` | 50k, nlists=1024, b0 | `artifacts/suite-task143-50k-n1024-ab.json` | `artifacts/suite-results.jsonl` |
| `reviews/task-143/005-release-100k-n1024-ab` | `900946c97f7ebc4f0c906129efd5c43fdb7159cc` | 100k, nlists=1024, b0 | `artifacts/suite-task143-100k-n1024-ab.json` | `artifacts/suite-results.jsonl` |

This packet did not run new `ecaz bench suite` jobs. It is a review decision
packet over the release `ecaz bench suite` artifacts above.

## Feedback Scan

- Read `reviews/task-141/001-release-anchor-rebaseline/feedback/2026-07-05-02-agent-ix.md`; Task 141 P0 substrate is approved and unblocks Tasks 142-146.
- No Task 143 feedback files existed before this packet.

## Decision Table

| Fixture | Variant | nprobe | recall@10 | p50 | Baseline comparison | Promotion-gate read |
| --- | --- | ---: | ---: | ---: | --- | --- |
| 10k/n128 | baseline | 32 | 0.9965 | 92.721 ms | - | - |
| 10k/n128 | baseline | 64 | 0.9995 | 174.386 ms | - | - |
| 10k/n128 | leaf-only | 32 | 1.0000 | 89.706 ms | +0.0035 recall and -3.015 ms vs baseline nprobe32 | Passes the aggressive half-nprobe gate: leaf-only nprobe32 >= baseline nprobe64. |
| 50k/n1024 | baseline | 32 | 0.8965 | 65.688 ms | - | - |
| 50k/n1024 | baseline | 64 | 0.9390 | 128.159 ms | - | - |
| 50k/n1024 | baseline | 96 | 0.9590 | 187.015 ms | - | - |
| 50k/n1024 | leaf-only | 32 | 0.9105 | 66.356 ms | +0.0140 recall and +0.668 ms vs baseline nprobe32 | Does not pass half-nprobe gate: leaf-only nprobe32 < baseline nprobe64. |
| 50k/n1024 | leaf-only | 64 | 0.9475 | 122.661 ms | +0.0085 recall and -5.498 ms vs baseline nprobe64 | Positive equal-nprobe result, but not a half-nprobe promotion result. |
| 50k/n1024 | overfetch-2.0 | 96 | 0.9605 | 183.365 ms | +0.0015 recall and -3.650 ms vs baseline nprobe96 | Best overfetch recall here, but the delta is too small and slower than leaf-only at lower probes. |
| 100k/n1024 | baseline | 32 | 0.8585 | 123.136 ms | - | - |
| 100k/n1024 | baseline | 64 | 0.9120 | 241.140 ms | - | - |
| 100k/n1024 | baseline | 96 | 0.9300 | 371.433 ms | - | - |
| 100k/n1024 | leaf-only | 32 | 0.8895 | 118.427 ms | +0.0310 recall and -4.709 ms vs baseline nprobe32 | Does not pass half-nprobe gate: leaf-only nprobe32 < baseline nprobe64. |
| 100k/n1024 | leaf-only | 64 | 0.9375 | 246.891 ms | +0.0255 recall and +5.751 ms vs baseline nprobe64 | Beats baseline nprobe96 recall, but the latency tradeoff is mixed. |
| 100k/n1024 | leaf-only | 96 | 0.9570 | 362.912 ms | +0.0270 recall and -8.521 ms vs baseline nprobe96 | Strong equal-nprobe result. |
| 100k/n1024 | overfetch-2.0 | 64 | 0.9315 | 239.017 ms | +0.0195 recall and -2.123 ms vs baseline nprobe64 | Faster than leaf-only at nprobe64 but lower recall than leaf-only. |

## Decision

- Leaf-score-only routing is a validated positive fix. It improves baseline
  recall at every measured nprobe in the 10k, 50k, and 100k release slices, and
  it improves latency in the higher-probe 50k/100k rows that matter for the
  current frontier.
- Do not flip `ec_spire.leaf_score_only_routing` to default-on in this packet.
  The strict Task 143 half-nprobe promotion gate is not consistently met at
  50k and 100k. The code remains available behind the default-off GUC for
  reviewer approval and follow-on Task 146 frontier selection.
- Keep `ec_spire.route_overfetch_multiplier` at its default `1.0`. Overfetch
  improves baseline recall, but it does not beat leaf-only recall at 100k and
  only wins by a marginal `+0.0010` over leaf-only at 50k nprobe96.
- Hand the remaining route precision problem to Task 144. The source packets
  show route containment equals final distinct recall in every row, so the
  unsolved gap remains in route / leaf selection rather than downstream rerank.

## Review Request

Please review this as the Task 143 Phase 2 decision:

- accept leaf-score-only routing as a positive, release-validated candidate;
- confirm that default-on promotion should wait because the half-nprobe gate is
  mixed at 50k/100k;
- confirm overfetch remains diagnostic/default-off;
- confirm Task 144 owns the next precision step.
