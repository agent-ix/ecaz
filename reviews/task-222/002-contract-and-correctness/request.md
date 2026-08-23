---
task: 222
packet: 002-contract-and-correctness
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 222 payload projection contract checkpoint

This packet requests review of implementation checkpoint `f088021ea`. It
implements the typed payload-mask API and the correctness-critical
ordering-only boundary requested by packet 001 feedback.

The checkpoint:

- exports `Exact(attnums)` versus `AllColumns(FallbackReason)` and retains the
  typed result in executor state;
- proves the narrow ordering exemption from the original Query and exact
  pathkeys at plan time, then derives the mask from the final PG18 executor
  target list, quals, and query-value expression during `BeginCustomScan`;
- replaces exactly one proven-unused ordering projection with a typed NULL and
  rebuilds projection state, rather than evaluating the distance against an
  unshipped vector;
- retains every attribute used by a visible expression or qual, falls back for
  whole-row/unproved shapes, and preserves the existing system-column error;
- emits verbose EXPLAIN evidence for mask variant, attnums/fallback reason, and
  ordering projection disposition.

The focused three-owner PG18 regression passes. It pins id-only attnum `1`,
retention of attnums `1,2` for visible distance and qual use, `SELECT *`,
whole-row fallback, no upper Sort, mixed frozen/remote execution, result
identity for the covered queries, and the existing `ctid`/`xmin` hard errors.

This is a narrow contract checkpoint, not Task 222 closeout. Null/toast,
correlated rescan with changed Param, EPQ/concurrent update, multi-window and
remote-failure coverage remain in P2, followed by the preregistered
same-generation 100k A/B. No benchmark result or production-default decision
is claimed here.

Validation and provenance are in `artifacts/manifest.md`. Strict repository
clippy remains blocked by four pre-existing warnings outside the changed
files; the same all-target command passes when only those four named baseline
lints are allowed.
