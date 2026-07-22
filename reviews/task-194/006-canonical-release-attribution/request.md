---
task: 194
packet: 006-canonical-release-attribution
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 194 canonical release attribution

This checkpoint replaces the rejected direct-run evidence with a successful
50/10 `ecaz bench suite` run from the corrected release extension. The suite
completed with no failed, missing, or stale steps and records unanimous
three-owner provenance for extension `809db6716` built in `release` profile.

The post-revert A/A resolves F1: `materialize_owner_open_validate_work` is
6.927 ms/scan, consistent with the accepted Task 187 baseline (6.722 ms) and
not the unprovenanced 0.026 ms result. The earlier collapsed value therefore
measured the reverted Task 192 schema-cache candidate, not current production.

The owner-sideband rework resolves F2 with remote-owner measurements:

- traversal total: 7.476 ms/scan;
- remote expansion: 6.067 ms/scan;
- remote owner service: 1.930 ms/scan;
- transport remainder: 4.078 ms/scan;
- owner straggler spread: 0.394 ms/scan;
- coordinator partition/decode/frontier work: 0.030 ms/scan combined;
- 10 hop rounds and 40 requested/returned nodes per scan, with zero repeated
  nodes.

The run retains 0.9625 recall and measures 23.50 ms warm mean latency (27.00
ms p95). Materialization remains separately attributed at 6.927 ms/scan
open/validate, 8.717 ms/scan payload SQL, and 0.297 ms/scan node lookup.

The evidence rejects traversal node caching (no repeated requests) and selects
one bounded Task 194 candidate for the next packet: fixed-work wider/fewer
rounds, comparing the production 4x100 cap against 8x50 on the same immutable
generation. No production behavior is promoted by this checkpoint.

Evidence and provenance are recorded in `artifacts/manifest.md` and the
canonical Task 194 packet-002 suite artifacts it cites.
