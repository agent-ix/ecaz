---
id: 30156
title: Spec Current-State Refresh
agent: coder1
status: open
created: 2026-05-01
checkpoint_commit: ba4e4e8a
---
# Review Request: Spec Current-State Refresh

## Summary

This docs/spec checkpoint refreshes the formal requirements spec after the landed HNSW, IVF, DiskANN, PG18, and docs work.

The checkpoint:

- rewrites `spec/spec.md` as the current Ecaz multi-AM master specification
- adds formal requirements for `ecvector`, multi-AM SQL bootstrap, current HNSW, IVF, DiskANN, and benchmark provenance
- adds `spec/adr/index.md` to handle duplicate historical ADR numbers without renaming files
- marks key landed ADRs/statuses as implemented and parallel index scan as shelved
- rebuilds `spec/tests.md` using the `/spec-matrix` traceability shape
- updates stale PG18/build requirement wording

## Files To Review

- `spec/spec.md`
- `spec/tests.md`
- `spec/adr/index.md`
- `spec/functional/FR-028-ecvector-canonical-row-type.md`
- `spec/functional/FR-029-multi-am-sql-bootstrap.md`
- `spec/functional/FR-030-current-hnsw-am.md`
- `spec/functional/FR-031-ivf-build-and-storage.md`
- `spec/functional/FR-032-ivf-scan-rerank-and-cost.md`
- `spec/functional/FR-033-ivf-insert-vacuum-admin.md`
- `spec/functional/FR-034-diskann-build-and-storage.md`
- `spec/functional/FR-035-diskann-scan-prefilter-rerank.md`
- `spec/functional/FR-036-diskann-insert-vacuum-diagnostics.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
- `spec/non-functional/NFR-008-scale-boundary.md`
- `spec/stakeholder/StR-005-multi-am-vector-search.md`
- `spec/stakeholder/StR-006-benchmark-evidence-discipline.md`
- `spec/usecase/US-012-store-and-query-ecvector.md`
- `spec/usecase/US-013-build-and-tune-ivf.md`
- `spec/usecase/US-014-build-and-tune-diskann.md`
- `spec/usecase/US-015-compare-am-benchmarks.md`

## Validation

- `git diff --check`
- No code tests run. This is a docs/spec-only checkpoint under the repository checkpoint policy.

## Reviewer Focus

1. Does the refreshed master spec accurately describe the current main-branch surface?
2. Are the new FR/US/NFR artifacts at the right level of normativity and detail?
3. Does the ADR index handle duplicate historical ADR IDs clearly enough?
4. Does the test matrix correctly distinguish implemented local evidence from deferred product benchmark gates?
