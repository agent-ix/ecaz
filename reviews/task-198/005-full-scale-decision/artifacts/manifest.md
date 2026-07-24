# Task 198 packet 005 artifact manifest

- Final decision head: `c6cabc917`
- Full 10k/50k runner SHA: `96dbf1067e4d5ec42c5b3cc5a717265463c82c3b`
- Corrected 100k lifecycle runner SHA:
  `3009a1bb1`
- Installed extension SHA: `2ff72b3e49609c44cec881f72edf183a83554412`
- Immutable 100k performance source:
  `reviews/task-198/004-isolated-100k/`
- Task bucket: `reviews/task-198/005-full-scale-decision/`
- Lane: Intel local, three independent PG18 owner processes; owner zero is
  also the single authoritative coordinator
- Fixture: exact/disjoint hash ownership, one index per source table
- Search: trained exact cap-4,096 head, ordered 32 seeds, degree 32, BW4/H100,
  RaBitQ neighbor values, exact final score, identity-keyed lazy10 owner
  payloads
- Protocol: 200 recall queries / 2,000 top-10 trials and 10 warmups / 50 timed
  samples for the decision matrix; a separate 10-query / 2-sample 100k run is
  lifecycle evidence only
- Timestamp: 2026-07-23 America/Los_Angeles

## Commands and source artifacts

Full 10k/50k completion suite:

```text
target/debug/ecaz bench suite audit --config reviews/task-198/005-full-scale-decision/artifacts/task198-full-scale-10k-50k.json
target/debug/ecaz bench suite run --config reviews/task-198/005-full-scale-decision/artifacts/task198-full-scale-10k-50k.json --artifact-dir reviews/task-198/005-full-scale-decision/artifacts/run
```

Corrected 100k lifecycle supplement:

```text
target/debug/ecaz bench suite audit --config reviews/task-198/005-full-scale-decision/artifacts/task198-corrected-lifecycle-100k.json
target/debug/ecaz bench suite run --config reviews/task-198/005-full-scale-decision/artifacts/task198-corrected-lifecycle-100k.json --artifact-dir reviews/task-198/005-full-scale-decision/artifacts/lifecycle-100k
```

Both checked-in configs, audit/run logs, suite manifests, structured
`results.jsonl`, compact summaries, raw recall/latency logs, and node logs are
packet-local. The full suite reports both steps `succeeded`; the corrected
100k suite reports its one step `succeeded`. Every PostgreSQL node
unanimously attested the installed release-profile extension SHA above.

The 100k performance row comes from packet 004's 200-query / 50-sample
isolated A/B. The supplemental 100k run exists only to replace packet 004's
non-triggering 20-row fault query with the corrected production-shape,
64-row, planner-forced drill. Its 10-query recall and two latency samples are
not used in the decision table.

## Full-scale result

| Scale | Recall owner / replica | Warm mean owner / replica | Mean improvement | p95 owner / replica | Traversal owner / replica |
|---|---:|---:|---:|---:|---:|
| 10k | 0.9990 / 0.9990 | 19.50 / 16.40 ms | 15.9% | 23.10 / 20.10 ms | 6.832 / 3.025 ms |
| 50k | 0.9685 / 0.9685 | 20.70 / 17.80 ms | 14.0% | 24.00 / 20.10 ms | 7.790 / 3.624 ms |
| 100k | 0.9625 / 0.9625 | 20.60 / 17.10 ms | 17.0% | 23.80 / 20.00 ms | 7.866 / 3.617 ms |

At every scale, owner and replica arms have identical ordered seed digests,
remote final-payload engagement, and recall. Both traversal reconciliations
pass. Replica traversal has zero remote expansion, transport wait, and
straggler spread. At 100k it replaces 6.405 ms remote expansion with 3.386 ms
local graph/vector read plus 0.140 ms RaBitQ score; final owner payload SQL is
8.977 versus 9.000 ms and remains owner-side.

## Capacity, build, and cache

| Scale | Physical generation | Replica relation | Replica / generation | Bytes copied | WAL | Build | Peak batch |
|---|---:|---:|---:|---:|---:|---:|---:|
| 10k | 242,745,344 | 158,326,784 | 65.2% | 131,520,000 | 137,624,200 | 5.208 s | 3,366,912 |
| 50k | 1,242,742,784 | 823,705,600 | 66.3% | 657,600,000 | 812,199,064 | 24.736 s | 3,366,912 |
| 100k | 2,496,659,456 | 1,659,518,976 | 66.5% | 1,315,200,000 | 1,925,866,616 | 51.995 s | 3,366,912 |

The independent corrected 100k rebuild reproduced relation/copy/peak bytes
exactly, took 50.663 s, and emitted 1,751,890,448 WAL bytes. Cache proxies
after the decision workloads were 53 read / 19,577 hit blocks at 10k, 1,253 /
61,690 at 50k, and 2,669 / 112,311 at 100k.

A deliberately simple per-coordinator 1m projection from the 100k result is
16.60 GB replica relation, 13.15 GB copied, roughly 19.26 GB WAL, and about
520 seconds build time. This is not a 1m measurement. Each future replica
coordinator would add approximately the same capacity, which is why
multi-coordinator serving remains rejected.

## Lifecycle and rollback

All three scales pass:

- owner-outage partial-build rollback with zero catalog residue;
- idempotent build replay and content-digest identity;
- Ready semantic identity;
- genuine failure before the second replica expansion, full owner restart,
  `fallback_count=1`, and identical ordered results;
- deliberately truncated/corrupt image fallback;
- durable `Ready -> Stale`, exactly one retryable `40001`, then owner fallback;
  and
- fenced retire/reclaim, idempotent reclaim replay, zero residue, and
  removed-image owner fallback.

Packet 003 separately covers null, external TOAST, qual/projection,
deepening, mixed ownership, and post-first-batch owner failure semantics.

## Decision

**PROMOTE to Task 199 productionization; do not change production defaults in
Task 198.**

The latency result is consistent and material with exact recall/result
identity, so STOP would discard a demonstrated architectural win. The
65–66% storage amplification, WAL, explicit build, single-authority
restriction, and first-mutation retry are too consequential for direct default
promotion. Task 199 therefore owns an explicit-build/read-mostly operator
surface, normal-build feature isolation, remaining production recovery/race
drills, observability/capacity policy, and a fresh normal-release
10k/50k/100k gate.
