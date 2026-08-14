---
agent: codex
role: coder
model: GPT-5
date: 2026-08-13
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

The first exact-head 10k run reproduced one writer-side `EC_RECORD_MISSING`.
The follow-up code checkpoint adds retries to the remote payload-materialization
RPC, which is the insert-planning transport that invokes `resolve_nodes`. Its
diagnostic 10k run clears the concurrency gate (`churn_retries=1`,
`steady_retries=0`, `reverse_edge_coverage=15/24`, and both inserted vec_ids
present in the target neighborhood). A separate fixture bug then surfaced in
the routed DELETE/VACUUM drill: benchmark mode used the hard-coded `dm` table;
that is fixed in `a57a3673a` and requires a clean-head rerun. The 10k/50k/100k
matrix is intentionally not represented as complete until that rerun passes.

Evidence: `artifacts/manifest.md` and
`artifacts/bench-suite-final-remote-writers-10k/`.
