---
task: 195
packet: 002-release-matrix
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 195 release-matrix closeout candidate

This packet requests outside review of the production owner-schema cache and
recommends **PROMOTE**. A release-profile A/B at 10k, 50k, and 100k preserved
exact recall and all compared materialization work while reducing warm mean
latency by 8.33%, 13.28%, and 18.11%, respectively. The intended owner
open/validate stage fell from about 7 ms/scan to 0.024--0.028 ms/scan.

## A/B result

| Scale | Recall, before -> after | Warm mean, before -> after | Delta | p95, before -> after | Owner open/validate, before -> after | Storage delta |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 0.9990 -> 0.9990 | 22.80 -> 20.90 ms | -8.33% | 26.00 -> 25.50 ms | 7.030 -> 0.028 ms | -16 KiB (-0.006749%) |
| 50k | 0.9685 -> 0.9685 | 24.10 -> 20.90 ms | -13.28% | 27.50 -> 24.30 ms | 6.792 -> 0.024 ms | +8 KiB (+0.000659%) |
| 100k | 0.9625 -> 0.9625 | 24.30 -> 19.90 ms | -18.11% | 27.20 -> 23.30 ms | 7.122 -> 0.024 ms | -24 KiB (-0.000984%) |

The storage measurements came from independent clean builds, so the one-to-
three PostgreSQL page deltas are allocator/page-layout variation rather than a
format change. Task 195 changes no stored representation. Query, training
slice, head sample, and seed-ID digests matched at every scale. All topology,
owner-engagement, and traversal-reconciliation gates passed.

The packet compared all 78 production materialization work metrics from each
side and found zero mismatches. Owner payload SQL remained flat while remote
materialization fell from 10.474/10.700/10.721 ms to
7.230/7.155/7.028 ms at 10k/50k/100k, reproducing the expected causal stage
movement rather than shifting or reducing payload work.

## Provenance and isolation

- Baseline extension: `cf862b9507f8ba211bb3e903df54c1f127714a51`,
  release PG18 plus attribution; installed and target binary were byte-equal
  (`34a1680e...`, 24,271,128 bytes).
- Candidate extension/runner: `51e5d614501742cb9d5db4b6b7d39ebcfba5d7c0`
  (production code commit `65e746fc1a85f8efa8032a81aa2cc95b55c23503`),
  release PG18 plus attribution; installed and target binary were byte-equal
  (`ad4798a6e...`, 24,269,272 bytes).
- Final normal production build: release PG18 without attribution; installed
  and target binary were byte-equal (`0967640b...`, 23,867,056 bytes).
- Both A/B sides used the same checked-in suite configuration (SHA-256
  `7b5ae0f2...`), runner, three local PG18 owners, one index per table, exact
  training landmarks, RaBitQ neighbor scoring, lazy materialization window 10,
  beam width 4, hop cap 100, 200 recall queries, 2,000 latency trials, 10
  warmups, and 50 measured iterations.
- The benchmark variant is the normal production path:
  `production:persisted_head:32:32:rabitq:10`. It contains no schema-cache
  selector and requires no removed GUC.
- The final normal installed SQL contains neither the attribution profile
  endpoint nor the removed schema-cache selector nor neighboring benchmark
  controls. Attribution remains feature-isolated.

The earlier packet-001 lifecycle test had installed a debug extension. Before
each A/B side, this packet rebuilt and installed an explicitly release-profile
binary and verified target/installed size and SHA-256 equality. The final
installed extension is the normal, non-attribution release binary.

## Outside review focus

Please verify the binary identities and common suite provenance, the exact
recall/work equality, the causal owner-stage movement, the interpretation of
the sub-page storage variance, and the normal-build feature-isolation audit.
If those agree with packet 001's epoch-fencing review, Task 195 is ready to
merge.
