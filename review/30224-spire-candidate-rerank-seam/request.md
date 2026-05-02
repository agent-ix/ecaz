---
id: 30224
title: SPIRE Candidate Rerank Seam
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: c2892cd9
---

# Review Request: SPIRE Candidate Rerank Seam

## Summary

This checkpoint adds an injected exact-rerank seam for already ranked SPIRE
scan candidates.

- Adds `rerank_scored_candidates_by_ip`.
- Uses the same score convention as approximate candidate ranking:
  scorer-provided inner product is stored as `score = -ip`.
- Supports `rerank_width == 0` as rerank-all, matching the local IVF heap-rerank
  width convention.
- Reranks only the prefix selected for rerank, sorts that prefix, and truncates
  when `rerank_width > 0`.
- Rejects non-finite exact scorer outputs.
- Keeps the seam caller-injected; no heap fetch, AM scan opaque state, or
  callback execution is introduced in this checkpoint.

## Non-Goals

- No PostgreSQL heap rerank callback.
- No quantizer integration.
- No AM result production.
- No planner/GUC surface for rerank width.

## Review Focus

- Whether the `rerank_width == 0` convention should be carried over from
  `ec_ivf` at this helper boundary.
- Whether prefix-only rerank plus truncation is the right behavior before the
  scan callback has a pre-rerank candidate limit.
- Whether the helper should keep approximate scores somewhere for diagnostics
  once exact scores overwrite `candidate.score`.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 132 passed, 0 failed
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
