# Task 146 Packet 004: Anchor Reporting and Suite

## Request

Review the Task 146 matched-anchor suite and reporting contract before running
the result matrix.

This packet addresses the packet 001 review notes that the matrix must pin HNSW
as well as IVF anchors, report matched-scale 10k/50k anchors, prove Task 142
epoch-cache engagement, and frame the 4x/15% gate as a viability band rather
than Pareto dominance.

## Evidence

- Anchor config: `artifacts/suite-task146-release-anchors.json`
- Manifest: `artifacts/manifest.md`
- Validation logs: `artifacts/audit.log`, `artifacts/dry-run.log`
- Dry-run manifest: `artifacts/dry-run-suite-manifest.json`

## Decision Rules Preserved

The Task 146 SPIRE shapes remain the frozen packet 001 shapes. This packet does
not add a SPIRE candidate or promote any Task 144/145 lever. It only fills the
baseline side of the frontier table.

Task76 has 10k and 100k ecaz IVF/HNSW controls, but no 50k ecaz IVF/HNSW
controls. The Task 146 result packet must either run this anchor suite or cite a
review-approved equivalent packet. It must not estimate the 50k anchor.

The 15% scan gate remains the permissive viability band because Tasks 139/144
showed the high-recall frontier already failing below that range at 50k/100k.
The result table must also report the stricter 10% line so a 15%-only pass is
not misread as a stronger Pareto result.

## Engagement and Faulty-Evidence Rule

The result packet must include the Task 142 epoch-cache profile fields
(`manifest_cache_hit_sum`, `manifest_cache_miss_sum`,
`routing_hierarchy_load_sum`, `socket_open_sum`,
`endpoint_identity_query_sum`) and per-node build profiles for latency rows.

Any row where the claimed mechanism's engagement counter is zero is faulty/null
for that mechanism. It cannot support recall-safety, latency, or promotion
claims. This explicitly carries forward the Task 145 bound-prune failure mode:
packet 008's recall/latency conclusions were rejected because the mechanism did
not engage; packet 011 made the inert/null result provable with
`pre_materialization_pruned_sum = 0`.

## Task 145 Structural Handoff

Task 145's durable handoff is structural: the remote floor is dispatch fan-out
times the 30000 remote heap frontier. Scan/leaf/pruning economy cannot move that
floor if dispatch and heap-frontier counters stay fixed. Task 146 must report
those counters separately from local scan economy before making any promote or
iterate call.
