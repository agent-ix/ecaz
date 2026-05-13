# Review Request: SPIRE Read Pass-Through and Local Recall Fix

- coder: coder1
- code commit: `f50482304336af6f2a01f47b96d72e778f60e847`
- tracker rows: Phase 12.9 packet-local artifact capture and final local
  production-readiness bundle

## Scope

This packet resolves the two remaining readiness blockers from packet `30978`:

1. ordinary SPIRE read queries used by `ecaz bench spire-pipeline` were being
   treated as ADR-069 DML frontdoor failures;
2. production heap-resolution rerank replaced operator-facing `<#>` scores with
   raw positive inner product, causing local readiness recall to rank the worst
   rows first.

## Code Changes

- DML frontdoor hook now fail-closes unsupported UPDATE/DELETE shapes and
  actual PK-select frontdoor edge shapes, while ordinary non-PK `SELECT` reads
  pass through as `non_pk_select_pass_through`.
- PK-select predicate detection still catches malformed PK-frontdoor shapes
  such as `id IN (...)`, `id = numeric`, and `OR` equality so existing
  fail-closed coverage remains intact.
- Production heap-resolution exact rerank now returns negative inner product,
  matching SQL `<#>` ordering and remote compact candidate score semantics.

## Evidence

Artifact metadata and key lines are in `artifacts/manifest.md`.

- `cargo test dml_frontdoor --lib`: 28 passed, including PG18 hook coverage.
- `cargo test remote_heap_exact_score_uses_orderby_negative_inner_product --lib`: passed.
- `ecaz bench spire-pipeline` now completes against the local PG18 readiness
  fixture and reports `recall@k = 1.0000`.
- The final bench artifact includes endpoint tuple transport readiness
  (`pg_binary_attr_v1_ready true`), p50/p95/p99 latency, route/candidate/heap
  rows, local-store object/read counters, and local remote-fanout status.

## Reviewer Focus

- Confirm the DML hook boundary is now correct: non-PK reads pass through, but
  malformed PK-select frontdoor attempts still fail closed.
- Confirm the heap-resolution score sign change is the right shared fix for
  local AM output and remote/local heap candidate merging.
- Confirm packets `30978`, `30979`, and `30980` together satisfy the Phase 12.9
  local readiness artifact rows without making AWS/product-scale claims.
