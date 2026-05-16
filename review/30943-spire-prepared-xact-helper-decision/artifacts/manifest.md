# Artifact Manifest: SPIRE Prepared Xact Helper Decision

- head SHA: `a3e8258bc0be5e1aa92b2b880b937c9a9a3897cf`
- packet/topic: `30943-spire-prepared-xact-helper-decision`
- timestamp: `2026-05-12T23:49:55Z`
- isolated one-index-per-table or shared-table surfaces: n/a, docs/tracker
  decision slice

## Artifacts

### `git-diff-check.log`

- lane: static whitespace validation
- fixture: docs/tracker diff for commit `a3e8258b`
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "git diff --check HEAD^ HEAD" review/30943-spire-prepared-xact-helper-decision/artifacts/git-diff-check.log`
- key result lines: command exited successfully with no diff whitespace
  diagnostics.

### `docs-decision-grep.log`

- lane: docs/tracker decision evidence
- fixture: ADR-069, SPIRE diagnostics, and Phase 12 tracker
- storage format: n/a
- rerank mode: n/a
- command used: `script -q -c "rg -n 'recover_orphaned_prepared_xacts|Decision: defer|max_prepared_transactions readiness' docs/SPIRE_DIAGNOSTICS.md spec/adr/ADR-069-spire-distributed-write-path-scope.md plan/tasks/task30-phase12-spire-production-hardening.md" review/30943-spire-prepared-xact-helper-decision/artifacts/docs-decision-grep.log`
- key result lines:
  - `plan/tasks/task30-phase12-spire-production-hardening.md:186:  - [x] Decision: defer the helper for v1. ADR-069 and`
  - `spec/adr/ADR-069-spire-distributed-write-path-scope.md:361:\`ec_spire_recover_orphaned_prepared_xacts(node_id)\` helper. Remote`
  - `docs/SPIRE_DIAGNOSTICS.md:292:\`ec_spire_recover_orphaned_prepared_xacts(node_id)\` helper. The helper cannot`
