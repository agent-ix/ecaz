# Task 91 Review Request: ADR-071/072 Acceptance

## Summary

This checkpoint advances Task 91 Phase 7 by updating the architecture records
after reviewer approval of the final SPIRE `QuantCodec` cutoff migration:

- ADR-071 is now `ACCEPTED` and records `QuantCodec` as the shared
  quantizer-family scoring contract across HNSW, DiskANN, IVF, and SPIRE.
- ADR-071 explicitly includes `try_score_ip_candidate` in the accepted contract
  so bounded candidate pruning has a common trait surface.
- ADR-072 is now `ACCEPTED` and records the companion boundary:
  `QuantCodec` owns shared scoring, while AM-local storage bindings own
  metadata, tuple/list layout, sidecars, traversal binding, and compatibility.
- `spec/adr/index.md` is updated so the canonical ADR navigation surface
  matches both ADR files.

This does not flip Task 91 itself to complete. The remaining Task 91 closeout
work is the aggregate parity/no-regression evidence packet and task-status
closeout.

## Code Under Review

- Code/doc commit: `dea59fcf9edd01d22b1c01097b1a54dbfb78dddc`
- Files:
  - `spec/adr/ADR-071-unified-quantizer-interface.md`
  - `spec/adr/ADR-072-index-local-quantized-codec-adapters.md`
  - `spec/adr/index.md`

## Validation

Artifacts are under `reviews/task-91/018-adr071-072-acceptance/artifacts/`.

- ADR status/index audit
  - Command: `rg -n 'status: (ACCEPTED|PROPOSED)|Unified quantizer interface|Index-local quantized codec adapters|QuantCodec|try_score_ip_candidate' spec/adr/ADR-071-unified-quantizer-interface.md spec/adr/ADR-072-index-local-quantized-codec-adapters.md spec/adr/index.md`
  - Result: ADR-071 and ADR-072 frontmatter show `status: ACCEPTED`; ADR index rows show `ACCEPTED`; ADR-071 references `try_score_ip_candidate`
  - Log: `artifacts/adr-status-audit.log`
- Stale proposed-status audit
  - Command: `rg -n 'ADR-071.*PROPOSED|ADR-072.*PROPOSED|status: PROPOSED' spec/adr/ADR-071-unified-quantizer-interface.md spec/adr/ADR-072-index-local-quantized-codec-adapters.md spec/adr/index.md`
  - Result: no matches
  - Log: `artifacts/no-stale-proposed-audit.log`
- `git diff --check`
  - Result: passed
  - Log: `artifacts/git-diff-check.log`
- Commit stat
  - Command: `git show --stat --oneline --no-renames HEAD`
  - Result: documents the three ADR/index files changed
  - Log: `artifacts/git-show-stat.log`

## Review Focus

- Confirm ADR-071 now accurately reflects the accepted `QuantCodec` scoring
  contract from Task 91, including cutoff-capable candidate scoring.
- Confirm ADR-072 accurately preserves the AM-local storage-binding boundary
  while recognizing that quantizer-family scoring is now shared.
- Confirm that leaving Task 91 itself open pending aggregate parity/no-regression
  closeout evidence is the right sequencing.
