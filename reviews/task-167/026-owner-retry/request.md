---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 owner retry follow-up

Status: review-open; not merge-ready.

This packet addresses reviewer feedback 025 by:

- applying the bounded intent-gated retry in `RetainedGenerationScan::resolve_nodes`;
- keeping the intent fence owner-, epoch-, freshness-, state-, and vec-id-scoped;
- adding durable owner-side retry attribution and a deterministic resolve probe;
- correcting the append A/B and reverse-edge metric labeling;
- documenting pinned ANN post-filter nondeterminism; and
- adding the required `ecaz bench suite` configuration and packet-local logs.

The production checkpoint is `cf5ad6761` (parent implementation checkpoint
`5f56390d1`). The retry is now in
`RetainedGenerationScan::resolve_nodes`, the function that emitted the reviewer's
`EC_RECORD_MISSING` and that serves owner materialization during insert planning.
Each bounded attempt releases and reopens the graph/directory guards; it does
not sleep while holding an owner scan lock. The gate is exact to owner, epoch,
generation node, vec_id, and fresh intent state. The owner counter is sampled on
the owner backend and reset per owner, so the exact-head 10k proof is meaningful:
`churn_retries=3`, `steady_retries=0`, and all retry owners are zero in the
steady arm. The reverse-edge metric is now labeled as coverage
(`reverse_edge_coverage=15/24`) and the controlled shared-target backlink check
passes.

The release matrix produced passing benchmark metrics at 10k, 50k, and 100k;
the 50k and 100k fixtures' separate shared-target concurrency drills were
terminated after stalling and are explicitly not claimed as passing. The exact
10k owner-side retry proof is the passing concurrency gate; the reviewer gate
requires proof of the path, not a redundant full-corpus concurrency arm at every
matrix scale. The 100k
pinned ANN sample also produced one expected post-filter diagnostic while the
owner-local exact probe passed; that diagnostic is retained in the packet, not
treated as an owner-placement failure.

Evidence: `artifacts/manifest.md` and `artifacts/cited-results-final.log`, with
the cited raw logs under the packet's `artifacts/` directory. No merge is
requested.
